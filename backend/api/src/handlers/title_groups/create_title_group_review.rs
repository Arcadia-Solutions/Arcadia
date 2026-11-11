use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        title_group_review::{TitleGroupReview, UserCreatedTitleGroupReview, validate_rating},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Create title group review",
    tag = "Title Group",
    path = "/api/title-groups/reviews",
    request_body = UserCreatedTitleGroupReview,
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 201, description = "Successfully created the review", body = TitleGroupReview),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    review: Json<UserCreatedTitleGroupReview>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::RateTitleGroups, req.path())
        .await?;
    validate_rating(review.rating)?;

    let requires_snatch_or_seeding = arc
        .settings
        .lock()
        .unwrap()
        .title_group_review_requires_snatch_or_seeding;
    if requires_snatch_or_seeding
        && !arc
            .pool
            .has_snatched_or_seeded_title_group(user.sub, review.title_group_id)
            .await?
    {
        return Err(Error::TitleGroupReviewRequiresSnatchOrSeeding);
    }

    let created_review = arc
        .pool
        .create_title_group_review(&review, user.sub)
        .await?;

    Ok(HttpResponse::Created().json(created_review))
}
