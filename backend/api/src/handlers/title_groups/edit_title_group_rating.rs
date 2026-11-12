use crate::{Arcadia, middlewares::auth_middleware::Authdata};
use actix_web::{
    HttpRequest, HttpResponse,
    web::{Data, Json, Path},
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        title_group_rating::{EditedTitleGroupRating, TitleGroupRating, validate_rating},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    put,
    operation_id = "Edit title group rating",
    tag = "Title Group",
    path = "/api/title-groups/ratings/{id}",
    request_body = EditedTitleGroupRating,
    params(
        ("id" = i64, Path, description = "Rating id")
    ),
    security(
        ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully edited the rating", body = TitleGroupRating),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    path: Path<i64>,
    form: Json<EditedTitleGroupRating>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
    req: HttpRequest,
) -> Result<HttpResponse> {
    arc.pool
        .require_permission(user.sub, &UserPermission::RateTitleGroups, req.path())
        .await?;
    validate_rating(form.rating)?;

    let rating_id = path.into_inner();
    if arc.pool.find_title_group_rating(rating_id).await?.user_id != user.sub {
        return Err(Error::InsufficientPermissions(
            "you can only edit your own rating".to_string(),
        ));
    }

    let updated_rating = arc.pool.update_title_group_rating(&form, rating_id).await?;

    Ok(HttpResponse::Ok().json(updated_rating))
}
