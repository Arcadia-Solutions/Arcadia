use crate::Arcadia;
use actix_web::{
    HttpResponse,
    web::{Data, Query},
};
use arcadia_common::error::Result;
use arcadia_storage::{models::user::UserLite, redis::RedisPoolInterface};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct GetUserLiteQuery {
    username: String,
}

#[utoipa::path(
    get,
    operation_id = "Search users lite",
    tag = "Search",
    path = "/api/search/users/lite",
    params (GetUserLiteQuery),
    description = "Case insensitive",
    responses(
        (status = 200, description = "Successfully got the users and some data about them", body=Vec<UserLite>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    query: Query<GetUserLiteQuery>,
    arc: Data<Arcadia<R>>,
) -> Result<HttpResponse> {
    let users = arc.pool.find_users_lite(&query.username).await?;

    Ok(HttpResponse::Ok().json(users))
}
