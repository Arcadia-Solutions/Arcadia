use crate::{
    connection_pool::ConnectionPool,
    models::{
        arcadia_settings::TorrentRequestVoteCurrency,
        bonus_points_log::BonusPointsLogAction,
        torrent_request_vote::{TorrentRequestVote, UserCreatedTorrentRequestVote},
    },
};
use arcadia_common::error::{Error, Result};

impl ConnectionPool {
    pub async fn create_torrent_request_vote(
        &self,
        torrent_request_vote: &UserCreatedTorrentRequestVote,
        user_id: i32,
        vote_currencies: &[TorrentRequestVoteCurrency],
    ) -> Result<TorrentRequestVote> {
        if torrent_request_vote.bounty_upload < 0 || torrent_request_vote.bounty_bonus_points < 0 {
            return Err(Error::VoteBountyNegative);
        }

        let has_upload = vote_currencies.contains(&TorrentRequestVoteCurrency::Upload)
            && torrent_request_vote.bounty_upload > 0;
        let has_bonus_points = vote_currencies.contains(&TorrentRequestVoteCurrency::BonusPoints)
            && torrent_request_vote.bounty_bonus_points > 0;

        if !has_upload && !has_bonus_points {
            return Err(Error::VoteBountyRequired);
        }

        let mut tx = self.pool.begin().await?;

        // the balance is locked until the debit commits, so that concurrent votes
        // can not all be checked against the same balance
        let current_user = sqlx::query!(
            r#"
                SELECT id, uploaded, bonus_points FROM users
                WHERE id = $1
                FOR UPDATE
            "#,
            user_id
        )
        .fetch_one(&mut *tx)
        .await?;
        if current_user.bonus_points - torrent_request_vote.bounty_bonus_points < 0 {
            return Err(Error::InsufficientBonusPointsForBounty);
        }
        if current_user.uploaded - torrent_request_vote.bounty_upload < 0 {
            return Err(Error::InsufficientUploadForBounty);
        }
        // TODO config: check if the bounty is above the minimum set in the config
        // TODO config: check if the user's ratio stays above the minimum ratio set in the config (after the uploaded amount changes)

        let created_torrent_request_vote = sqlx::query_as!(
            TorrentRequestVote,
            r#"
                WITH inserted_vote AS (
                    INSERT INTO torrent_request_votes (torrent_request_id, created_by_id,
                                                      bounty_upload, bounty_bonus_points)
                    VALUES ($1, $2, $3, $4)
                    RETURNING id, torrent_request_id, created_at, created_by_id, bounty_upload, bounty_bonus_points
                ),
                is_first_vote AS (
                    SELECT NOT EXISTS (
                        SELECT 1
                        FROM torrent_request_votes
                        WHERE torrent_request_id = $1
                          AND created_by_id = $2
                          AND id != (SELECT id FROM inserted_vote)
                    ) AS first_vote
                ),
                updated_user AS (
                    UPDATE users u
                    SET
                        uploaded = u.uploaded - $3,
                        bonus_points = u.bonus_points - $4,
                        requests_voted = u.requests_voted + CASE WHEN (SELECT first_vote FROM is_first_vote) THEN 1 ELSE 0 END
                    WHERE u.id = (SELECT created_by_id FROM inserted_vote)
                )
                SELECT
                    inserted_vote.id, inserted_vote.torrent_request_id, inserted_vote.created_at, inserted_vote.created_by_id, inserted_vote.bounty_upload, inserted_vote.bounty_bonus_points
                FROM inserted_vote
            "#,
            torrent_request_vote.torrent_request_id,
            current_user.id,
            torrent_request_vote.bounty_upload,
            torrent_request_vote.bounty_bonus_points
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(Error::CouldNotCreateTorrentRequestVote)?;

        if has_bonus_points {
            let title_group_name = Self::fetch_title_group_name_for_torrent_request(
                &mut tx,
                torrent_request_vote.torrent_request_id,
            )
            .await?;

            Self::log_bonus_points_change_tx(
                &mut tx,
                current_user.id,
                BonusPointsLogAction::TorrentRequestVoteSpent,
                -torrent_request_vote.bounty_bonus_points,
                Some(&title_group_name),
                Some(torrent_request_vote.torrent_request_id),
            )
            .await?;
        }

        tx.commit().await?;

        Ok(created_torrent_request_vote)
    }
}
