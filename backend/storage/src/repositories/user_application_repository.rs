use crate::{
    connection_pool::ConnectionPool,
    models::{
        common::PaginatedResults,
        user::UserLite,
        user_application::{
            UserApplication, UserApplicationHierarchy, UserApplicationStatus,
            UserCreatedUserApplication,
        },
    },
};
use arcadia_common::error::{Error, Result};
use sqlx::types::ipnetwork::IpNetwork;

impl ConnectionPool {
    pub async fn create_user_application(
        &self,
        application: &UserCreatedUserApplication,
        from_ip: IpNetwork,
    ) -> Result<UserApplication> {
        let created_application = sqlx::query_as!(
            UserApplication,
            r#"
                INSERT INTO user_applications (body, referral, email, applied_from_ip, staff_note, status)
                VALUES ($1, $2, $3, $4, '', 'pending')
                RETURNING id, created_at, body, email, referral,
                          applied_from_ip as "applied_from_ip: IpNetwork",
                          staff_note, status as "status: UserApplicationStatus"
            "#,
            application.body,
            application.referral,
            application.email,
            from_ip
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateUserApplication)?;

        Ok(created_application)
    }

    pub async fn find_user_applications(
        &self,
        page_size: i64,
        page: i64,
        status: Option<UserApplicationStatus>,
    ) -> Result<PaginatedResults<UserApplicationHierarchy>> {
        let offset = (page - 1) * page_size;

        let total_items: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM user_applications
            WHERE $1::user_application_status_enum IS NULL
               OR status = $1::user_application_status_enum
            "#,
            status.clone() as Option<UserApplicationStatus>
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotGetUserApplications)?
        .unwrap_or(0);

        let rows = sqlx::query!(
            r#"
            SELECT ua.id, ua.created_at, ua.body, ua.email, ua.referral,
                   ua.applied_from_ip, ua.staff_note,
                   ua.status as "status: UserApplicationStatus",
                   u.id as "user_id?", u.username as "username?",
                   u.warned as "warned?", u.banned as "banned?"
            FROM user_applications ua
            LEFT JOIN invitations i ON i.user_application_id = ua.id
            LEFT JOIN users u ON u.id = i.receiver_id
            WHERE $1::user_application_status_enum IS NULL
               OR ua.status = $1::user_application_status_enum
            ORDER BY ua.created_at DESC
            OFFSET $2 LIMIT $3
            "#,
            status as Option<UserApplicationStatus>,
            offset,
            page_size
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotGetUserApplications)?;

        let results = rows
            .into_iter()
            .map(|row| {
                let user = if let (Some(id), Some(username), Some(warned), Some(banned)) =
                    (row.user_id, row.username, row.warned, row.banned)
                {
                    Some(UserLite {
                        id,
                        username,
                        warned,
                        banned,
                    })
                } else {
                    None
                };
                UserApplicationHierarchy {
                    id: row.id,
                    created_at: row.created_at,
                    body: row.body,
                    email: row.email,
                    referral: row.referral,
                    applied_from_ip: row.applied_from_ip,
                    staff_note: row.staff_note,
                    status: row.status,
                    user,
                }
            })
            .collect();

        Ok(PaginatedResults {
            results,
            total_items,
            page: page as u32,
            page_size: page_size as u32,
        })
    }

    pub async fn update_user_application_status(
        &self,
        application_id: i64,
        status: UserApplicationStatus,
    ) -> Result<UserApplication> {
        let application = sqlx::query_as!(
            UserApplication,
            r#"
                UPDATE user_applications
                SET status = $2::user_application_status_enum
                WHERE id = $1
                RETURNING id, created_at, body, email, referral,
                          applied_from_ip as "applied_from_ip: IpNetwork",
                          staff_note, status as "status: UserApplicationStatus"
            "#,
            application_id,
            status as UserApplicationStatus
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotUpdateUserApplication)?;

        Ok(application)
    }
}
