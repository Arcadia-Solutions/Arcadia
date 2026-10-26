use crate::{
    connection_pool::ConnectionPool,
    models::css_sheet::{CssSheet, EditedCssSheet, UserCreatedCssSheet},
};
use arcadia_common::error::{Error, Result};

impl ConnectionPool {
    pub async fn create_css_sheet(
        &self,
        css_sheet: &UserCreatedCssSheet,
        current_user_id: i32,
    ) -> Result<CssSheet> {
        let css_sheet = sqlx::query_as!(
            CssSheet,
            r#"
                INSERT INTO css_sheets (name, css, created_by_id, preview_image_url)
                VALUES ($1, $2, $3, $4)
                RETURNING created_at, created_by_id, name, css, preview_image_url
            "#,
            css_sheet.name.trim(),
            css_sheet.css,
            current_user_id,
            css_sheet.preview_image_url.trim()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CouldNotCreateCssSheet)?;

        Ok(css_sheet)
    }

    pub async fn find_css_sheets(&self) -> Result<Vec<CssSheet>> {
        let sheets = sqlx::query_as!(
            CssSheet,
            r#"
                SELECT created_at, created_by_id, name, css, preview_image_url FROM css_sheets
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::CouldNotFindCssSheets)?;

        Ok(sheets)
    }

    pub async fn find_css_sheet(&self, name: &str) -> Result<CssSheet> {
        let sheet = sqlx::query_as!(
            CssSheet,
            r#"
                SELECT created_at, created_by_id, name, css, preview_image_url FROM css_sheets WHERE name = $1
            "#,
            name
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CssSheetNotFound)?;

        Ok(sheet)
    }

    pub async fn update_css_sheet(&self, form: &EditedCssSheet) -> Result<CssSheet> {
        let css_sheet = sqlx::query_as!(
            CssSheet,
            r#"
                UPDATE css_sheets
                SET name = $2, css = $3, preview_image_url = $4
                WHERE name = $1
                RETURNING created_at, created_by_id, name, css, preview_image_url
            "#,
            form.old_name,
            form.name,
            form.css,
            form.preview_image_url
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CssSheetNotFound)?;

        Ok(css_sheet)
    }

    pub async fn get_css_sheet_content(&self, name: &str) -> Result<String> {
        let css = sqlx::query!(
            r#"
                SELECT css FROM css_sheets
                WHERE name = $1
            "#,
            name
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::CssSheetNotFound)?;

        Ok(css.css)
    }
}
