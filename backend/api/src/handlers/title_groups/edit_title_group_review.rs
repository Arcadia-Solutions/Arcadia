use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json, Path},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        title_group_review::{EditedTitleGroupReview, TitleGroupReview, validate_rating},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit title group review",
    tag = "Title Group",
    path = "/api/title-groups/reviews/{id}",
    request_body = EditedTitleGroupReview,
    params(
        ("id" = i64, Path, description = "Review id")
    ),
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the review", body = TitleGroupReview),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i64>,
    form: Json<EditedTitleGroupReview>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::RateTitleGroups, req.path())
        .await?;
    validate_rating(form.rating)?;

    let review_id = path.into_inner();
    if arc.pool.find_title_group_review(review_id).await?.user_id != user.sub {
        return Err(Error::InsufficientPermissions(
            "you can only edit your own review".to_string(),
        ));
    }

    let updated_review = arc.pool.update_title_group_review(&form, review_id).await?;

    Ok(HttpResponse::Ok().json(updated_review))
}
