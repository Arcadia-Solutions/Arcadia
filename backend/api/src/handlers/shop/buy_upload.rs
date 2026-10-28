use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::arcadia_settings::AvailableShopItem,
    models::shop::{BuyUploadRequest, UploadDiscountTier},
    redis::RedisPoolInterface,
    services::shop_service::calculate_upload_price,
};

const BYTES_PER_GB: i64 = 1_073_741_824;

#[utoipa::path(
    post,
    operation_id = "Buy upload",
    tag = "Shop",
    path = "/api/shop/buy-upload",
    security(("http" = ["Bearer"])),
    request_body = BuyUploadRequest,
    responses(
        (status = 201, description = "Successfully bought upload"),
        (status = 400, description = "Invalid amount"),
        (status = 403, description = "Shop item not available"),
        (status = 409, description = "Not enough bonus points"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    request: Json<BuyUploadRequest>,
    current_user: Authdata,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    if request.bytes < BYTES_PER_GB {
        return Err(Error::InvalidShopPurchaseAmount);
    }

    let price_calculation = {
        let settings = arc.settings.lock().unwrap();
        if !settings
            .available_shop_items
            .contains(&AvailableShopItem::UploadAmount)
        {
            return Err(Error::ShopItemNotAvailable);
        }
        let discount_tiers: Vec<UploadDiscountTier> =
            serde_json::from_value(settings.shop_upload_discount_tiers.clone())?;
        calculate_upload_price(
            request.bytes,
            settings.shop_upload_base_price_per_gb,
            &discount_tiers,
        )
    };

    arc.pool
        .purchase_upload(
            current_user.sub,
            request.bytes,
            price_calculation.final_price,
        )
        .await?;

    Ok(HttpResponse::Created().finish())
}
