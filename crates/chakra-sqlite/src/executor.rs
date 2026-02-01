//! SQLite query executor

use crate::connection::SqliteConnection;
use crate::types::{row_to_chakra, to_sqlite_value};
use chakra_core::error::{ChakraError, QueryError, Result};
use chakra_core::result::Row;
use chakra_core::sql::{SqlFragment, SqliteDialect};
use chakra_core::types::Value;
use rusqlite::params_from_iter;
use std::sync::Arc;
use tracing::{debug, error};

/// SQLite query executor
pub struct SqliteExecutor {
    conn: Arc<SqliteConnection>,
    dialect: SqliteDialect,
}

impl SqliteExecutor {
    /// Create a new executor
    pub fn new(conn: Arc<SqliteConnection>) -> Self {
        Self {
            conn,
            dialect: SqliteDialect,
        }
    }

    /// Get the dialect
    pub fn dialect(&self) -> &SqliteDialect {
        &self.dialect
    }

    /// Execute a query and return rows
    pub async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>> {
        let sql = sql.to_string();
        let params: Vec<_> = params.iter().map(to_sqlite_value).collect();

        self.conn
            .call(move |conn| {
                let mut stmt = conn.prepare(&sql)?;

                let column_names: Vec<String> = stmt
                    .column_names()
                    .iter()
                    .map(|s| s.to_string())
                    .collect();

                let rows: Vec<Row> = stmt
                    .query_map(params_from_iter(params.iter()), |row| {
                        row_to_chakra(row, &column_names)
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;

                Ok(rows)
            })
            .await
    }

    /// Execute a query with a SqlFragment
    pub async fn query_fragment(&self, fragment: &SqlFragment) -> Result<Vec<Row>> {
        self.query(&fragment.sql, &fragment.params).await
    }

    /// Execute a query and return a single row
    pub async fn query_one(&self, sql: &str, params: &[Value]) -> Result<Option<Row>> {
        let rows = self.query(sql, params).await?;
        Ok(rows.into_iter().next())
    }

    /// Execute a statement and return affected row count
    pub async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64> {
        let sql = sql.to_string();
        let params: Vec<_> = params.iter().map(to_sqlite_value).collect();

        self.conn
            .call(move |conn| {
                let count = conn.execute(&sql, params_from_iter(params.iter()))?;
                Ok(count as u64)
            })
            .await
    }

    /// Execute a statement with a SqlFragment
    pub async fn execute_fragment(&self, fragment: &SqlFragment) -> Result<u64> {
        self.execute(&fragment.sql, &fragment.params).await
    }

    /// Execute multiple statements in a batch
    pub async fn execute_batch(&self, sql: &str) -> Result<()> {
        let sql = sql.to_string();

        self.conn
            .call(move |conn| {
                conn.execute_batch(&sql)?;
                Ok(())
            })
            .await
    }

    /// Begin a transaction
    pub async fn begin(&self) -> Result<()> {
        self.execute_batch("BEGIN").await
    }

    /// Commit a transaction
    pub async fn commit(&self) -> Result<()> {
        self.execute_batch("COMMIT").await
    }

    /// Rollback a transaction
    pub async fn rollback(&self) -> Result<()> {
        self.execute_batch("ROLLBACK").await
    }

    /// Get the last inserted row ID
    pub async fn last_insert_rowid(&self) -> Result<i64> {
        self.conn
            .call(|conn| Ok(conn.last_insert_rowid()))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_query_execute() {
        let conn = Arc::new(SqliteConnection::open_memory().await.unwrap());
        let executor = SqliteExecutor::new(conn);

        // Create table
        executor
            .execute_batch("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT)")
            .await
            .unwrap();

        // Insert
        executor
            .execute(
                "INSERT INTO users (name) VALUES (?)",
                &[Value::String("Alice".to_string())],
            )
            .await
            .unwrap();

        // Query
        let rows = executor.query("SELECT * FROM users", &[]).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("name"), Some(&Value::String("Alice".to_string())));
    }
}
