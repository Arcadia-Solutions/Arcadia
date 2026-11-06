pub mod create_emoji;
pub mod delete_emoji;
pub mod edit_emoji;
pub mod get_emoji_image;
pub mod get_emojis;
pub mod get_emojis_usage;
pub mod reorder_emojis;
pub mod set_emoji_enabled;

use actix_multipart::form::{MultipartFormConfig, bytes::Bytes, text::Text};
use actix_web::web::{ServiceConfig, delete, get, post, put, resource};
use arcadia_common::error::{Error, Result};
use arcadia_storage::redis::RedisPoolInterface;

/// A custom emoji image is small on purpose: it is stored in the database and rendered inline.
pub const MAX_EMOJI_IMAGE_SIZE: usize = 32 * 1024;
const ALLOWED_EMOJI_IMAGE_MIME_TYPES: [&str; 4] =
    ["image/png", "image/webp", "image/gif", "image/svg+xml"];

/// Checks that the name is neither blank nor longer than `MAX_EMOJI_NAME_LENGTH`, and returns it
/// trimmed so a name only differing by surrounding whitespace still collides with the unique
/// index instead of creating a second, visually identical emoji.
pub fn validate_emoji_name(name: &str) -> Result<&str> {
    let name = name.trim();

    if name.is_empty() {
        return Err(Error::InvalidEmojiName);
    }

    Ok(name)
}

/// Checks that the emoji has exactly one representation, and that an image is small enough and
/// of an allowed type. Returns the mime type of the image when there is one.
pub fn validate_emoji_representation(
    unicode_character: Option<&str>,
    image: Option<(&str, &[u8])>,
) -> Result<()> {
    match (unicode_character, image) {
        (Some(_), None) => Ok(()),
        (None, Some((mime_type, bytes))) => {
            if !ALLOWED_EMOJI_IMAGE_MIME_TYPES.contains(&mime_type) {
                return Err(Error::InvalidEmojiImageMimeType);
            }
            if bytes.len() > MAX_EMOJI_IMAGE_SIZE {
                return Err(Error::EmojiImageTooLarge);
            }
            Ok(())
        }
        _ => Err(Error::EmojiMustHaveExactlyOneRepresentation),
    }
}

/// Pulls the unicode character, the image bytes and the image mime type out of the multipart
/// fields shared by the create and the edit emoji forms, so a future validation change on this
/// extraction cannot be applied to one path and missed on the other.
///
/// A part that is present but carries nothing (an empty text field, a zero byte file) counts as
/// absent: a browser form submits every field it knows about, and an empty one means "not
/// provided". Without this, an empty `unicode_character` would satisfy the "exactly one
/// representation" check while storing an emoji that renders as nothing, and on edit it would
/// wipe the image of an existing image emoji.
pub fn extract_emoji_representation<'a>(
    unicode_character: Option<&'a Text<String>>,
    image: Option<&'a Bytes>,
) -> (Option<&'a str>, Option<&'a [u8]>, Option<String>) {
    let unicode_character = unicode_character
        .map(|text| text.0.trim())
        .filter(|character| !character.is_empty());
    let image = image.filter(|image| !image.data.is_empty());
    let image_mime_type = image
        .and_then(|image| image.content_type.as_ref())
        .map(|mime_type| mime_type.essence_str().to_string());
    let image = image.map(|image| image.data.as_ref());

    (unicode_character, image, image_mime_type)
}

pub fn config<R: RedisPoolInterface + 'static>(cfg: &mut ServiceConfig) {
    cfg.app_data(
        // The transport ceiling is twice `MAX_EMOJI_IMAGE_SIZE` because the multipart form also
        // carries the other text fields (name, unicode character) alongside the
        // image bytes, and multipart encoding itself adds boundary and header overhead. The
        // per-image cap is still enforced separately in `validate_emoji_representation`.
        MultipartFormConfig::default()
            .total_limit(MAX_EMOJI_IMAGE_SIZE * 2)
            .memory_limit(MAX_EMOJI_IMAGE_SIZE * 2),
    );
    cfg.service(
        resource("")
            .route(get().to(self::get_emojis::exec::<R>))
            .route(post().to(self::create_emoji::exec::<R>))
            .route(put().to(self::edit_emoji::exec::<R>))
            .route(delete().to(self::delete_emoji::exec::<R>)),
    );
    cfg.service(resource("/reorder").route(put().to(self::reorder_emojis::exec::<R>)));
    cfg.service(resource("/enabled").route(put().to(self::set_emoji_enabled::exec::<R>)));
    cfg.service(resource("/usage").route(get().to(self::get_emojis_usage::exec::<R>)));
    cfg.service(resource("/{emoji_id}/image").route(get().to(self::get_emoji_image::exec::<R>)));
}
