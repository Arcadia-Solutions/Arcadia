use crate::{
    connection_pool::ConnectionPool,
    models::{
        forum::{
            EditedUserClassForbiddenForumItems, ForumCategoryLite, ForumItemClassVisibility,
            ForumItemId, ForumSubCategoryLite, UserClassForbiddenForumItems,
        },
        user::UserLite,
    },
};
use arcadia_common::error::{Error, Result};
use sqlx::PgConnection;

impl ConnectionPool {
    /// Fails with the item's not found error when it doesn't exist or is hidden from the user,
    /// so that its existence doesn't leak
    pub async fn ensure_forum_item_visible(&self, user_id: i32, item: ForumItemId) -> Result<()> {
        let mut connection = self.pool.acquire().await?;
        if is_forum_item_visible(&mut connection, user_id, item).await? {
            Ok(())
        } else {
            Err(forum_item_not_found_error(item))
        }
    }

    /// Same as `ensure_forum_item_visible`, for several items checked in at most one query per
    /// item kind
    pub async fn ensure_forum_items_visible(
        &self,
        user_id: i32,
        category_ids: &[i32],
        sub_category_ids: &[i32],
    ) -> Result<()> {
        let mut connection = self.pool.acquire().await?;
        match first_hidden_forum_item(&mut connection, user_id, category_ids, sub_category_ids)
            .await?
        {
            Some(hidden_item) => Err(forum_item_not_found_error(hidden_item)),
            None => Ok(()),
        }
    }

    pub async fn ensure_forum_thread_visible(
        &self,
        user_id: i32,
        forum_thread_id: i64,
    ) -> Result<()> {
        let mut connection = self.pool.acquire().await?;
        ensure_forum_thread_visible_with(&mut connection, user_id, forum_thread_id).await
    }

    pub async fn ensure_forum_post_visible(&self, user_id: i32, forum_post_id: i64) -> Result<()> {
        let visible = sqlx::query_scalar!(
            r#"SELECT forum_thread_visible($1, forum_thread_id) FROM forum_posts WHERE id = $2"#,
            user_id,
            forum_post_id
        )
        .fetch_optional(&self.pool)
        .await?
        .flatten()
        .unwrap_or(false);

        if visible {
            Ok(())
        } else {
            Err(Error::CouldNotFindForumPost(sqlx::Error::RowNotFound))
        }
    }

    pub async fn find_forum_item_allowed_users(&self, item: ForumItemId) -> Result<Vec<UserLite>> {
        let users = match item {
            ForumItemId::Category(id) => {
                sqlx::query_as!(
                    UserLite,
                    r#"
                    SELECT u.id, u.username, u.warned, u.banned
                    FROM forum_category_allowed_users allowed
                    JOIN users u ON u.id = allowed.user_id
                    WHERE allowed.forum_category_id = $1
                    ORDER BY u.id
                    "#,
                    id
                )
                .fetch_all(&self.pool)
                .await?
            }
            ForumItemId::SubCategory(id) => {
                sqlx::query_as!(
                    UserLite,
                    r#"
                    SELECT u.id, u.username, u.warned, u.banned
                    FROM forum_sub_category_allowed_users allowed
                    JOIN users u ON u.id = allowed.user_id
                    WHERE allowed.forum_sub_category_id = $1
                    ORDER BY u.id
                    "#,
                    id
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(users)
    }

    pub async fn find_forum_item_class_visibility(
        &self,
        item: ForumItemId,
    ) -> Result<ForumItemClassVisibility> {
        let classes = match item {
            ForumItemId::Category(id) => {
                sqlx::query!(
                    r#"
                    SELECT uc.name, EXISTS (
                        SELECT 1 FROM forum_category_forbidden_classes forbidden
                        WHERE forbidden.forum_category_id = $1 AND forbidden.user_class_name = uc.name
                    ) AS "hidden!"
                    FROM user_classes uc
                    ORDER BY uc.name
                    "#,
                    id
                )
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|class| (class.name, class.hidden))
                .collect::<Vec<_>>()
            }
            ForumItemId::SubCategory(id) => {
                sqlx::query!(
                    r#"
                    SELECT uc.name, (
                        EXISTS (
                            SELECT 1 FROM forum_sub_category_forbidden_classes forbidden
                            WHERE forbidden.forum_sub_category_id = $1 AND forbidden.user_class_name = uc.name
                        )
                        OR EXISTS (
                            SELECT 1 FROM forum_sub_categories fsc
                            JOIN forum_category_forbidden_classes forbidden ON forbidden.forum_category_id = fsc.forum_category_id
                            WHERE fsc.id = $1 AND forbidden.user_class_name = uc.name
                        )
                    ) AS "hidden!"
                    FROM user_classes uc
                    ORDER BY uc.name
                    "#,
                    id
                )
                .fetch_all(&self.pool)
                .await?
                .into_iter()
                .map(|class| (class.name, class.hidden))
                .collect()
            }
        };

        let mut class_visibility = ForumItemClassVisibility {
            visible_to_class_names: Vec::new(),
            hidden_from_class_names: Vec::new(),
        };
        for (name, hidden) in classes {
            if hidden {
                class_visibility.hidden_from_class_names.push(name);
            } else {
                class_visibility.visible_to_class_names.push(name);
            }
        }

        Ok(class_visibility)
    }

    pub async fn find_user_class_forbidden_forum_items(
        &self,
        user_class_name: &str,
    ) -> Result<UserClassForbiddenForumItems> {
        let categories = sqlx::query_as!(
            ForumCategoryLite,
            r#"
            SELECT fc.id, fc.name
            FROM forum_category_forbidden_classes forbidden
            JOIN forum_categories fc ON fc.id = forbidden.forum_category_id
            WHERE forbidden.user_class_name = $1
            ORDER BY fc.name
            "#,
            user_class_name
        )
        .fetch_all(&self.pool)
        .await?;

        let sub_categories = sqlx::query_as!(
            ForumSubCategoryLite,
            r#"
            SELECT fsc.id, fsc.name
            FROM forum_sub_category_forbidden_classes forbidden
            JOIN forum_sub_categories fsc ON fsc.id = forbidden.forum_sub_category_id
            WHERE forbidden.user_class_name = $1
            ORDER BY fsc.name
            "#,
            user_class_name
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(UserClassForbiddenForumItems {
            categories,
            sub_categories,
        })
    }

    /// Sets the forum items hidden from a user class, refused when one of them doesn't exist.
    /// The editor can edit user classes, so every existing item is visible to them.
    pub async fn update_user_class_forbidden_forum_items(
        &self,
        user_class_name: &str,
        edited: &EditedUserClassForbiddenForumItems,
        editor_id: i32,
    ) -> Result<()> {
        let mut transaction = self.pool.begin().await?;

        if let Some(missing_item) = first_hidden_forum_item(
            &mut transaction,
            editor_id,
            &edited.category_ids,
            &edited.sub_category_ids,
        )
        .await?
        {
            return Err(forum_item_not_found_error(missing_item));
        }

        sqlx::query!(
            r#"
            DELETE FROM forum_category_forbidden_classes
            WHERE user_class_name = $1 AND NOT forum_category_id = ANY($2)
            "#,
            user_class_name,
            &edited.category_ids
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query!(
            r#"
            INSERT INTO forum_category_forbidden_classes (user_class_name, forum_category_id)
            SELECT $1, UNNEST($2::int[])
            ON CONFLICT DO NOTHING
            "#,
            user_class_name,
            &edited.category_ids
        )
        .execute(&mut *transaction)
        .await?;

        sqlx::query!(
            r#"
            DELETE FROM forum_sub_category_forbidden_classes
            WHERE user_class_name = $1 AND NOT forum_sub_category_id = ANY($2)
            "#,
            user_class_name,
            &edited.sub_category_ids
        )
        .execute(&mut *transaction)
        .await?;
        sqlx::query!(
            r#"
            INSERT INTO forum_sub_category_forbidden_classes (user_class_name, forum_sub_category_id)
            SELECT $1, UNNEST($2::int[])
            ON CONFLICT DO NOTHING
            "#,
            user_class_name,
            &edited.sub_category_ids
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }
}

pub(crate) async fn ensure_forum_thread_visible_with(
    connection: &mut PgConnection,
    user_id: i32,
    forum_thread_id: i64,
) -> Result<()> {
    let visible = sqlx::query_scalar!(
        "SELECT forum_thread_visible($1, $2)",
        user_id,
        forum_thread_id
    )
    .fetch_one(connection)
    .await?
    .unwrap_or(false);

    if visible {
        Ok(())
    } else {
        Err(Error::CouldNotFindForumThread(sqlx::Error::RowNotFound))
    }
}

fn forum_item_not_found_error(item: ForumItemId) -> Error {
    match item {
        ForumItemId::Category(_) => Error::ForumCategoryNotFound,
        ForumItemId::SubCategory(_) => Error::ForumSubCategoryNotFound,
    }
}

/// A missing item counts as hidden
pub(crate) async fn is_forum_item_visible(
    connection: &mut PgConnection,
    user_id: i32,
    item: ForumItemId,
) -> Result<bool> {
    let hidden_item = match item {
        ForumItemId::Category(id) => {
            first_hidden_forum_item(connection, user_id, &[id], &[]).await?
        }
        ForumItemId::SubCategory(id) => {
            first_hidden_forum_item(connection, user_id, &[], &[id]).await?
        }
    };

    Ok(hidden_item.is_none())
}

/// The first of the items that is hidden from the user or doesn't exist
async fn first_hidden_forum_item(
    connection: &mut PgConnection,
    user_id: i32,
    category_ids: &[i32],
    sub_category_ids: &[i32],
) -> Result<Option<ForumItemId>> {
    if !category_ids.is_empty() {
        let hidden_category_id = sqlx::query_scalar!(
            r#"
            SELECT requested.id AS "id!" FROM UNNEST($2::int[]) AS requested(id)
            LEFT JOIN forum_categories fc ON fc.id = requested.id
            WHERE fc.id IS NULL OR fc.id IN (SELECT * FROM forum_hidden_category_ids($1))
            LIMIT 1
            "#,
            user_id,
            category_ids
        )
        .fetch_optional(&mut *connection)
        .await?;
        if let Some(id) = hidden_category_id {
            return Ok(Some(ForumItemId::Category(id)));
        }
    }

    if !sub_category_ids.is_empty() {
        let hidden_sub_category_id = sqlx::query_scalar!(
            r#"
            SELECT requested.id AS "id!" FROM UNNEST($2::int[]) AS requested(id)
            LEFT JOIN forum_sub_categories fsc ON fsc.id = requested.id
            WHERE fsc.id IS NULL OR fsc.id IN (SELECT * FROM forum_hidden_sub_category_ids($1))
            LIMIT 1
            "#,
            user_id,
            sub_category_ids
        )
        .fetch_optional(&mut *connection)
        .await?;
        if let Some(id) = hidden_sub_category_id {
            return Ok(Some(ForumItemId::SubCategory(id)));
        }
    }

    Ok(None)
}

/// Replaces the users allowed in a forum item regardless of their class, as part of the
/// item's creation or edit. Refused when the editor wouldn't see the item anymore after the change.
pub(crate) async fn replace_forum_item_allowed_users(
    connection: &mut PgConnection,
    item: ForumItemId,
    allowed_user_ids: &[i32],
    editor_id: i32,
) -> Result<()> {
    let unknown_user_id = sqlx::query_scalar!(
        r#"
        SELECT requested_user.id AS "id!"
        FROM UNNEST($1::int[]) AS requested_user(id)
        WHERE NOT EXISTS (SELECT 1 FROM users WHERE users.id = requested_user.id)
        LIMIT 1
        "#,
        &allowed_user_ids[..]
    )
    .fetch_optional(&mut *connection)
    .await?;
    if let Some(unknown_user_id) = unknown_user_id {
        return Err(Error::UserWithIdNotFound(unknown_user_id));
    }

    match item {
        ForumItemId::Category(id) => {
            sqlx::query!(
                "DELETE FROM forum_category_allowed_users WHERE forum_category_id = $1",
                id
            )
            .execute(&mut *connection)
            .await?;
            sqlx::query!(
                r#"
                INSERT INTO forum_category_allowed_users (forum_category_id, user_id)
                SELECT $1, UNNEST($2::int[])
                ON CONFLICT DO NOTHING
                "#,
                id,
                &allowed_user_ids[..]
            )
            .execute(&mut *connection)
            .await?;
        }
        ForumItemId::SubCategory(id) => {
            sqlx::query!(
                "DELETE FROM forum_sub_category_allowed_users WHERE forum_sub_category_id = $1",
                id
            )
            .execute(&mut *connection)
            .await?;
            sqlx::query!(
                r#"
                INSERT INTO forum_sub_category_allowed_users (forum_sub_category_id, user_id)
                SELECT $1, UNNEST($2::int[])
                ON CONFLICT DO NOTHING
                "#,
                id,
                &allowed_user_ids[..]
            )
            .execute(&mut *connection)
            .await?;
        }
    }

    if !is_forum_item_visible(connection, editor_id, item).await? {
        return Err(Error::ForumItemWouldBeHiddenFromEditor);
    }

    Ok(())
}
