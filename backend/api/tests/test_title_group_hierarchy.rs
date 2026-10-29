pub mod common;
pub mod mocks;

use actix_web::{http::StatusCode, test};
use arcadia_storage::models::title_group::TitleGroupAndAssociatedData;
use arcadia_storage::{connection_pool::ConnectionPool, models::torrent::PeerStatus};
use common::{auth_header, create_test_app_and_login, TestUser};
use mocks::mock_redis::MockRedisPool;
use sqlx::PgPool;
use std::sync::Arc;

#[sqlx::test(
    fixtures(
        "with_test_users",
        "with_test_artist",
        "with_test_series",
        "with_test_title_group",
        "with_test_edition_group",
        "with_test_torrent",
        "with_test_affiliated_artist",
        "with_test_title_group_comments",
        "with_test_torrent_request",
        "with_test_torrent_request_vote",
        "with_test_collage",
        "with_test_collage_entry",
        "with_test_peers",
        "with_test_title_group_hierarchy",
        "with_test_title_group_bookmark",
    ),
    migrations = "../storage/migrations"
)]
async fn test_get_title_group_hierarchy_returns_all_associated_data(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));
    let (service, user) =
        create_test_app_and_login(pool, MockRedisPool::default(), TestUser::Standard).await;

    let req = test::TestRequest::get()
        .uri("/api/title-groups?id=1")
        .insert_header(auth_header(&user.token))
        .to_request();

    let data: TitleGroupAndAssociatedData =
        common::call_and_read_body_json_with_status(&service, req, StatusCode::OK).await;

    // Title group
    assert_eq!(data.title_group.id, 1);
    assert_eq!(data.title_group.name, "Love Me Do / P.S. I Love You");
    assert_eq!(data.title_group.master_group_id, Some(1));
    assert_eq!(data.title_group.series_id, Some(1));

    // Series
    let series = data.series.expect("series should be present");
    assert_eq!(series.id, 1);
    assert_eq!(series.name, "Test Series");

    // Edition groups (1 for title_group 1)
    assert_eq!(data.edition_groups.len(), 1);
    let edition_group = &data.edition_groups[0];
    assert_eq!(
        edition_group.name,
        Some("red label Parlophone single".to_string())
    );

    // Torrents nested inside edition group
    assert_eq!(edition_group.torrents.len(), 1);
    let torrent = &edition_group.torrents[0];
    assert_eq!(torrent.id, 1);
    assert!(
        torrent.release_name.as_deref().unwrap().contains("Beatles"),
        "release_name should contain 'Beatles'"
    );

    // Torrent reports
    assert_eq!(torrent.reports.len(), 1);
    assert_eq!(
        torrent.reports[0].description,
        "Possible trump: better source available"
    );

    // Peer status: user 100 has an active seeder peer (from with_test_peers)
    // AND a completed activity (from with_test_title_group_hierarchy).
    // Active peer takes priority, so status should be Seeding.
    assert!(
        torrent.peer_status.is_some(),
        "peer_status should be present for user with active peer"
    );
    assert_eq!(
        torrent.peer_status,
        Some(PeerStatus::Seeding),
        "peer_status should be present for user with active peer"
    );

    // Affiliated artists (2: The Beatles as main, George Martin as guest)
    assert_eq!(data.affiliated_artists.len(), 2);
    let artist_names: Vec<&str> = data
        .affiliated_artists
        .iter()
        .map(|a| a.artist.name.as_str())
        .collect();
    assert!(artist_names.contains(&"The Beatles"));
    assert!(artist_names.contains(&"George Martin"));

    // Comments (2 on title_group 1 from with_test_title_group_comments)
    assert_eq!(data.title_group_comments.len(), 2);
    assert_eq!(
        data.title_group_comments[0].content,
        "This is a great album!"
    );

    // Torrent requests (1 with bounty)
    assert_eq!(data.torrent_requests.len(), 1);
    assert_eq!(
        data.torrent_requests[0].torrent_request.edition_name,
        Some("red label Parlophone single".to_string())
    );
    assert!(
        data.torrent_requests[0].bounty.upload > 0,
        "bounty upload should be > 0"
    );
    assert_eq!(data.torrent_requests[0].user_votes_amount, 1);

    // Subscriptions (user 100 is subscribed to both from with_test_subscriptions)
    assert!(data.is_subscribed_to_torrents);
    assert!(data.is_subscribed_to_comments);

    // Bookmark (user 100 bookmarked title_group 1 from with_test_title_group_bookmark)
    assert!(data.bookmark_id.is_some());

    // Same master group (title_group 3 "Please Please Me" should appear)
    assert_eq!(data.in_same_master_group.len(), 1);
    assert_eq!(data.in_same_master_group[0].name, "Please Please Me");
    assert_eq!(data.in_same_master_group[0].id, 3);

    // Collages (title_group 1 is in collage 1 from with_test_collage_entry)
    assert_eq!(data.collages.len(), 1);
    assert_eq!(data.collages[0].name, "Test Collage");
    assert!(data.collages[0].entries_amount > 0);
}
