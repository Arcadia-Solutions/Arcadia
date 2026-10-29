use crate::{middlewares::auth_middleware::Authdata, Arcadia};
use actix_web::{
    web::{Data, Json},
    HttpResponse,
};
use arcadia_common::error::{Error, Result};
use arcadia_storage::{
    models::{
        collage::{CollageCategory, CollageEntry, UserCreatedCollageEntry},
        user::UserPermission,
    },
    redis::RedisPoolInterface,
};

#[utoipa::path(
    post,
    operation_id = "Inserts entries into a collage",
    tag = "Collages",
    path = "/api/collages/entries",
    security(
      ("http" = ["Bearer"])
    ),
    responses(
        (status = 200, description = "Successfully created the collage entries", body=Vec<CollageEntry>),
    )
)]
pub async fn exec<R: RedisPoolInterface + 'static>(
    collage_entries: Json<Vec<UserCreatedCollageEntry>>,
    arc: Data<Arcadia<R>>,
    user: Authdata,
) -> Result<HttpResponse> {
    // personal collages and staff picks are curated by their creator, other collages are open to everyone
    if !arc
        .pool
        .user_has_permission(user.sub, &UserPermission::EditCollage)
        .await?
    {
        let mut collage_ids = collage_entries
            .iter()
            .map(|entry| entry.collage_id)
            .collect::<Vec<_>>();
        collage_ids.sort_unstable();
        collage_ids.dedup();

        for collage_id in collage_ids {
            let collage = arc.pool.find_collage(collage_id).await?;
            if matches!(
                collage.category,
                CollageCategory::Personal | CollageCategory::StaffPicks
            ) && collage.created_by_id != user.sub
            {
                return Err(Error::InsufficientPermissions(format!(
                    "{:?}",
                    UserPermission::EditCollage
                )));
            }
        }
    }

    // TODO: instead, we should return Vec<CollageAndAssociatedData> so the frontend can hydrate the page
    // without having to refetch everything
    let collage_entries = arc
        .pool
        .create_collage_entries(&collage_entries, user.sub, &arc.notification_sender)
        .await?;

    Ok(HttpResponse::Created().json(collage_entries))
}
