use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::arcadia_settings::AvailableShopItem, redis::RedisPoolInterface,
    services::promotion_service::meets_requirements,
};

#[utoipa::path(
    post,
    operation_id = "Buy promotion",
    tag = "Shop",
    path = "/api/shop/buy-promotion",
    security(("http" = ["Bearer"])),
    responses(
        (status = 200, description = "Successfully bought promotion"),
        (status = 400, description = "Cannot buy promotion - requirements not met or promotion not available"),
        (status = 403, description = "Shop item not available"),
        (status = 409, description = "Not enough bonus points"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    {
        let settings = arc.settings.lock().unwrap();
        if !settings
            .available_shop_items
            .contains(&AvailableShopItem::UserClassPromotion)
        {
            return Err(Error::ShopItemNotAvailable);
        }
    }

    let user_stats = arc.pool.get_user_stats(current_user.sub).await?;

    if user_stats.class_locked {
        return Err(Error::UserClassLocked);
    }

    let next_class = arc
        .pool
        .get_next_user_class(&user_stats.class_name)
        .await?
        .ok_or_else(|| {
            Error::PromotionNotAvailable("No next class available for promotion".to_string())
        })?;

    // Check if promotion can be bought (cost > 0 means it can be purchased)
    let cost = next_class.promotion_cost_bonus_points;
    if cost == 0 {
        return Err(Error::PromotionNotAvailable(
            "This class promotion cannot be bought".to_string(),
        ));
    }

    if user_stats.warned && !next_class.promotion_allowed_while_warned {
        return Err(Error::PromotionNotAvailable(
            "Cannot buy promotion while warned".to_string(),
        ));
    }

    if !meets_requirements(&user_stats, &next_class) {
        return Err(Error::PromotionNotAvailable(
            "You do not meet the requirements for this promotion".to_string(),
        ));
    }

    if user_stats.bonus_points < cost {
        return Err(Error::NotEnoughBonusPointsAvailable);
    }

    arc.pool
        .purchase_promotion(current_user.sub, &next_class.name, cost)
        .await?;

    Ok(HttpResponse::Ok().finish())
}
