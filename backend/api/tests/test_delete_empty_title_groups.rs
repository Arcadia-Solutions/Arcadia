use arcadia_storage::connection_pool::ConnectionPool;
use chrono::{TimeZone, Utc};
use sqlx::PgPool;
use std::{borrow::Borrow, sync::Arc};

// Title group 1 has no torrent but a request, 2 has no torrent, 3 and 4 only have deleted torrents
#[sqlx::test(fixtures(
    "with_test_users",
    "with_test_title_group",
    "with_test_torrent_request",
    "with_deletable_title_group",
    "with_test_collage"
))]
async fn test_delete_empty_title_groups(pool: PgPool) {
    sqlx::query("UPDATE title_groups SET created_at = '2020-01-01T00:00:00Z' WHERE id = 4")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO collage_entry (created_by_id, collage_id, title_group_id) VALUES (100, 1, 3)",
    )
    .execute(&pool)
    .await
    .unwrap();
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let ids = |groups: Vec<arcadia_storage::models::title_group::TitleGroupHierarchyLite>| {
        groups.into_iter().map(|tg| tg.id).collect::<Vec<_>>()
    };
    let empty_title_groups = pool.find_empty_title_groups(None, None).await.unwrap();
    assert_eq!(empty_title_groups[2].affiliated_artists[0].artist_id, 2);
    assert_eq!(ids(empty_title_groups), [4, 2, 3]);
    assert_eq!(pool.count_empty_title_groups(None, None).await.unwrap(), 3);
    let after_2024 = Some(Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap());
    assert_eq!(
        ids(pool
            .find_empty_title_groups(after_2024, None)
            .await
            .unwrap()),
        [2, 3]
    );
    assert_eq!(
        ids(pool
            .find_empty_title_groups(None, after_2024)
            .await
            .unwrap()),
        [4]
    );

    // Title group 1 is not empty, so asking for it explicitly does nothing to it
    let deleted = pool
        .delete_empty_title_groups(None, None, Some(&[1, 3]))
        .await
        .unwrap();
    assert_eq!(deleted, 1);
    assert!(pool.find_title_group(1).await.is_ok());
    assert!(pool.find_title_group(3).await.is_err());
    assert!(pool.find_title_group(4).await.is_ok());

    let artist = pool.find_artist_by_id(2).await.unwrap();
    assert_eq!(artist.title_groups_amount, 0);
    assert_eq!(artist.edition_groups_amount, 0);
    assert_eq!(artist.torrents_amount, 0);
    let (title_groups, edition_groups): (i32, i32) =
        sqlx::query_as("SELECT title_groups, edition_groups FROM users WHERE id = 100")
            .fetch_one((*pool).borrow())
            .await
            .unwrap();
    assert_eq!((title_groups, edition_groups), (1, 1));

    assert_eq!(
        pool.delete_empty_title_groups(None, None, None)
            .await
            .unwrap(),
        2
    );
    assert!(pool.find_title_group(2).await.is_err());
    assert!(pool.find_title_group(4).await.is_err());
}
