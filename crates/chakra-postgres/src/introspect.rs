//! PostgreSQL schema introspection

use crate::connection::PostgresPool;
use async_trait::async_trait;
use chakra_core::error::Result;
use chakra_schema::introspect::{
    RawColumnInfo, RawConstraintInfo, RawIndexInfo, RawTableInfo, SchemaIntrospector,
};
use chakra_schema::schema::{Schema, Table};
use std::sync::Arc;
use tracing::debug;

/// PostgreSQL schema introspector
pub struct PostgresIntrospector {
    pool: Arc<PostgresPool>,
}

impl PostgresIntrospector {
    /// Create a new introspector
    pub fn new(pool: Arc<PostgresPool>) -> Self {
        Self { pool }
    }

    /// Get tables query
    fn tables_query(&self, schema: &str) -> String {
        format!(
            r#"
            SELECT
                table_schema,
                table_name,
                table_type,
                obj_description((quote_ident(table_schema) || '.' || quote_ident(table_name))::regclass, 'pg_class') as comment
            FROM information_schema.tables
            WHERE table_schema = '{}'
            AND table_type IN ('BASE TABLE', 'VIEW')
            ORDER BY table_name
            "#,
            schema
        )
    }

    /// Get columns query
    fn columns_query(&self, schema: &str, table: &str) -> String {
        format!(
            r#"
            SELECT
                c.table_name,
                c.column_name,
                c.ordinal_position,
                c.column_default,
                c.is_nullable = 'YES' as is_nullable,
                c.data_type,
                c.character_maximum_length,
                c.numeric_precision,
                c.numeric_scale,
                c.is_identity = 'YES' as is_identity,
                c.identity_generation,
                col_description((quote_ident(c.table_schema) || '.' || quote_ident(c.table_name))::regclass, c.ordinal_position) as comment
            FROM information_schema.columns c
            WHERE c.table_schema = '{}'
            AND c.table_name = '{}'
            ORDER BY c.ordinal_position
            "#,
            schema, table
        )
    }

    /// Get indexes query
    fn indexes_query(&self, schema: &str, table: &str) -> String {
        format!(
            r#"
            SELECT
                t.relname as table_name,
                i.relname as index_name,
                ix.indisunique as is_unique,
                ix.indisprimary as is_primary,
                am.amname as index_type,
                pg_get_expr(ix.indpred, ix.indrelid) as where_clause,
                array_agg(a.attname ORDER BY array_position(ix.indkey, a.attnum)) as column_names
            FROM pg_index ix
            JOIN pg_class t ON t.oid = ix.indrelid
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_am am ON am.oid = i.relam
            LEFT JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
            WHERE n.nspname = '{}'
            AND t.relname = '{}'
            GROUP BY t.relname, i.relname, ix.indisunique, ix.indisprimary, am.amname, ix.indpred, ix.indrelid
            "#,
            schema, table
        )
    }

    /// Get constraints query
    fn constraints_query(&self, schema: &str, table: &str) -> String {
        format!(
            r#"
            SELECT
                tc.table_name,
                tc.constraint_name,
                tc.constraint_type,
                array_agg(kcu.column_name ORDER BY kcu.ordinal_position) as columns,
                cc.check_clause as check_expression,
                ccu.table_name as references_table,
                array_agg(ccu.column_name) as references_columns,
                rc.delete_rule as on_delete,
                rc.update_rule as on_update
            FROM information_schema.table_constraints tc
            LEFT JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            LEFT JOIN information_schema.check_constraints cc
                ON tc.constraint_name = cc.constraint_name
                AND tc.table_schema = cc.constraint_schema
            LEFT JOIN information_schema.constraint_column_usage ccu
                ON tc.constraint_name = ccu.constraint_name
                AND tc.table_schema = ccu.constraint_schema
                AND tc.constraint_type = 'FOREIGN KEY'
            LEFT JOIN information_schema.referential_constraints rc
                ON tc.constraint_name = rc.constraint_name
                AND tc.table_schema = rc.constraint_schema
            WHERE tc.table_schema = '{}'
            AND tc.table_name = '{}'
            GROUP BY tc.table_name, tc.constraint_name, tc.constraint_type, cc.check_clause,
                     ccu.table_name, rc.delete_rule, rc.update_rule
            "#,
            schema, table
        )
    }
}

#[async_trait]
impl SchemaIntrospector for PostgresIntrospector {
    async fn introspect(&self) -> Result<Schema> {
        self.introspect_schema("public").await
    }

    async fn introspect_schema(&self, schema_name: &str) -> Result<Schema> {
        let mut schema = Schema::with_name(schema_name);
        let tables = self.list_tables(Some(schema_name)).await?;

        for table_name in tables {
            let table = self.introspect_table(&table_name).await?;
            schema.add_table(table);
        }

        debug!(
            "Introspected schema {} with {} tables",
            schema_name,
            schema.tables.len()
        );

        Ok(schema)
    }

    async fn introspect_table(&self, table_name: &str) -> Result<Table> {
        let conn = self.pool.get().await?;
        let schema_name = self.pool.config().schema.as_deref().unwrap_or("public");

        // Get columns
        let column_rows = conn
            .client
            .query(&self.columns_query(schema_name, table_name), &[])
            .await
            .map_err(|e| chakra_core::error::ChakraError::internal(e.to_string()))?;

        let mut table = Table::new(table_name);

        for row in &column_rows {
            let column_info = RawColumnInfo {
                table_name: row.get("table_name"),
                column_name: row.get("column_name"),
                ordinal_position: row.get("ordinal_position"),
                column_default: row.get("column_default"),
                is_nullable: row.get("is_nullable"),
                data_type: row.get("data_type"),
                character_maximum_length: row.get("character_maximum_length"),
                numeric_precision: row.get("numeric_precision"),
                numeric_scale: row.get("numeric_scale"),
                is_identity: row.get("is_identity"),
                identity_generation: row.get("identity_generation"),
                comment: row.get("comment"),
            };

            table.add_column(column_info.to_column());
        }

        // Get constraints
        let constraint_rows = conn
            .client
            .query(&self.constraints_query(schema_name, table_name), &[])
            .await
            .map_err(|e| chakra_core::error::ChakraError::internal(e.to_string()))?;

        for row in &constraint_rows {
            let constraint_type: String = row.get("constraint_type");
            let columns: Vec<String> = row.get("columns");

            match constraint_type.as_str() {
                "PRIMARY KEY" => {
                    table.primary_key = Some(chakra_schema::schema::PrimaryKey::new(columns));
                }
                "UNIQUE" | "CHECK" | "FOREIGN KEY" => {
                    // Handle other constraints
                }
                _ => {}
            }
        }

        // Get indexes
        let index_rows = conn
            .client
            .query(&self.indexes_query(schema_name, table_name), &[])
            .await
            .map_err(|e| chakra_core::error::ChakraError::internal(e.to_string()))?;

        for row in &index_rows {
            let is_primary: bool = row.get("is_primary");
            if !is_primary {
                let index = chakra_schema::schema::Index::new(
                    row.get::<_, String>("index_name"),
                    row.get::<_, Vec<String>>("column_names"),
                );

                let is_unique: bool = row.get("is_unique");
                table.add_index(if is_unique { index.unique() } else { index });
            }
        }

        Ok(table)
    }

    async fn list_schemas(&self) -> Result<Vec<String>> {
        let conn = self.pool.get().await?;

        let rows = conn
            .client
            .query(
                "SELECT schema_name FROM information_schema.schemata
                 WHERE schema_name NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
                 ORDER BY schema_name",
                &[],
            )
            .await
            .map_err(|e| chakra_core::error::ChakraError::internal(e.to_string()))?;

        Ok(rows.iter().map(|r| r.get(0)).collect())
    }

    async fn list_tables(&self, schema_name: Option<&str>) -> Result<Vec<String>> {
        let conn = self.pool.get().await?;
        let schema = schema_name.unwrap_or("public");

        let rows = conn
            .client
            .query(&self.tables_query(schema), &[])
            .await
            .map_err(|e| chakra_core::error::ChakraError::internal(e.to_string()))?;

        Ok(rows.iter().map(|r| r.get("table_name")).collect())
    }

    async fn table_exists(&self, table_name: &str) -> Result<bool> {
        let conn = self.pool.get().await?;
        let schema = self.pool.config().schema.as_deref().unwrap_or("public");

        let rows = conn
            .client
            .query(
                "SELECT 1 FROM information_schema.tables
                 WHERE table_schema = $1 AND table_name = $2",
                &[&schema, &table_name],
            )
            .await
            .map_err(|e| chakra_core::error::ChakraError::internal(e.to_string()))?;

        Ok(!rows.is_empty())
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would require a running PostgreSQL instance
}
