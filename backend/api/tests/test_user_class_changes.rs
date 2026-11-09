use arcadia_periodic_tasks::periodic_tasks::user_classes::process_user_class_changes;
use arcadia_storage::connection_pool::ConnectionPool;
use sqlx::PgPool;
use std::sync::Arc;

async fn class_name_of(pool: &ConnectionPool, user_id: i32) -> String {
    pool.find_user_with_id(user_id).await.unwrap().class_name
}

#[sqlx::test(fixtures("with_automatic_user_class_changes"))]
async fn test_process_user_class_changes(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let changes = process_user_class_changes(Arc::clone(&pool)).await.unwrap();
    assert_eq!(changes, 2);

    // meets the elite requirements too, but only climbs one class per run
    assert_eq!(class_name_of(&pool, 2000).await, "auto_member");
    // not promoted to invite_only, which has no requirement but no automatic promotion
    assert_eq!(class_name_of(&pool, 2001).await, "auto_newbie");
    assert_eq!(class_name_of(&pool, 2002).await, "auto_newbie");
    assert_eq!(class_name_of(&pool, 2003).await, "auto_newbie");
    assert_eq!(class_name_of(&pool, 2004).await, "sticky_member");

    let changes = process_user_class_changes(Arc::clone(&pool)).await.unwrap();
    assert_eq!(changes, 1);
    assert_eq!(class_name_of(&pool, 2000).await, "auto_elite");
}

#[sqlx::test(fixtures("with_many_promotable_users"))]
async fn test_process_user_class_changes_goes_through_every_batch(pool: PgPool) {
    let pool = Arc::new(ConnectionPool::with_pg_pool(pool));

    let changes = process_user_class_changes(Arc::clone(&pool)).await.unwrap();
    assert_eq!(changes, 250);
    assert_eq!(class_name_of(&pool, 3000).await, "batch_member");
    assert_eq!(class_name_of(&pool, 3249).await, "batch_member");
}
