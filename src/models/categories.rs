use loco_rs::model::ModelResult;
use sea_orm::entity::prelude::*;
use sea_orm::{FromQueryResult, DbBackend, JsonValue, Statement};
use super::_entities::categories::{ActiveModel, Model};
use crate::views::category::{CategoryListResponse, CategoryTree};

impl ActiveModelBehavior for ActiveModel {
    // extend activemodel below (keep comment for generators)
}

impl Model {
    pub async fn get_all_categories(
        db: &DatabaseConnection,
    ) -> ModelResult<Vec<CategoryListResponse>> {
        let categories: Vec<CategoryListResponse> = JsonValue::find_by_statement(Statement::from_sql_and_values(
            DbBackend::Postgres,
            r#"
                SELECT
                    p.id,
                    p.name,
                    COALESCE((
                       SELECT json_agg(json_build_object('id', c.id, 'name', c.name))
                       FROM categories c where c.parent_id = p.id
                    ), '[]'::json) as child
                FROM categories p
                WHERE p.parent_id IS NULL OR p.parent_id = 0
                GROUP BY p.id, p.name
            "#,
            [],
        )).into_model::<CategoryListResponse>()
            .all(db)
            .await?;
        Ok(categories)
    }

    pub async fn get_category_tree(
        db: &DatabaseConnection,
    ) -> Result<Vec<CategoryTree>, sea_orm::DbErr> {

        let sql = r#"
        WITH RECURSIVE cte AS (
            SELECT
                c.*,
                c.id::text AS path
            FROM categories c
            WHERE parent_id IS NULL

            UNION ALL

            SELECT
                c.*,
                ct.path || '/' || c.id::text AS path
            FROM categories c
            INNER JOIN cte ct ON c.parent_id = ct.id
        )
        SELECT * FROM cte ORDER BY path
    "#;

        CategoryTree::find_by_statement(
            Statement::from_string(DbBackend::Postgres, sql.to_string())
        )
            .all(db)
            .await
    }
}
