use crate::{
    Arcadia,
    handlers::emojis::{
        extract_emoji_representation, validate_emoji_name, validate_emoji_representation,
    },
    middlewares::auth_middleware::Authdata,
};
use actix_multipart::form::{MultipartForm, bytes::Bytes, text::Text};
use actix_web::{HttpRequest, HttpResponse, web::Data};
use arcadia_common::error::Result;
use arcadia_storage::{
    models::{emoji::Emoji, user::UserPermission},
    redis::RedisPoolInterface,
};
use utoipa::ToSchema;

#[derive(Debug, MultipartForm, ToSchema)]
pub struct UserCreatedEmojiForm {
    #[schema(value_type = String)]
    pub name: Text<String>,
    #[schema(value_type = Option<String>)]
    pub unicode_character: Option<Text<String>>,
    #[schema(value_type = Option<String>, format = Binary, content_media_type = "application/octet-stream")]
    pub image: Option<Bytes>,
}

#[utoipa::path(
    post,
    operation_id = "Create emoji",
    tag = "Emojis",
    path = "/api/emojis",
    request_body(content = UserCreatedEmojiForm, content_type = "multipart/form-data"),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 201, description = "Successfully created the emoji", body=Emoji),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: MultipartForm<UserCreatedEmojiForm>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::EditArcadiaSettings, req.path())
        .await?;

    let name = validate_emoji_name(&form.name)?;

    let (unicode_character, image, image_mime_type) =
        extract_emoji_representation(form.unicode_character.as_ref(), form.image.as_ref());

    validate_emoji_representation(
        unicode_character,
        image.map(|bytes| (image_mime_type.as_deref().unwrap_or_default(), bytes)),
    )?;

    let emoji = arc
        .pool
        .create_emoji(name, unicode_character, image, image_mime_type.as_deref())
        .await?;

    Ok(HttpResponse::Created().json(emoji))
}
