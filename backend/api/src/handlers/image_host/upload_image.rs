use actix_multipart::form::{MultipartForm, bytes::Bytes};
use actix_web::{HttpResponse, web::Data};
use arcadia_common::error::{Error, Result};
use arcadia_storage::redis::RedisPoolInterface;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    Arcadia,
    middlewares::auth_middleware::Authdata,
    services::image_host_service::{self, Uploader},
};

#[derive(Debug, MultipartForm, ToSchema)]
pub struct UploadImageForm {
    #[schema(value_type = String, format = Binary, content_media_type = "application/octet-stream")]
    pub image: Bytes,
}

#[derive(Serialize, ToSchema)]
pub struct UploadImageResponse {
    pub url: String,
}

#[utoipa::path(
    post,
    operation_id = "Upload image",
    tag = "Image Host",
    path = "/api/image-host/upload",
    request_body(content = UploadImageForm, content_type = "multipart/form-data"),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully uploaded the image", body = UploadImageResponse),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: MultipartForm<UploadImageForm>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    let image_host = arc
        .image_host
        .as_ref()
        .ok_or(Error::ImageHostNotConfigured)?;
    let uploader = Uploader {
        id: user.sub,
        username: arc.pool.find_user_with_id(user.sub).await?.username,
    };

    let url = image_host_service::upload_image(
        image_host,
        &uploader,
        &form.image.data,
        form.image.file_name.as_deref().unwrap_or("image"),
    )
    .await?;

    Ok(HttpResponse::Ok().json(UploadImageResponse { url }))
}
