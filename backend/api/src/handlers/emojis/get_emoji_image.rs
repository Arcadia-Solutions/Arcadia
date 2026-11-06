use crate::Arcadia;
use actix_web::{
    HttpResponse,
    http::header::{CacheControl, CacheDirective, ETag, EntityTag},
    web::{Data, Path},
};
use arcadia_common::error::Result;
use arcadia_storage::redis::RedisPoolInterface;

/// A custom emoji image can be an SVG, which is served from our own origin and can carry
/// script. `sandbox` disables script execution and every other active capability, and
/// `default-src 'none'` blocks the document from loading anything else; `style-src
/// 'unsafe-inline'` is kept because inline styling is how SVGs commonly draw themselves and
/// carries no script risk. Applied to every emoji image response, not only SVG ones, so the
/// policy cannot be forgotten if another image type gains script capability later.
const EMOJI_IMAGE_CONTENT_SECURITY_POLICY: &str =
    "default-src 'none'; style-src 'unsafe-inline'; sandbox";

#[utoipa::path(
    get,
    operation_id = "Get emoji image",
    tag = "Emojis",
    path = "/api/emojis/{emoji_id}/image",
    params(
        ("emoji_id" = i32, Path, description = "ID of the emoji whose image is requested")
    ),
    responses(
        (status = 200, description = "Successfully retrieved the emoji image", content_type = "image/png, image/webp, image/gif, image/svg+xml"),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    emoji_id: Path<i32>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let emoji_image = arc.pool.find_emoji_image(emoji_id.into_inner()).await?;

    Ok(HttpResponse::Ok()
        .content_type(emoji_image.image_mime_type)
        .insert_header(CacheControl(vec![
            CacheDirective::Public,
            CacheDirective::MaxAge(31_536_000),
            CacheDirective::Extension("immutable".to_string(), None),
        ]))
        .insert_header(ETag(EntityTag::new_strong(
            emoji_image.image_version.to_string(),
        )))
        .insert_header((
            "Content-Security-Policy",
            EMOJI_IMAGE_CONTENT_SECURITY_POLICY,
        ))
        .insert_header(("X-Content-Type-Options", "nosniff"))
        .body(emoji_image.image))
}
