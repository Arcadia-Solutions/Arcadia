use crate::{
    connection_pool::ConnectionPool,
    models::master_group::{MasterGroup, UserCreatedMasterGroup},
};
use arcadia_common::error::{Error, Result};

impl ConnectionPool {
    pub async fn create_master_group(
        &self,
        master_group_form: &UserCreatedMasterGroup,
        current_user_id: i32,
    ) -> Result<MasterGroup> {
        let created_master_group = sqlx::query_as!(
            MasterGroup,
            r#"
                INSERT INTO master_groups (name,created_by_id)
                VALUES ($1, $2)
                RETURNING id, name, created_at, updated_at, created_by_id
            "#,
            master_group_form.name,
            current_user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateMasterGroup)?;

        Ok(created_master_group)
    }
}
