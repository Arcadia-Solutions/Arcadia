use crate::{
    connection_pool::ConnectionPool,
    models::{user::UserPermission, user_staff_note::UserStaffNoteWithAuthor},
};
use arcadia_common::error::{Error, Result};

impl ConnectionPool {
    /// The notes of a user, from the most recent to the oldest one. A user with the
    /// `view_foreign_user_staff_notes` permission gets every note of the user, any other user
    /// only gets the notes they wrote themselves.
    pub async fn find_user_staff_notes(
        &self,
        user_id: i32,
        can_view_foreign_notes: bool,
        requesting_user_id: i32,
    ) -> Result<Vec<UserStaffNoteWithAuthor>> {
        sqlx::query_as!(
            UserStaffNoteWithAuthor,
            r#"
                SELECT
                    usn.id, usn.user_id, usn.created_at, usn.content,
                    u.id AS author_id, u.username AS author_username
                FROM user_staff_notes usn
                JOIN users u ON u.id = usn.created_by_id
                WHERE usn.user_id = $1
                AND ($2 OR usn.created_by_id = $3)
                ORDER BY usn.created_at DESC
            "#,
            user_id,
            can_view_foreign_notes,
            requesting_user_id,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotFindUserStaffNotes)
    }

    async fn find_user_staff_note(&self, staff_note_id: i64) -> Result<UserStaffNoteWithAuthor> {
        sqlx::query_as!(
            UserStaffNoteWithAuthor,
            r#"
                SELECT
                    usn.id, usn.user_id, usn.created_at, usn.content,
                    u.id AS author_id, u.username AS author_username
                FROM user_staff_notes usn
                JOIN users u ON u.id = usn.created_by_id
                WHERE usn.id = $1
            "#,
            staff_note_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::CouldNotFindUserStaffNotes)?
        .ok_or(Error::UserStaffNoteNotFound)
    }

    /// Checks that a user may edit or remove one of the notes of a user, without loading it for
    /// the handler: a user with the `edit_user_staff_notes` permission may act on every note, the
    /// author of a note may act on it during the
    /// [`STAFF_NOTE_EDIT_WINDOW_IN_HOURS`](crate::models::user_staff_note::STAFF_NOTE_EDIT_WINDOW_IN_HOURS)
    /// hours following its creation.
    pub async fn check_user_staff_note_can_be_edited(
        &self,
        user_id: i32,
        staff_note_id: i64,
        requesting_user_id: i32,
        has_permission: bool,
    ) -> Result<()> {
        let note = self.find_user_staff_note(staff_note_id).await?;

        if note.user_id != user_id {
            return Err(Error::UserStaffNoteNotFound);
        }

        if !note.can_be_edited_by(requesting_user_id, has_permission) {
            return Err(Error::InsufficientPermissions(format!(
                "{:?}",
                UserPermission::EditUserStaffNotes
            )));
        }

        Ok(())
    }

    pub async fn create_user_staff_note(
        &self,
        user_id: i32,
        created_by_id: i32,
        content: &str,
    ) -> Result<UserStaffNoteWithAuthor> {
        sqlx::query_as!(
            UserStaffNoteWithAuthor,
            r#"
                WITH inserted_note AS (
                    INSERT INTO user_staff_notes (user_id, created_by_id, content)
                    VALUES ($1, $2, $3)
                    RETURNING id, user_id, created_at, content, created_by_id
                )
                SELECT
                    inserted_note.id, inserted_note.user_id, inserted_note.created_at,
                    inserted_note.content,
                    u.id AS author_id, u.username AS author_username
                FROM inserted_note
                JOIN users u ON u.id = inserted_note.created_by_id
            "#,
            user_id,
            created_by_id,
            content
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateUserStaffNote)
    }

    pub async fn edit_user_staff_note(&self, staff_note_id: i64, content: &str) -> Result<()> {
        let result = sqlx::query!(
            r#"
                UPDATE user_staff_notes
                SET content = $2
                WHERE id = $1
            "#,
            staff_note_id,
            content
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotEditUserStaffNote)?;

        if result.rows_affected() == 0 {
            return Err(Error::UserStaffNoteNotFound);
        }

        Ok(())
    }

    pub async fn delete_user_staff_note(&self, staff_note_id: i64) -> Result<()> {
        let result = sqlx::query!(
            r#"DELETE FROM user_staff_notes WHERE id = $1"#,
            staff_note_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteUserStaffNote)?;

        if result.rows_affected() == 0 {
            return Err(Error::UserStaffNoteNotFound);
        }

        Ok(())
    }
}
