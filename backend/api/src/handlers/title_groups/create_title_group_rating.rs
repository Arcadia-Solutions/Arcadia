use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        title_group_rating::{TitleGroupRating, UserCreatedTitleGroupRating, validate_rating},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create title group rating",
    tag = "Title Group",
    path = "/api/title-groups/ratings",
    request_body = UserCreatedTitleGroupRating,
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 201, description = "Successfully created the rating", body = TitleGroupRating),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    rating: Json<UserCreatedTitleGroupRating>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::RateTitleGroups, req.path())
        .await?;
    validate_rating(rating.rating)?;

    let requires_snatch_or_seeding = arc
        .settings
        .lock()
        .unwrap()
        .title_group_rating_requires_snatch_or_seeding;
    if requires_snatch_or_seeding
        && !arc
            .pool
            .has_snatched_or_seeded_title_group(user.sub, rating.title_group_id)
            .await?
    {
        return Err(Error::TitleGroupRatingRequiresSnatchOrSeeding);
    }

    let created_rating = arc
        .pool
        .create_title_group_rating(&rating, user.sub)
        .await?;

    Ok(HttpResponse::Created().json(created_rating))
}
