use crate::{
    connection_pool::ConnectionPool,
    models::{
        title_group_stats::{
            CatalogKind, TitleGroupAttributeCountDataPoint, TitleGroupStatsResponse,
        },
        torrent_stats::TitleGroupsPerReleaseYearDataPoint,
    },
};
use arcadia_common::error::Result;
use sqlx::types::Json;

// Postgres truncates a column alias to 63 bytes, so the sqlx type overrides below have to stay
// short: a truncated override arrives unparseable instead of failing with a clear error.
type ReleaseYears = Vec<TitleGroupsPerReleaseYearDataPoint>;
type AttributeCounts = Vec<TitleGroupAttributeCountDataPoint>;

impl ConnectionPool {
    /// Aggregates every title group of a single catalog, be it an artist, a series or a collage,
    /// grouped by release year and content type. The two aggregations share a single scoped CTE,
    /// so the catalog is resolved only once.
    ///
    /// The scope is a UNION ALL of the three catalogs rather than a CASE over one filter, because
    /// the planner cannot tell which branch a parameter will take and so estimates the CASE as if
    /// every title group matched, falling back to a scan of the whole table. As a UNION ALL each
    /// branch keeps its own estimate and is read through its index. Only the branch whose kind
    /// equals the given one can return rows, so no title group is ever counted twice.
    pub async fn get_title_group_catalog_stats(
        &self,
        catalog: CatalogKind,
        catalog_id: i64,
    ) -> Result<TitleGroupStatsResponse> {
        let row = sqlx::query!(
            r#"
            WITH scoped_title_groups AS MATERIALIZED (
                SELECT tg.original_release_date, tg.content_type
                FROM affiliated_artists aa
                JOIN title_groups tg ON tg.id = aa.title_group_id
                WHERE $1 = 'artist' AND aa.artist_id = $2
                UNION ALL
                SELECT tg.original_release_date, tg.content_type
                FROM title_groups tg
                WHERE $1 = 'series' AND tg.series_id = $2
                UNION ALL
                SELECT tg.original_release_date, tg.content_type
                FROM collage_entry ce
                JOIN title_groups tg ON tg.id = ce.title_group_id
                WHERE $1 = 'collage' AND ce.collage_id = $2
            ),
            release_years AS (
                SELECT
                    EXTRACT(YEAR FROM original_release_date)::INT AS year,
                    COUNT(*)::BIGINT AS count
                FROM scoped_title_groups
                GROUP BY year
            ),
            content_types AS (
                SELECT
                    content_type::TEXT AS attribute_value,
                    COUNT(*)::BIGINT AS count
                FROM scoped_title_groups
                GROUP BY attribute_value
            )
            SELECT
                COALESCE(
                    (SELECT jsonb_agg(to_jsonb(ry) ORDER BY ry.year NULLS LAST) FROM release_years ry),
                    '[]'::jsonb
                ) AS "title_groups_per_release_year!: Json<ReleaseYears>",
                COALESCE(
                    (SELECT jsonb_agg(to_jsonb(ct) ORDER BY ct.attribute_value) FROM content_types ct),
                    '[]'::jsonb
                ) AS "content_types!: Json<AttributeCounts>"
            "#,
            catalog.to_string(),
            catalog_id,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(TitleGroupStatsResponse {
            title_groups_per_release_year: row.title_groups_per_release_year.0,
            content_types: row.content_types.0,
        })
    }
}
