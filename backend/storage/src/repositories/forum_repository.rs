use crate::{
    connection_pool::ConnectionPool,
    models::{
        common::PaginatedResults,
        forum::{
            CreateRelatedForumThread, DeleteRelatedForumThreadQuery, EditedForumCategory,
            EditedForumPost, EditedForumSubCategory, EditedForumThread, ForumCategory,
            ForumCategoryHierarchy, ForumCategoryLite, ForumPoll, ForumPollHierarchy,
            ForumPollOptionResult, ForumPost, ForumPostAndThreadName, ForumPostHierarchy,
            ForumPostSearchQuery, ForumPostWithLocation, ForumSearchQuery, ForumSearchResult,
            ForumSubCategory, ForumSubCategoryHierarchy, ForumThread, ForumThreadEnriched,
            ForumThreadEnrichedHierarchy, ForumThreadPostLite, ForumThreadSortBy,
            ForumThreadSortDirection, GetForumThreadPostsQuery, PinForumThread, RelatedForumThread,
            ReorderForumCategories, ReorderForumSubCategories, UserCreatedForumCategory,
            UserCreatedForumPoll, UserCreatedForumPollVote, UserCreatedForumPost,
            UserCreatedForumSubCategory, UserCreatedForumThread,
        },
        notification::NotificationEvent,
        site_highlight::SiteHighlightItemType,
        user::{UserLite, UserLiteAvatar},
    },
};
use arcadia_common::error::{Error, Result};
use chrono::{DateTime, Local, Utc};
use serde_json::Value;
use sqlx::{prelude::FromRow, PgPool, Postgres, Transaction};
use std::borrow::Borrow;
use tokio::sync::broadcast;

#[derive(FromRow)]
#[allow(dead_code)]
struct DBImportSubCategoryWithLatestPost {
    id: i32,
    name: String,
    sort_order: i32,
    threads_amount: i64,
    posts_amount: i64,
    forbidden_classes: Vec<String>,
    new_threads_restricted: bool,
    thread_sort_by: ForumThreadSortBy,
    thread_sort_direction: ForumThreadSortDirection,
    forum_category_id: i32,
    category_name: String,
    latest_post_id: Option<i64>,
    thread_id: Option<i64>,
    thread_name: Option<String>,
    latest_post_created_at: Option<DateTime<Utc>>,
    user_id: Option<i32>,
    username: Option<String>,
    warned: Option<bool>,
    banned: Option<bool>,
}

#[derive(Debug, FromRow)]
struct DBImportForumPost {
    id: i64,
    content: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    sticky: bool,
    locked: bool,
    forum_thread_id: i64,
    created_by_user_id: i32,
    created_by_user_username: String,
    created_by_user_class_name: String,
    created_by_user_avatar: Option<String>,
    created_by_user_banned: bool,
    created_by_user_warned: bool,
    created_by_user_custom_title: Option<String>,
}

#[derive(Debug, FromRow)]
struct DBImportForumPostWithLocation {
    id: i64,
    content: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    sticky: bool,
    locked: bool,
    forum_thread_id: i64,
    created_by_user_id: i32,
    created_by_user_username: String,
    created_by_user_class_name: String,
    created_by_user_avatar: Option<String>,
    created_by_user_banned: bool,
    created_by_user_warned: bool,
    created_by_user_custom_title: Option<String>,
    thread_name: String,
    sub_category_id: i32,
    sub_category_name: String,
    category_id: i32,
    category_name: String,
}

impl ConnectionPool {
    pub async fn create_forum_post(
        &self,
        forum_post: &UserCreatedForumPost,
        current_user_id: i32,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<ForumPost> {
        if forum_post.content.trim().is_empty() {
            return Err(Error::ForumPostEmpty);
        }

        let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;

        let thread = sqlx::query!(
            r#"SELECT locked FROM forum_threads WHERE id = $1"#,
            forum_post.forum_thread_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateForumPost)?;

        if thread.locked {
            return Err(Error::ForumThreadLocked);
        }

        let created_forum_post = sqlx::query_as!(
            ForumPost,
            r#"
                INSERT INTO forum_posts (content, created_by_id, forum_thread_id)
                VALUES ($1, $2, $3)
                RETURNING id, forum_thread_id, created_at, updated_at, created_by_id, content, sticky, locked
            "#,
            forum_post.content,
            current_user_id,
            forum_post.forum_thread_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateForumPost)?;

        sqlx::query!(
            r#"
            UPDATE forum_threads
            SET posts_amount = posts_amount + 1
            WHERE id = $1;
            "#,
            forum_post.forum_thread_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateForumPost)?;

        sqlx::query!(
            r#"
            UPDATE forum_sub_categories
            SET posts_amount = posts_amount + 1
            WHERE id = (SELECT forum_sub_category_id FROM forum_threads WHERE id = $1);
            "#,
            forum_post.forum_thread_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateForumPost)?;

        sqlx::query!(
            r#"
            UPDATE users
            SET forum_posts = forum_posts + 1
            WHERE id = $1;
            "#,
            current_user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateForumPost)?;

        let user_ids = Self::notify_users_forum_thread_posts(
            &mut tx,
            forum_post.forum_thread_id,
            created_forum_post.id,
            current_user_id,
        )
        .await?;

        // the thread should be marked as read for the user who created the post
        Self::upsert_forum_thread_read(
            &mut tx,
            forum_post.forum_thread_id,
            created_forum_post.id,
            current_user_id,
        )
        .await?;

        tx.commit().await?;

        if !user_ids.is_empty() {
            let _ = notification_sender.send(NotificationEvent::ForumThreadPost { user_ids });
        }

        Ok(created_forum_post)
    }

    pub async fn find_forum_post(&self, forum_post_id: i64) -> Result<ForumPost> {
        let forum_post = sqlx::query_as!(
            ForumPost,
            r#"
                SELECT id, forum_thread_id, created_at, updated_at, created_by_id, content, sticky, locked FROM forum_posts WHERE id = $1
            "#,
            forum_post_id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumPost)?;

        Ok(forum_post)
    }

    pub async fn update_forum_post(&self, edited_post: &EditedForumPost) -> Result<ForumPost> {
        let updated_post = sqlx::query_as!(
            ForumPost,
            r#"
                UPDATE forum_posts
                SET content = $1, sticky = $2, locked = $3
                WHERE id = $4
                RETURNING id, forum_thread_id, created_at, updated_at, created_by_id, content, sticky, locked
            "#,
            edited_post.content,
            edited_post.sticky,
            edited_post.locked,
            edited_post.id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotUpdateForumPost)?;

        Ok(updated_post)
    }

    pub async fn create_forum_thread(
        &self,
        forum_thread: &mut UserCreatedForumThread,
        current_user_id: i32,
        notification_sender: &broadcast::Sender<NotificationEvent>,
    ) -> Result<ForumThread> {
        if forum_thread.name.trim().is_empty() {
            return Err(Error::ForumThreadNameEmpty);
        }

        if forum_thread.first_post.content.trim().is_empty() {
            return Err(Error::ForumPostEmpty);
        }

        let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;

        // Check if the subcategory restricts thread creation
        let is_restricted = sqlx::query_scalar!(
            r#"SELECT new_threads_restricted FROM forum_sub_categories WHERE id = $1"#,
            forum_thread.forum_sub_category_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateForumThread)?;

        if is_restricted {
            let is_allowed = sqlx::query_scalar!(
                r#"SELECT EXISTS(SELECT 1 FROM forum_sub_category_allowed_posters WHERE forum_sub_category_id = $1 AND user_id = $2) AS "exists!""#,
                forum_thread.forum_sub_category_id,
                current_user_id
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(Error::CouldNotCreateForumThread)?;

            if !is_allowed {
                return Err(Error::ForumSubCategoryNewThreadsRestricted);
            }
        }

        let created_forum_thread = sqlx::query_as!(
            ForumThread,
            r#"
                INSERT INTO forum_threads (name, created_by_id, forum_sub_category_id)
                VALUES ($1, $2, $3)
                RETURNING id, forum_sub_category_id, name, created_at, created_by_id, posts_amount, pinned, locked, views_count
            "#,
            forum_thread.name,
            current_user_id,
            forum_thread.forum_sub_category_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateForumThread)?;

        forum_thread.first_post.forum_thread_id = created_forum_thread.id;

        sqlx::query!(
            r#"
            UPDATE forum_sub_categories
            SET threads_amount = threads_amount + 1
            WHERE id = $1;
            "#,
            forum_thread.forum_sub_category_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateForumPost)?;

        sqlx::query!(
            r#"
            UPDATE users
            SET forum_threads = forum_threads + 1
            WHERE id = $1;
            "#,
            current_user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateForumThread)?;

        let sub_category_subscriber_ids = Self::notify_users_forum_sub_category_threads(
            &mut tx,
            forum_thread.forum_sub_category_id,
            created_forum_thread.id,
            current_user_id,
        )
        .await?;

        tx.commit().await?;

        if !sub_category_subscriber_ids.is_empty() {
            let _ = notification_sender.send(NotificationEvent::ForumSubCategoryThread {
                user_ids: sub_category_subscriber_ids,
            });
        }

        // Create the first post (this will increment posts_amount)
        self.create_forum_post(
            &forum_thread.first_post,
            current_user_id,
            notification_sender,
        )
        .await?;

        // Subscribe the creator to the thread
        self.create_subscription_forum_thread_posts(created_forum_thread.id, current_user_id)
            .await?;

        // Fetch and return the updated thread with correct posts_amount and subscription status
        let updated_thread = sqlx::query_as!(
            ForumThread,
            r#"SELECT id, forum_sub_category_id, name, created_at, created_by_id, posts_amount, pinned, locked, views_count FROM forum_threads WHERE id = $1"#,
            created_forum_thread.id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumThread)?;

        Ok(updated_thread)
    }

    pub async fn update_forum_thread(
        &self,
        edited_thread: &EditedForumThread,
        user_id: i32,
    ) -> Result<ForumThreadEnriched> {
        if edited_thread.name.trim().is_empty() {
            return Err(Error::ForumThreadNameEmpty);
        }

        let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;

        // Get current sub-category id
        let old_sub_category_id = sqlx::query_scalar!(
            r#"SELECT forum_sub_category_id FROM forum_threads WHERE id = $1"#,
            edited_thread.id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotFindForumThread)?;

        // If sub-category is changing, update counters on both sub-categories
        if old_sub_category_id != edited_thread.forum_sub_category_id {
            // Decrement counters on the old sub-category
            sqlx::query!(
                r#"
                UPDATE forum_sub_categories
                SET threads_amount = threads_amount - 1,
                    posts_amount = posts_amount - (SELECT posts_amount FROM forum_threads WHERE id = $2)
                WHERE id = $1
                "#,
                old_sub_category_id,
                edited_thread.id
            )
            .execute(&mut *tx)
            .await
            .map_err(Error::CouldNotUpdateForumThread)?;

            // Increment counters on the new sub-category
            sqlx::query!(
                r#"
                UPDATE forum_sub_categories
                SET threads_amount = threads_amount + 1,
                    posts_amount = posts_amount + (SELECT posts_amount FROM forum_threads WHERE id = $2)
                WHERE id = $1
                "#,
                edited_thread.forum_sub_category_id,
                edited_thread.id
            )
            .execute(&mut *tx)
            .await
            .map_err(Error::CouldNotUpdateForumThread)?;
        }

        // Update the thread
        let updated_thread = sqlx::query_as!(
            ForumThreadEnriched,
            r#"
            WITH updated_row AS (
                UPDATE forum_threads
                SET name = $1, forum_sub_category_id = $2
                WHERE id = $3
                RETURNING id, forum_sub_category_id, name, created_at, created_by_id, posts_amount, pinned, locked, views_count
            )
            SELECT
                ur.id,
                ur.forum_sub_category_id,
                ur.name,
                ur.created_at,
                ur.created_by_id,
                ur.posts_amount,
                ur.pinned,
                ur.locked,
                ur.views_count,
                fsc.name AS forum_sub_category_name,
                fc.name AS forum_category_name,
                fc.id AS forum_category_id,
                (sft.id IS NOT NULL) AS "is_subscribed!"
            FROM updated_row ur
            JOIN
                forum_sub_categories AS fsc ON ur.forum_sub_category_id = fsc.id
            JOIN
                forum_categories AS fc ON fsc.forum_category_id = fc.id
            LEFT JOIN
                subscriptions_forum_thread_posts AS sft
                ON sft.forum_thread_id = ur.id AND sft.user_id = $4
            "#,
            edited_thread.name,
            edited_thread.forum_sub_category_id,
            edited_thread.id,
            user_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotUpdateForumThread)?;

        tx.commit().await?;

        Ok(updated_thread)
    }

    pub async fn upsert_forum_thread_read(
        tx: &mut Transaction<'_, Postgres>,
        thread_id: i64,
        post_id: i64,
        user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            WITH upsert AS (
                INSERT INTO forum_thread_reads (user_id, forum_thread_id, last_read_post_id)
                VALUES ($1, $2, $3)
                ON CONFLICT (user_id, forum_thread_id)
                DO UPDATE SET last_read_post_id = GREATEST(forum_thread_reads.last_read_post_id, $3),
                              read_at = NOW()
                RETURNING (xmax = 0) AS is_insert
            )
            UPDATE forum_threads SET views_count = views_count + 1
            WHERE id = $2 AND (SELECT is_insert FROM upsert)
            "#,
            user_id,
            thread_id,
            post_id
        )
        .execute(&mut **tx)
        .await
        .map_err(Error::CouldNotUpsertForumThreadRead)?;

        Ok(())
    }

    pub async fn mark_all_announcements_as_read(&self, user_id: i32) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO forum_thread_reads (user_id, forum_thread_id, last_read_post_id)
            SELECT $1, ft.id, (
                SELECT fp.id FROM forum_posts fp
                WHERE fp.forum_thread_id = ft.id
                ORDER BY fp.id ASC
                LIMIT 1
            )
            FROM forum_threads ft
            WHERE ft.forum_sub_category_id = 1
            ON CONFLICT (user_id, forum_thread_id) DO NOTHING
            "#,
            user_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotUpsertForumThreadRead)?;

        Ok(())
    }

    pub async fn find_forum_cateogries_hierarchy(&self) -> Result<Vec<ForumCategoryHierarchy>> {
        // Query all categories at once
        let categories = sqlx::query_as!(
            ForumCategoryLite,
            "SELECT id, name FROM forum_categories ORDER BY sort_order, id"
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumSubCategory)?;

        // Query all subcategories with their latest posts in one query
        let sub_categories_data = sqlx::query_as!(
            DBImportSubCategoryWithLatestPost,
            r#"
            SELECT fsc.id, fsc.name, fsc.sort_order, fsc.threads_amount, fsc.posts_amount, fsc.forbidden_classes,
                   fsc.new_threads_restricted, fsc.thread_sort_by AS "thread_sort_by: ForumThreadSortBy",
                   fsc.thread_sort_direction AS "thread_sort_direction: ForumThreadSortDirection",
                   fsc.forum_category_id, fc.name AS category_name,
                   fp.id AS "latest_post_id?", ft.id AS "thread_id?", ft.name AS "thread_name?", fp.created_at AS "latest_post_created_at?",
                   u.id AS "user_id?", u.username AS "username?", u.warned AS "warned?", u.banned AS "banned?"
            FROM forum_sub_categories fsc
            INNER JOIN forum_categories fc ON fsc.forum_category_id = fc.id
            LEFT JOIN LATERAL (
                SELECT fp.id, fp.created_at, fp.created_by_id, fp.forum_thread_id
                FROM forum_posts fp
                JOIN forum_threads ft_inner ON fp.forum_thread_id = ft_inner.id
                WHERE ft_inner.forum_sub_category_id = fsc.id
                ORDER BY fp.created_at DESC LIMIT 1
            ) AS fp ON TRUE
            LEFT JOIN forum_threads ft ON fp.forum_thread_id = ft.id
            LEFT JOIN users u ON fp.created_by_id = u.id
            ORDER BY fsc.forum_category_id, fsc.sort_order, fsc.id
            "#
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumSubCategory)?;

        // Build hierarchy by grouping subcategories by category
        use std::collections::HashMap;
        let mut category_map: HashMap<i32, Vec<ForumSubCategoryHierarchy>> = HashMap::new();

        for sc in sub_categories_data {
            let sub_category = ForumSubCategoryHierarchy {
                id: sc.id,
                name: sc.name,
                threads_amount: sc.threads_amount,
                posts_amount: sc.posts_amount,
                forbidden_classes: sc.forbidden_classes,
                new_threads_restricted: sc.new_threads_restricted,
                thread_sort_by: sc.thread_sort_by,
                thread_sort_direction: sc.thread_sort_direction,
                // these are not needed on this endpoint, which saves us joins
                is_allowed_poster: false,
                is_subscribed: false,
                latest_post_in_thread: match (
                    sc.latest_post_id,
                    sc.thread_id,
                    sc.thread_name,
                    sc.latest_post_created_at,
                    sc.user_id,
                    sc.username,
                    sc.warned,
                    sc.banned,
                ) {
                    (
                        Some(id),
                        Some(thread_id),
                        Some(name),
                        Some(created_at),
                        Some(user_id),
                        Some(username),
                        Some(warned),
                        Some(banned),
                    ) => Some(ForumThreadPostLite {
                        id,
                        thread_id,
                        name,
                        created_at: created_at.with_timezone(&Local),
                        created_by: UserLite {
                            id: user_id,
                            username,
                            warned,
                            banned,
                        },
                    }),
                    _ => None,
                },
                threads: None,
                category: ForumCategoryLite {
                    id: sc.forum_category_id,
                    name: sc.category_name,
                },
            };
            category_map
                .entry(sc.forum_category_id)
                .or_default()
                .push(sub_category);
        }

        // Build final result with categories in order
        let forum_categories = categories
            .into_iter()
            .map(|category| ForumCategoryHierarchy {
                id: category.id,
                name: category.name,
                sub_categories: category_map.remove(&category.id).unwrap_or_default(),
            })
            .collect();

        Ok(forum_categories)
    }

    pub async fn find_forum_sub_category_threads(
        &self,
        forum_sub_category_id: i32,
        user_id: i32,
    ) -> Result<Value> {
        let forum_sub_category = sqlx::query!(
            r#"
            SELECT
                json_strip_nulls(
                    json_build_object(
                        'id', fsc.id,
                        'name', fsc.name,
                        'threads_amount', fsc.threads_amount,
                        'posts_amount', fsc.posts_amount,
                        'forbidden_classes', fsc.forbidden_classes,
                        'new_threads_restricted', fsc.new_threads_restricted,
                        'thread_sort_by', fsc.thread_sort_by,
                        'thread_sort_direction', fsc.thread_sort_direction,
                        'is_allowed_poster', (
                            NOT fsc.new_threads_restricted
                            OR EXISTS (
                                SELECT 1 FROM forum_sub_category_allowed_posters fsap
                                WHERE fsap.forum_sub_category_id = fsc.id AND fsap.user_id = $2
                            )
                        ),
                        'is_subscribed', EXISTS (
                            SELECT 1 FROM subscriptions_forum_sub_category_threads s
                            WHERE s.forum_sub_category_id = fsc.id AND s.user_id = $2
                        ),
                        'category', json_build_object(
                            'id', fc.id,
                            'name', fc.name
                        ),
                        'threads', (
                            SELECT
                                COALESCE(
                                    json_agg(
                                        json_build_object(
                                            'id', ft.id,
                                            'name', ft.name,
                                            'created_at', ft.created_at,
                                            'posts_amount', ft.posts_amount,
                                            'pinned', ft.pinned,
                                            'locked', ft.locked,
                                            'views_count', ft.views_count,
                                            'ever_opened', (ftr.last_read_post_id IS NOT NULL),
                                            'has_new_posts', COALESCE(ftr.last_read_post_id < fp_latest.id, TRUE),
                                            'created_by', json_build_object(
                                                'id', u_thread.id,
                                                'username', u_thread.username,
                                                'warned', u_thread.warned,
                                                'banned', u_thread.banned
                                            ),
                                            'latest_post', json_build_object(
                                                'id', fp_latest.id,
                                                'thread_id', ft.id,
                                                'name', ft.name,
                                                'created_at', fp_latest.created_at,
                                                'created_by', json_build_object(
                                                    'id', u_post.id,
                                                    'username', u_post.username,
                                                    'warned', u_post.warned,
                                                    'banned', u_post.banned
                                                )
                                            )
                                        ) ORDER BY
                                            ft.pinned DESC,
                                            CASE WHEN ft.pinned THEN ft.name END ASC,
                                            CASE WHEN fsc.thread_sort_by = 'created_at' AND fsc.thread_sort_direction = 'ascending' THEN ft.created_at END ASC,
                                            CASE WHEN fsc.thread_sort_by = 'created_at' AND fsc.thread_sort_direction = 'descending' THEN ft.created_at END DESC,
                                            CASE WHEN fsc.thread_sort_by = 'name' AND fsc.thread_sort_direction = 'ascending' THEN ft.name END ASC,
                                            CASE WHEN fsc.thread_sort_by = 'name' AND fsc.thread_sort_direction = 'descending' THEN ft.name END DESC,
                                            CASE WHEN fsc.thread_sort_by = 'posts_amount' AND fsc.thread_sort_direction = 'ascending' THEN ft.posts_amount END ASC,
                                            CASE WHEN fsc.thread_sort_by = 'posts_amount' AND fsc.thread_sort_direction = 'descending' THEN ft.posts_amount END DESC,
                                            CASE WHEN fsc.thread_sort_by = 'views_count' AND fsc.thread_sort_direction = 'ascending' THEN ft.views_count END ASC,
                                            CASE WHEN fsc.thread_sort_by = 'views_count' AND fsc.thread_sort_direction = 'descending' THEN ft.views_count END DESC,
                                            CASE WHEN fsc.thread_sort_by = 'latest_post' AND fsc.thread_sort_direction = 'ascending' THEN fp_latest.created_at END ASC NULLS LAST,
                                            fp_latest.created_at DESC NULLS LAST
                                    ),
                                    '[]'::json
                                )
                            FROM
                                forum_threads ft
                            JOIN
                                users u_thread ON ft.created_by_id = u_thread.id
                            LEFT JOIN LATERAL (
                                SELECT
                                    fp.id,
                                    fp.created_at,
                                    fp.created_by_id
                                FROM
                                    forum_posts fp
                                WHERE
                                    fp.forum_thread_id = ft.id
                                ORDER BY
                                    fp.created_at DESC
                                LIMIT 1
                            ) AS fp_latest ON TRUE
                            LEFT JOIN
                                users u_post ON fp_latest.created_by_id = u_post.id
                            LEFT JOIN
                                forum_thread_reads ftr ON ftr.forum_thread_id = ft.id AND ftr.user_id = $2
                            WHERE
                                ft.forum_sub_category_id = fsc.id
                        )
                    )
                ) AS result_json
            FROM
                forum_sub_categories fsc
            JOIN
                forum_categories fc ON fsc.forum_category_id = fc.id
            WHERE
                fsc.id = $1
            GROUP BY
                fsc.id, fc.id;
            "#,
            forum_sub_category_id,
            user_id
        )
        .fetch_optional(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumSubCategory)?;

        match forum_sub_category {
            Some(record) => Ok(record.result_json.unwrap_or(serde_json::json!({}))),
            None => Err(Error::CouldNotFindForumSubCategory(
                sqlx::Error::RowNotFound,
            )),
        }
    }

    pub async fn find_forum_thread(
        &self,
        forum_thread_id: i64,
        user_id: i32,
    ) -> Result<ForumThreadEnriched> {
        let forum_thread = sqlx::query_as!(
            ForumThreadEnriched,
            r#"
            SELECT
                ft.id,
                ft.forum_sub_category_id,
                ft.name,
                ft.created_at,
                ft.created_by_id,
                ft.posts_amount,
                ft.pinned,
                ft.locked,
                ft.views_count,
                fsc.name AS forum_sub_category_name,
                fc.name AS forum_category_name,
                fc.id AS forum_category_id,
                (sft.id IS NOT NULL) AS "is_subscribed!"
            FROM
                forum_threads AS ft
            JOIN
                forum_sub_categories AS fsc ON ft.forum_sub_category_id = fsc.id
            JOIN
                forum_categories AS fc ON fsc.forum_category_id = fc.id
            LEFT JOIN
                subscriptions_forum_thread_posts AS sft
                ON sft.forum_thread_id = ft.id AND sft.user_id = $2
            WHERE
                ft.id = $1;
            "#,
            forum_thread_id,
            user_id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumThread)?;

        if forum_thread.is_subscribed {
            Self::mark_notification_forum_thread_post_as_read(self, forum_thread_id, user_id)
                .await?;
        }

        self.mark_notification_forum_sub_category_thread_as_read_by_thread_id(
            forum_thread_id,
            user_id,
        )
        .await?;

        Ok(forum_thread)
    }

    pub async fn fetch_forum_poll(
        &self,
        forum_thread_id: i64,
        user_id: i32,
    ) -> Result<Option<ForumPollHierarchy>> {
        let poll = sqlx::query_as!(
            ForumPoll,
            r#"SELECT id, forum_thread_id, question, created_at, created_by_id FROM forum_polls WHERE forum_thread_id = $1"#,
            forum_thread_id
        )
        .fetch_optional(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumPoll)?;

        let Some(poll) = poll else {
            return Ok(None);
        };

        let has_voted = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM forum_poll_votes WHERE forum_poll_id = $1 AND user_id = $2) AS "exists!""#,
            poll.id,
            user_id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumPoll)?;

        let options = sqlx::query_as!(
            ForumPollOptionResult,
            r#"
            SELECT
                o.id,
                o.content,
                CASE WHEN $2 THEN COUNT(v.user_id) ELSE NULL END AS votes_amount
            FROM forum_poll_options o
            LEFT JOIN forum_poll_votes v ON v.forum_poll_option_id = o.id
            WHERE o.forum_poll_id = $1
            GROUP BY o.id, o.content, o.sort_order
            ORDER BY o.sort_order
            "#,
            poll.id,
            has_voted
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumPoll)?;

        let blank_votes_amount = sqlx::query_scalar!(
            r#"
            SELECT CASE WHEN $2 THEN COUNT(*) ELSE NULL END AS blank_votes_amount
            FROM forum_poll_votes
            WHERE forum_poll_id = $1 AND forum_poll_option_id IS NULL
            "#,
            poll.id,
            has_voted
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumPoll)?;

        Ok(Some(ForumPollHierarchy {
            id: poll.id,
            question: poll.question,
            created_at: poll.created_at,
            created_by_id: poll.created_by_id,
            has_voted,
            options,
            blank_votes_amount,
        }))
    }

    pub async fn find_forum_thread_with_poll(
        &self,
        forum_thread_id: i64,
        user_id: i32,
    ) -> Result<ForumThreadEnrichedHierarchy> {
        let thread = self.find_forum_thread(forum_thread_id, user_id).await?;
        let poll = self.fetch_forum_poll(forum_thread_id, user_id).await?;

        Ok(ForumThreadEnrichedHierarchy { thread, poll })
    }

    pub async fn create_forum_poll(
        &self,
        poll: &UserCreatedForumPoll,
        current_user_id: i32,
    ) -> Result<ForumPoll> {
        if poll.question.trim().is_empty() {
            return Err(Error::ForumPollQuestionEmpty);
        }

        let options: Vec<String> = poll
            .options
            .iter()
            .map(|option| option.trim().to_string())
            .filter(|option| !option.is_empty())
            .collect();

        if options.is_empty() {
            return Err(Error::ForumPollOptionsInsufficient);
        }

        let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;

        let thread_creator_id = sqlx::query_scalar!(
            r#"SELECT created_by_id FROM forum_threads WHERE id = $1"#,
            poll.forum_thread_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotFindForumThread)?;

        if thread_creator_id != current_user_id {
            return Err(Error::InsufficientPermissions(
                "only the thread author can attach a poll to it".to_string(),
            ));
        }

        let created_poll = sqlx::query_as!(
            ForumPoll,
            r#"
                INSERT INTO forum_polls (forum_thread_id, question, created_by_id)
                VALUES ($1, $2, $3)
                RETURNING id, forum_thread_id, question, created_at, created_by_id
            "#,
            poll.forum_thread_id,
            poll.question,
            current_user_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e
                && db_err.code().as_deref() == Some("23505")
            {
                return Error::ForumThreadAlreadyHasPoll;
            }
            Error::CouldNotCreateForumPoll(e)
        })?;

        sqlx::query!(
            r#"
            INSERT INTO forum_poll_options (forum_poll_id, content, sort_order)
            SELECT $1, content, ordinality::int
            FROM UNNEST($2::text[]) WITH ORDINALITY AS t(content, ordinality)
            "#,
            created_poll.id,
            &options
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateForumPoll)?;

        tx.commit().await?;

        Ok(created_poll)
    }

    pub async fn create_forum_poll_vote(
        &self,
        vote: &UserCreatedForumPollVote,
        user_id: i32,
    ) -> Result<ForumPollHierarchy> {
        let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;

        let forum_thread_id = sqlx::query_scalar!(
            r#"SELECT forum_thread_id FROM forum_polls WHERE id = $1"#,
            vote.forum_poll_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotFindForumPoll)?;

        if let Some(option_id) = vote.forum_poll_option_id {
            let option_belongs_to_poll = sqlx::query_scalar!(
                r#"SELECT EXISTS(SELECT 1 FROM forum_poll_options WHERE id = $1 AND forum_poll_id = $2) AS "exists!""#,
                option_id,
                vote.forum_poll_id
            )
            .fetch_one(&mut *tx)
            .await
            .map_err(Error::CouldNotCreateForumPollVote)?;

            if !option_belongs_to_poll {
                return Err(Error::ForumPollOptionNotFound);
            }
        }

        sqlx::query!(
            r#"
            INSERT INTO forum_poll_votes (forum_poll_id, forum_poll_option_id, user_id)
            VALUES ($1, $2, $3)
            "#,
            vote.forum_poll_id,
            vote.forum_poll_option_id,
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e
                && db_err.code().as_deref() == Some("23505")
            {
                return Error::ForumPollAlreadyVoted;
            }
            Error::CouldNotCreateForumPollVote(e)
        })?;

        tx.commit().await?;

        self.fetch_forum_poll(forum_thread_id, user_id)
            .await?
            .ok_or(Error::CouldNotFindForumPoll(sqlx::Error::RowNotFound))
    }

    pub async fn find_forum_thread_posts(
        &self,
        form: GetForumThreadPostsQuery,
        user_id: i32,
    ) -> Result<PaginatedResults<ForumPostHierarchy>> {
        let page_size = form.page_size as i64;
        let mut current_page = form.page.unwrap_or(1);

        let offset = if let Some(post_id) = form.post_id {
            let position = sqlx::query_scalar!(
                r#"
                SELECT COUNT(*)::BIGINT FROM forum_posts
                WHERE forum_thread_id = $1 AND id < $2
                "#,
                form.thread_id,
                post_id
            )
            .fetch_one(self.borrow())
            .await?
            .unwrap_or(0);

            // i64 ceil division is unstable as of now
            current_page = ((position + 1) as u64).div_ceil(form.page_size as u64) as u32;
            ((position / page_size) * page_size) as i64
        } else {
            ((form.page.unwrap_or(1) - 1) as i64) * page_size
        };

        let posts = sqlx::query_as!(
            DBImportForumPost,
            r#"
            SELECT
                fp.id,
                fp.content,
                fp.created_at,
                fp.updated_at,
                fp.sticky,
                fp.locked,
                fp.forum_thread_id,
                u.id AS created_by_user_id,
                u.username AS created_by_user_username,
                u.class_name AS created_by_user_class_name,
                u.avatar AS created_by_user_avatar,
                u.banned AS created_by_user_banned,
                u.warned AS created_by_user_warned,
                u.custom_title AS created_by_user_custom_title
            FROM forum_posts fp
            JOIN users u ON fp.created_by_id = u.id
            WHERE fp.forum_thread_id = $1
            ORDER BY fp.created_at ASC
            OFFSET $2
            LIMIT $3
            "#,
            form.thread_id,
            offset,
            page_size
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumThread)?;

        let total_forum_posts_in_thread = sqlx::query_scalar!(
            r#"SELECT COUNT(id) FROM forum_posts WHERE forum_thread_id = $1"#,
            form.thread_id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumThread)?
        .unwrap_or(0);

        // Track the last post ID on this page for read markers
        if let Some(last_post) = posts.last() {
            let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
                .begin()
                .await?;
            Self::upsert_forum_thread_read(&mut tx, form.thread_id, last_post.id, user_id).await?;
            tx.commit().await?;
        }

        let forum_post_ids: Vec<i64> = posts.iter().map(|post| post.id).collect();
        let mut reactions_per_post = self
            .find_reactions_for_forum_posts(&forum_post_ids, user_id)
            .await?;

        let forum_posts: Vec<ForumPostHierarchy> = posts
            .into_iter()
            .map(|r| ForumPostHierarchy {
                id: r.id,
                content: r.content,
                created_at: r.created_at,
                updated_at: r.updated_at,
                sticky: r.sticky,
                locked: r.locked,
                forum_thread_id: r.forum_thread_id,
                created_by: UserLiteAvatar {
                    id: r.created_by_user_id,
                    username: r.created_by_user_username,
                    class_name: r.created_by_user_class_name,
                    avatar: r.created_by_user_avatar,
                    banned: r.created_by_user_banned,
                    warned: r.created_by_user_warned,
                    custom_title: r.created_by_user_custom_title,
                },
                reactions: reactions_per_post.remove(&r.id).unwrap_or_default(),
            })
            .collect();

        let paginated_results = PaginatedResults {
            results: forum_posts,
            page: current_page,
            page_size: form.page_size,
            total_items: total_forum_posts_in_thread,
        };

        Ok(paginated_results)
    }

    pub async fn find_first_thread_posts_in_sub_category(
        &self,
        forum_sub_category_id: i32,
        limit: u32,
    ) -> Result<Vec<ForumPostAndThreadName>> {
        sqlx::query_as!(
            ForumPostAndThreadName,
            r#"
            SELECT DISTINCT ON (ft.id)
                fp.id,
                fp.forum_thread_id,
                fp.created_at as "created_at!",
                fp.updated_at as "updated_at!",
                fp.created_by_id,
                fp.content,
                fp.sticky,
                ft.name as "forum_thread_name"
            FROM
                forum_threads AS ft
            JOIN
                forum_posts AS fp ON ft.id = fp.forum_thread_id
            WHERE
                ft.forum_sub_category_id = $1
            ORDER BY
                ft.id DESC, fp.created_at ASC
            LIMIT $2
            "#,
            forum_sub_category_id,
            limit as i32
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumThreadsFirstPost)
    }

    pub async fn search_forum_threads(
        &self,
        form: &ForumSearchQuery,
    ) -> Result<PaginatedResults<ForumSearchResult>> {
        let limit = form.page_size as i64;
        let offset = (form.page - 1) as i64 * form.page_size as i64;

        let results = sqlx::query_as!(
            ForumSearchResult,
            r#"
            SELECT
                t.name AS thread_name,
                t.id AS thread_id,
                p.content AS post,
                p.id AS post_id,
                p.created_at AS post_created_at,
                p.created_by_id AS post_created_by_id,
                u.username AS post_created_by_username,
                s.name AS sub_category_name,
                s.id AS sub_category_id,
                c.name AS category_name,
                c.id AS category_id
            FROM forum_threads t
            JOIN LATERAL (
                SELECT p.*
                FROM forum_posts p
                WHERE p.forum_thread_id = t.id
                ORDER BY p.created_at DESC
                LIMIT 1
            ) p ON TRUE
            JOIN users u ON u.id = p.created_by_id
            JOIN forum_sub_categories s ON s.id = t.forum_sub_category_id
            JOIN forum_categories c ON c.id = s.forum_category_id

            WHERE $1::TEXT IS NULL OR t.name ILIKE '%' || $1 || '%'

            ORDER BY p.created_at DESC

            LIMIT $2 OFFSET $3;
            "#,
            form.thread_name,
            limit,
            offset
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumThreadsFirstPost)?;

        let total_results = sqlx::query!(
            "SELECT COUNT(*) AS total FROM forum_threads WHERE name ILIKE '%' || $1 || '%'",
            form.thread_name
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotSearchForumThreads)?
        .total
        .unwrap_or(0);

        Ok(PaginatedResults {
            results,
            total_items: total_results,
            page: form.page,
            page_size: form.page_size,
        })
    }

    /// Every forum post written by a user, most recent first.
    pub async fn search_forum_posts(
        &self,
        form: &ForumPostSearchQuery,
    ) -> Result<PaginatedResults<ForumPostWithLocation>> {
        let limit = form.page_size as i64;
        let offset = (form.page - 1) as i64 * form.page_size as i64;

        let posts = sqlx::query_as!(
            DBImportForumPostWithLocation,
            r#"
            SELECT
                p.id,
                p.content,
                p.created_at,
                p.updated_at,
                p.sticky,
                p.locked,
                p.forum_thread_id,
                u.id AS created_by_user_id,
                u.username AS created_by_user_username,
                u.class_name AS created_by_user_class_name,
                u.avatar AS created_by_user_avatar,
                u.banned AS created_by_user_banned,
                u.warned AS created_by_user_warned,
                u.custom_title AS created_by_user_custom_title,
                t.name AS thread_name,
                s.id AS sub_category_id,
                s.name AS sub_category_name,
                c.id AS category_id,
                c.name AS category_name
            FROM forum_posts p
            JOIN users u ON u.id = p.created_by_id
            JOIN forum_threads t ON t.id = p.forum_thread_id
            JOIN forum_sub_categories s ON s.id = t.forum_sub_category_id
            JOIN forum_categories c ON c.id = s.forum_category_id

            WHERE p.created_by_id = $1

            ORDER BY p.created_at DESC

            LIMIT $2 OFFSET $3;
            "#,
            form.created_by_id,
            limit,
            offset
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotSearchForumPosts)?;

        let results = posts
            .into_iter()
            .map(|post| ForumPostWithLocation {
                post: ForumPostHierarchy {
                    id: post.id,
                    forum_thread_id: post.forum_thread_id,
                    created_at: post.created_at,
                    updated_at: post.updated_at,
                    content: post.content,
                    sticky: post.sticky,
                    locked: post.locked,
                    created_by: UserLiteAvatar {
                        id: post.created_by_user_id,
                        username: post.created_by_user_username,
                        class_name: post.created_by_user_class_name,
                        avatar: post.created_by_user_avatar,
                        banned: post.created_by_user_banned,
                        warned: post.created_by_user_warned,
                        custom_title: post.created_by_user_custom_title,
                    },
                    reactions: Vec::new(),
                },
                thread_name: post.thread_name,
                sub_category_id: post.sub_category_id,
                sub_category_name: post.sub_category_name,
                category_id: post.category_id,
                category_name: post.category_name,
            })
            .collect();

        let total_results = sqlx::query!(
            "SELECT COUNT(*) AS total FROM forum_posts WHERE created_by_id = $1",
            form.created_by_id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotSearchForumPosts)?
        .total
        .unwrap_or(0);

        Ok(PaginatedResults {
            results,
            total_items: total_results,
            page: form.page,
            page_size: form.page_size,
        })
    }

    pub async fn find_forum_category(&self, category_id: i32) -> Result<ForumCategory> {
        sqlx::query_as!(
            ForumCategory,
            r#"SELECT id, name, sort_order, created_at, created_by_id FROM forum_categories WHERE id = $1"#,
            category_id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(|_| Error::ForumCategoryNotFound)
    }

    pub async fn find_forum_sub_category_raw(
        &self,
        sub_category_id: i32,
    ) -> Result<ForumSubCategory> {
        sqlx::query_as!(
            ForumSubCategory,
            r#"
            SELECT
                fsc.id,
                fsc.forum_category_id,
                fsc.name,
                fsc.sort_order,
                fsc.created_at,
                fsc.created_by_id,
                fsc.forbidden_classes,
                fsc.new_threads_restricted,
                fsc.thread_sort_by AS "thread_sort_by: ForumThreadSortBy",
                fsc.thread_sort_direction AS "thread_sort_direction: ForumThreadSortDirection",
                (SELECT COUNT(*) FROM forum_threads ft WHERE ft.forum_sub_category_id = fsc.id) AS "threads_amount!",
                (SELECT COUNT(*) FROM forum_posts fp JOIN forum_threads ft ON fp.forum_thread_id = ft.id WHERE ft.forum_sub_category_id = fsc.id) AS "posts_amount!"
            FROM forum_sub_categories fsc
            WHERE fsc.id = $1
            "#,
            sub_category_id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumSubCategory)
    }

    pub async fn create_forum_category(
        &self,
        forum_category: &UserCreatedForumCategory,
        current_user_id: i32,
    ) -> Result<ForumCategory> {
        if forum_category.name.trim().is_empty() {
            return Err(Error::ForumCategoryNameEmpty);
        }

        let created_category = sqlx::query_as!(
            ForumCategory,
            r#"
                INSERT INTO forum_categories (name, created_by_id, sort_order)
                VALUES ($1, $2, COALESCE((SELECT MAX(sort_order) FROM forum_categories), 0) + 1)
                RETURNING id, name, sort_order, created_at, created_by_id
            "#,
            forum_category.name,
            current_user_id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotCreateForumCategory)?;

        Ok(created_category)
    }

    pub async fn update_forum_category(
        &self,
        edited_category: &EditedForumCategory,
    ) -> Result<ForumCategory> {
        if edited_category.name.trim().is_empty() {
            return Err(Error::ForumCategoryNameEmpty);
        }

        let updated_category = sqlx::query_as!(
            ForumCategory,
            r#"
                UPDATE forum_categories
                SET name = $1
                WHERE id = $2
                RETURNING id, name, sort_order, created_at, created_by_id
            "#,
            edited_category.name,
            edited_category.id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::ForumCategoryNotFound,
            _ => Error::CouldNotUpdateForumCategory(e),
        })?;

        Ok(updated_category)
    }

    pub async fn create_forum_sub_category(
        &self,
        forum_sub_category: &UserCreatedForumSubCategory,
        current_user_id: i32,
    ) -> Result<ForumSubCategory> {
        if forum_sub_category.name.trim().is_empty() {
            return Err(Error::ForumSubCategoryNameEmpty);
        }

        let created_sub_category = sqlx::query_as!(
            ForumSubCategory,
            r#"
                INSERT INTO forum_sub_categories (name, forum_category_id, created_by_id, new_threads_restricted, sort_order)
                VALUES ($1, $2, $3, $4, COALESCE((SELECT MAX(sort_order) FROM forum_sub_categories WHERE forum_category_id = $2), 0) + 1)
                RETURNING id, forum_category_id, name, sort_order, created_at, created_by_id, threads_amount, posts_amount, forbidden_classes, new_threads_restricted, thread_sort_by AS "thread_sort_by: ForumThreadSortBy", thread_sort_direction AS "thread_sort_direction: ForumThreadSortDirection"
            "#,
            forum_sub_category.name,
            forum_sub_category.forum_category_id,
            current_user_id,
            forum_sub_category.new_threads_restricted
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotCreateForumSubCategory)?;

        Ok(created_sub_category)
    }

    pub async fn update_forum_sub_category(
        &self,
        edited_sub_category: &EditedForumSubCategory,
    ) -> Result<ForumSubCategory> {
        if edited_sub_category.name.trim().is_empty() {
            return Err(Error::ForumSubCategoryNameEmpty);
        }

        let updated_sub_category = sqlx::query_as!(
            ForumSubCategory,
            r#"
                UPDATE forum_sub_categories
                SET name = $1, new_threads_restricted = $2, thread_sort_by = $3, thread_sort_direction = $4
                WHERE id = $5
                RETURNING id, forum_category_id, name, sort_order, created_at, created_by_id, threads_amount, posts_amount, forbidden_classes, new_threads_restricted, thread_sort_by AS "thread_sort_by: ForumThreadSortBy", thread_sort_direction AS "thread_sort_direction: ForumThreadSortDirection"
            "#,
            edited_sub_category.name,
            edited_sub_category.new_threads_restricted,
            edited_sub_category.thread_sort_by as ForumThreadSortBy,
            edited_sub_category.thread_sort_direction as ForumThreadSortDirection,
            edited_sub_category.id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::ForumSubCategoryNotFound,
            _ => Error::CouldNotUpdateForumSubCategory(e),
        })?;

        Ok(updated_sub_category)
    }

    pub async fn delete_forum_category(&self, category_id: i32) -> Result<()> {
        // Check if category has any sub-categories
        let sub_category_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM forum_sub_categories WHERE forum_category_id = $1"#,
            category_id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotDeleteForumCategory)?
        .unwrap_or(0);

        if sub_category_count > 0 {
            return Err(Error::ForumCategoryHasSubCategories);
        }

        // Delete the category
        let result = sqlx::query!(r#"DELETE FROM forum_categories WHERE id = $1"#, category_id)
            .execute(self.borrow())
            .await
            .map_err(Error::CouldNotDeleteForumCategory)?;

        if result.rows_affected() == 0 {
            return Err(Error::ForumCategoryNotFound);
        }

        Ok(())
    }

    pub async fn delete_forum_sub_category(&self, sub_category_id: i32) -> Result<()> {
        // Check if sub-category has any threads
        let thread_count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM forum_threads WHERE forum_sub_category_id = $1"#,
            sub_category_id
        )
        .fetch_one(self.borrow())
        .await
        .map_err(Error::CouldNotDeleteForumSubCategory)?
        .unwrap_or(0);

        if thread_count > 0 {
            return Err(Error::ForumSubCategoryHasThreads);
        }

        // Delete the sub-category
        let result = sqlx::query!(
            r#"DELETE FROM forum_sub_categories WHERE id = $1"#,
            sub_category_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotDeleteForumSubCategory)?;

        if result.rows_affected() == 0 {
            return Err(Error::ForumSubCategoryNotFound);
        }

        Ok(())
    }

    pub async fn delete_forum_thread(&self, thread_id: i64) -> Result<()> {
        let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;

        // Get thread info (sub_category_id, posts_amount, created_by_id) before deletion
        let thread_info = sqlx::query!(
            r#"
            SELECT forum_sub_category_id, posts_amount, created_by_id
            FROM forum_threads
            WHERE id = $1
            "#,
            thread_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::CouldNotFindForumThread(e),
            _ => Error::CouldNotDeleteForumThread(e),
        })?;

        // Delete all posts in the thread and decrement their authors' forum_posts counters
        sqlx::query!(
            r#"
            WITH deleted AS (
                DELETE FROM forum_posts WHERE forum_thread_id = $1 RETURNING created_by_id
            )
            UPDATE users
            SET forum_posts = forum_posts - d.amount
            FROM (SELECT created_by_id, COUNT(*) AS amount FROM deleted GROUP BY created_by_id) d
            WHERE users.id = d.created_by_id
            "#,
            thread_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotDeleteForumThread)?;

        // Delete the thread
        let result = sqlx::query!(r#"DELETE FROM forum_threads WHERE id = $1"#, thread_id)
            .execute(&mut *tx)
            .await
            .map_err(Error::CouldNotDeleteForumThread)?;

        if result.rows_affected() == 0 {
            return Err(Error::CouldNotFindForumThread(sqlx::Error::RowNotFound));
        }

        // Update sub-category counters
        sqlx::query!(
            r#"
            UPDATE forum_sub_categories
            SET threads_amount = threads_amount - 1,
                posts_amount = posts_amount - $2
            WHERE id = $1
            "#,
            thread_info.forum_sub_category_id,
            thread_info.posts_amount
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotDeleteForumThread)?;

        // Decrement user's forum_threads counter
        sqlx::query!(
            r#"
            UPDATE users
            SET forum_threads = forum_threads - 1
            WHERE id = $1;
            "#,
            thread_info.created_by_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotDeleteForumThread)?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn delete_forum_post(&self, post_id: i64) -> Result<()> {
        let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;

        // Get post info (thread_id and created_by_id) before deletion
        let post_info = sqlx::query!(
            r#"SELECT forum_thread_id, created_by_id FROM forum_posts WHERE id = $1"#,
            post_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::CouldNotFindForumPost(e),
            _ => Error::CouldNotDeleteForumPost(e),
        })?;

        // Delete the post
        let result = sqlx::query!(r#"DELETE FROM forum_posts WHERE id = $1"#, post_id)
            .execute(&mut *tx)
            .await
            .map_err(Error::CouldNotDeleteForumPost)?;

        if result.rows_affected() == 0 {
            return Err(Error::CouldNotFindForumPost(sqlx::Error::RowNotFound));
        }

        // Decrement thread post counter
        sqlx::query!(
            r#"
            UPDATE forum_threads
            SET posts_amount = posts_amount - 1
            WHERE id = $1
            "#,
            post_info.forum_thread_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotDeleteForumPost)?;

        // Decrement sub-category post counter
        sqlx::query!(
            r#"
            UPDATE forum_sub_categories
            SET posts_amount = posts_amount - 1
            WHERE id = (SELECT forum_sub_category_id FROM forum_threads WHERE id = $1)
            "#,
            post_info.forum_thread_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotDeleteForumPost)?;

        // Decrement user's forum_posts counter
        sqlx::query!(
            r#"
            UPDATE users
            SET forum_posts = forum_posts - 1
            WHERE id = $1;
            "#,
            post_info.created_by_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotDeleteForumPost)?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn pin_forum_thread(&self, form: &PinForumThread) -> Result<()> {
        sqlx::query!(
            r#"
                UPDATE forum_threads SET pinned = $1 WHERE id = $2
            "#,
            form.pin,
            form.thread_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotPinForumThread)?;

        Ok(())
    }

    pub async fn add_forum_sub_category_allowed_poster(
        &self,
        forum_sub_category_id: i32,
        user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                INSERT INTO forum_sub_category_allowed_posters (forum_sub_category_id, user_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
            "#,
            forum_sub_category_id,
            user_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotUpdateForumSubCategory)?;

        Ok(())
    }

    pub async fn remove_forum_sub_category_allowed_poster(
        &self,
        forum_sub_category_id: i32,
        user_id: i32,
    ) -> Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM forum_sub_category_allowed_posters
                WHERE forum_sub_category_id = $1 AND user_id = $2
            "#,
            forum_sub_category_id,
            user_id
        )
        .execute(self.borrow())
        .await
        .map_err(Error::CouldNotUpdateForumSubCategory)?;

        Ok(())
    }

    pub async fn get_forum_sub_category_allowed_posters(
        &self,
        forum_sub_category_id: i32,
    ) -> Result<Vec<UserLite>> {
        let users = sqlx::query_as!(
            UserLite,
            r#"
                SELECT u.id, u.username, u.warned, u.banned
                FROM forum_sub_category_allowed_posters fsap
                JOIN users u ON fsap.user_id = u.id
                WHERE fsap.forum_sub_category_id = $1
                ORDER BY u.username
            "#,
            forum_sub_category_id
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotFindForumSubCategory)?;

        Ok(users)
    }

    /// Reorders forum categories. Runs inside a transaction so a body naming an unknown
    /// category id rolls back the whole bulk update instead of leaving the known ids reordered
    /// while the caller is told the request failed.
    pub async fn reorder_forum_categories(&self, reorder: &ReorderForumCategories) -> Result<()> {
        let ids: Vec<i32> = reorder.categories.iter().map(|entry| entry.id).collect();
        let sort_orders: Vec<i32> = reorder
            .categories
            .iter()
            .map(|entry| entry.sort_order)
            .collect();

        let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;

        let result = sqlx::query!(
            r#"
                UPDATE forum_categories
                SET sort_order = new_values.sort_order
                FROM UNNEST($1::int[], $2::int[]) AS new_values(id, sort_order)
                WHERE forum_categories.id = new_values.id
            "#,
            &ids,
            &sort_orders
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotReorderForumCategory)?;

        if result.rows_affected() != ids.len() as u64 {
            // Dropping the transaction without committing rolls back the partial update.
            return Err(Error::ForumCategoryNotFound);
        }

        tx.commit().await?;

        Ok(())
    }

    /// Reorders forum sub categories. Runs inside a transaction so a body naming an unknown
    /// sub category id rolls back the whole bulk update instead of leaving the known ids
    /// reordered while the caller is told the request failed.
    pub async fn reorder_forum_sub_categories(
        &self,
        reorder: &ReorderForumSubCategories,
    ) -> Result<()> {
        let ids: Vec<i32> = reorder
            .sub_categories
            .iter()
            .map(|entry| entry.id)
            .collect();
        let sort_orders: Vec<i32> = reorder
            .sub_categories
            .iter()
            .map(|entry| entry.sort_order)
            .collect();

        let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;

        let result = sqlx::query!(
            r#"
                UPDATE forum_sub_categories
                SET sort_order = new_values.sort_order
                FROM UNNEST($1::int[], $2::int[]) AS new_values(id, sort_order)
                WHERE forum_sub_categories.id = new_values.id
                    AND forum_sub_categories.forum_category_id = $3
            "#,
            &ids,
            &sort_orders,
            reorder.forum_category_id
        )
        .execute(&mut *tx)
        .await
        .map_err(Error::CouldNotReorderForumSubCategory)?;

        if result.rows_affected() != ids.len() as u64 {
            // Dropping the transaction without committing rolls back the partial update.
            return Err(Error::ForumSubCategoryNotFound);
        }

        tx.commit().await?;

        Ok(())
    }

    pub async fn insert_related_forum_thread_tx(
        tx: &mut Transaction<'_, Postgres>,
        item_type: SiteHighlightItemType,
        item_id: i64,
        forum_thread_id: i64,
        user_id: i32,
    ) -> Result<()> {
        match item_type {
            SiteHighlightItemType::TitleGroup => {
                sqlx::query!(
                    r#"
                        INSERT INTO title_group_related_threads (title_group_id, forum_thread_id, created_by_id)
                        VALUES ($1, $2, $3)
                        ON CONFLICT (title_group_id, forum_thread_id) DO NOTHING
                    "#,
                    item_id as i32,
                    forum_thread_id,
                    user_id,
                )
                .execute(&mut **tx)
                .await
                .map_err(Error::CouldNotCreateRelatedThread)?;
            }
            SiteHighlightItemType::Series => {
                sqlx::query!(
                    r#"
                        INSERT INTO series_related_threads (series_id, forum_thread_id, created_by_id)
                        VALUES ($1, $2, $3)
                        ON CONFLICT (series_id, forum_thread_id) DO NOTHING
                    "#,
                    item_id,
                    forum_thread_id,
                    user_id,
                )
                .execute(&mut **tx)
                .await
                .map_err(Error::CouldNotCreateRelatedThread)?;
            }
            SiteHighlightItemType::Artist => {
                sqlx::query!(
                    r#"
                        INSERT INTO artist_related_threads (artist_id, forum_thread_id, created_by_id)
                        VALUES ($1, $2, $3)
                        ON CONFLICT (artist_id, forum_thread_id) DO NOTHING
                    "#,
                    item_id,
                    forum_thread_id,
                    user_id,
                )
                .execute(&mut **tx)
                .await
                .map_err(Error::CouldNotCreateRelatedThread)?;
            }
        }
        Ok(())
    }

    pub async fn delete_related_forum_thread_tx(
        tx: &mut Transaction<'_, Postgres>,
        item_type: SiteHighlightItemType,
        item_id: i64,
        forum_thread_id: i64,
    ) -> Result<u64> {
        let rows_affected = match item_type {
            SiteHighlightItemType::TitleGroup => sqlx::query!(
                r#"
                        DELETE FROM title_group_related_threads
                        WHERE title_group_id = $1 AND forum_thread_id = $2
                    "#,
                item_id as i32,
                forum_thread_id,
            )
            .execute(&mut **tx)
            .await
            .map_err(Error::CouldNotDeleteRelatedThread)?
            .rows_affected(),
            SiteHighlightItemType::Series => sqlx::query!(
                r#"
                        DELETE FROM series_related_threads
                        WHERE series_id = $1 AND forum_thread_id = $2
                    "#,
                item_id,
                forum_thread_id,
            )
            .execute(&mut **tx)
            .await
            .map_err(Error::CouldNotDeleteRelatedThread)?
            .rows_affected(),
            SiteHighlightItemType::Artist => sqlx::query!(
                r#"
                        DELETE FROM artist_related_threads
                        WHERE artist_id = $1 AND forum_thread_id = $2
                    "#,
                item_id,
                forum_thread_id,
            )
            .execute(&mut **tx)
            .await
            .map_err(Error::CouldNotDeleteRelatedThread)?
            .rows_affected(),
        };
        Ok(rows_affected)
    }

    pub async fn create_related_forum_thread(
        &self,
        payload: &CreateRelatedForumThread,
        user_id: i32,
    ) -> Result<()> {
        let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;
        Self::insert_related_forum_thread_tx(
            &mut tx,
            payload.item_type,
            payload.item_id,
            payload.forum_thread_id,
            user_id,
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_related_forum_thread(
        &self,
        query: &DeleteRelatedForumThreadQuery,
    ) -> Result<()> {
        let mut tx = <ConnectionPool as Borrow<PgPool>>::borrow(self)
            .begin()
            .await?;
        let rows_affected = Self::delete_related_forum_thread_tx(
            &mut tx,
            query.item_type,
            query.item_id,
            query.forum_thread_id,
        )
        .await?;
        if rows_affected == 0 {
            return Err(Error::RelatedForumThreadNotFound);
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn find_related_forum_threads_for_title_group(
        &self,
        title_group_id: i32,
    ) -> Result<Vec<RelatedForumThread>> {
        sqlx::query_as!(
            RelatedForumThread,
            r#"
                SELECT
                    rt.forum_thread_id,
                    ft.name AS thread_name,
                    rt.created_at
                FROM title_group_related_threads rt
                JOIN forum_threads ft ON ft.id = rt.forum_thread_id
                WHERE rt.title_group_id = $1
                ORDER BY rt.created_at DESC
            "#,
            title_group_id
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetRelatedThreads)
    }

    pub async fn find_related_forum_threads_for_series(
        &self,
        series_id: i64,
    ) -> Result<Vec<RelatedForumThread>> {
        sqlx::query_as!(
            RelatedForumThread,
            r#"
                SELECT
                    rt.forum_thread_id,
                    ft.name AS thread_name,
                    rt.created_at
                FROM series_related_threads rt
                JOIN forum_threads ft ON ft.id = rt.forum_thread_id
                WHERE rt.series_id = $1
                ORDER BY rt.created_at DESC
            "#,
            series_id
        )
        .fetch_all(self.borrow())
        .await
        .map_err(Error::CouldNotGetRelatedThreads)
    }
}
