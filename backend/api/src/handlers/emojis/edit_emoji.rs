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
pub struct EditedEmojiForm {
    #[schema(value_type = i32)]
    pub id: Text<i32>,
    #[schema(value_type = String)]
    pub name: Text<String>,
    #[schema(value_type = Option<String>)]
    pub unicode_character: Option<Text<String>>,
    #[schema(value_type = Option<String>, format = Binary, content_media_type = "application/octet-stream")]
    pub image: Option<Bytes>,
}

#[utoipa::path(
    put,
    operation_id = "Edit emoji",
    tag = "Emojis",
    path = "/api/emojis",
    request_body(content = EditedEmojiForm, content_type = "multipart/form-data"),
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the emoji", body=Emoji),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    form: MultipartForm<EditedEmojiForm>,
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

    // Sending neither of them keeps the current representation, so there is nothing to validate.
    if unicode_character.is_some() || image.is_some() {
        validate_emoji_representation(
            unicode_character,
            image.map(|bytes| (image_mime_type.as_deref().unwrap_or_default(), bytes)),
        )?;
    }

    let emoji = arc
        .pool
        .update_emoji(
            form.id.0,
            name,
            unicode_character,
            image,
            image_mime_type.as_deref(),
        )
        .await?;

    Ok(HttpResponse::Ok().json(emoji))
}
