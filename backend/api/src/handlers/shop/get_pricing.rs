use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::arcadia_settings::AvailableShopItem,
    models::shop::{FreeleechTokenDiscountTier, PromotionPricing, ShopPricing, UploadDiscountTier},
    redis::RedisPoolInterface,
    services::promotion_service::meets_requirements,
};

#[utoipa::path(
    get,
    operation_id = "Get shop pricing",
    tag = "Shop",
    path = "/api/shop/pricing",
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "Shop pricing and discount tiers", body = ShopPricing),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let (
        upload_base_price_per_gb,
        upload_discount_tiers,
        freeleech_token_base_price,
        freeleech_token_discount_tiers,
        available_shop_items,
    ) = {
        let settings = arc.settings.lock().unwrap();
        (
            settings.shop_upload_base_price_per_gb,
            serde_json::from_value::<Vec<UploadDiscountTier>>(
                settings.shop_upload_discount_tiers.clone(),
            )?,
            settings.shop_freeleech_token_base_price,
            serde_json::from_value::<Vec<FreeleechTokenDiscountTier>>(
                settings.shop_freeleech_token_discount_tiers.clone(),
            )?,
            settings.available_shop_items.clone(),
        )
    };

    let promotion = if available_shop_items.contains(&AvailableShopItem::UserClassPromotion) {
        get_promotion_pricing(&arc, current_user.sub).await?
    } else {
        None
    };

    let pricing = ShopPricing {
        upload_base_price_per_gb: if available_shop_items.contains(&AvailableShopItem::UploadAmount)
        {
            Some(upload_base_price_per_gb)
        } else {
            None
        },
        upload_discount_tiers: if available_shop_items.contains(&AvailableShopItem::UploadAmount) {
            Some(upload_discount_tiers)
        } else {
            None
        },
        freeleech_token_base_price: if available_shop_items
            .contains(&AvailableShopItem::FreeleechTokens)
        {
            Some(freeleech_token_base_price)
        } else {
            None
        },
        freeleech_token_discount_tiers: if available_shop_items
            .contains(&AvailableShopItem::FreeleechTokens)
        {
            Some(freeleech_token_discount_tiers)
        } else {
            None
        },
        promotion,
    };

    Ok(HttpResponse::Ok().json(pricing))
}

async fn get_promotion_pricing<R: RedisPoolInterface + 'static>(
    arc: &Data<Arcadia<R>>,
    user_id: i32,
) -> Result<Option<PromotionPricing>> {
    let user_stats = arc.pool.get_user_stats(user_id).await?;

    if user_stats.class_locked {
        return Ok(None);
    }

    let next_class = match arc.pool.get_next_user_class(&user_stats.class_name).await? {
        Some(class) => class,
        None => return Ok(None),
    };

    let cost = next_class.promotion_cost_bonus_points;
    if cost == 0 {
        return Ok(None);
    }

    if user_stats.warned && !next_class.promotion_allowed_while_warned {
        return Ok(None);
    }

    let requirements_met = meets_requirements(&user_stats, &next_class);

    Ok(Some(PromotionPricing {
        next_class_name: next_class.name,
        cost,
        requirements_met,
    }))
}
