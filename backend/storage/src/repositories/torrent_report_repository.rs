use crate::{
    connection_pool::ConnectionPool,
    models::{
        torrent_report::{TorrentReport, UserCreatedTorrentReport},
        user::UserLite,
    },
};
use arcadia_common::error::{Error, Result};
use sqlx::types::Json;

impl ConnectionPool {
    pub async fn report_torrent(
        &self,
        form: &UserCreatedTorrentReport,
        user_id: i32,
    ) -> Result<TorrentReport> {
        let torrent_report = sqlx::query_as!(
            TorrentReport,
            r#"
                WITH tr AS (
                    INSERT INTO torrent_reports (reported_by_id, reported_torrent_id, description)
                    VALUES ($1, $2, $3)
                    RETURNING *
                )
                SELECT tr.id, tr.reported_at, tr.reported_torrent_id, tr.description,
                    json_build_object('id', u.id, 'username', u.username, 'warned', u.warned, 'banned', u.banned) AS "reported_by!: Json<UserLite>"
                FROM tr
                JOIN users u ON u.id = tr.reported_by_id
            "#,
            user_id,
            form.reported_torrent_id,
            form.description,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateTorrentReport)?;

        Ok(torrent_report)
    }

    pub async fn delete_torrent_report(&self, torrent_report_id: i64) -> Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM torrent_reports
                WHERE id = $1
            "#,
            torrent_report_id,
        )
        .execute(&self.pool)
        .await
        .map_err(Error::CouldNotDeleteTorrentReport)?;

        Ok(())
    }

    pub async fn torrent_report_exists(&self, torrent_report_id: i64) -> Result<bool> {
        sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM torrent_reports WHERE id = $1) AS "exists!""#,
            torrent_report_id,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotGetTorrentReport)
    }
}
