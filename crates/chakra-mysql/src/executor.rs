//! MySQL query executor

use crate::connection::MySqlPool;
use crate::types::to_mysql_value;
use chakra_core::error::{ChakraError, QueryError, Result};
use chakra_core::result::Row;
use chakra_core::sql::{MySqlDialect, SqlFragment};
use chakra_core::types::Value;
use mysql_async::prelude::*;
use std::sync::Arc;
use tracing::{debug, error};

/// MySQL query executor
pub struct MySqlExecutor {
    pool: Arc<MySqlPool>,
    dialect: MySqlDialect,
}

impl MySqlExecutor {
    /// Create a new executor
    pub fn new(pool: Arc<MySqlPool>) -> Self {
        Self {
            pool,
            dialect: MySqlDialect,
        }
    }

    /// Get the dialect
    pub fn dialect(&self) -> &MySqlDialect {
        &self.dialect
    }

    /// Execute a query and return rows
    pub async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>> {
        let mut conn = self.pool.get().await?;

        debug!("Executing query: {} with {} params", sql, params.len());

        let mysql_params: Vec<mysql_async::Value> = params.iter().map(to_mysql_value).collect();

        let result: Vec<mysql_async::Row> = conn
            .inner()
            .exec(sql, mysql_params)
            .await
            .map_err(|e| {
                error!("Query failed: {}", e);
                ChakraError::Query(QueryError::ExecutionFailed {
                    message: e.to_string(),
                })
            })?;

        Ok(result.into_iter().map(mysql_row_to_chakra).collect())
    }

    /// Execute a query with a SqlFragment
    pub async fn query_fragment(&self, fragment: &SqlFragment) -> Result<Vec<Row>> {
        self.query(&fragment.sql, &fragment.params).await
    }

    /// Execute a statement and return affected row count
    pub async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64> {
        let mut conn = self.pool.get().await?;

        debug!("Executing statement: {} with {} params", sql, params.len());

        let mysql_params: Vec<mysql_async::Value> = params.iter().map(to_mysql_value).collect();

        conn.inner()
            .exec_drop(sql, mysql_params)
            .await
            .map_err(|e| {
                error!("Statement failed: {}", e);
                ChakraError::Query(QueryError::ExecutionFailed {
                    message: e.to_string(),
                })
            })?;

        Ok(conn.inner().affected_rows())
    }

    /// Execute a statement with a SqlFragment
    pub async fn execute_fragment(&self, fragment: &SqlFragment) -> Result<u64> {
        self.execute(&fragment.sql, &fragment.params).await
    }
}

/// Convert a MySQL row to a Chakra row
fn mysql_row_to_chakra(row: mysql_async::Row) -> Row {
    let columns: Vec<String> = row
        .columns_ref()
        .iter()
        .map(|c| c.name_str().to_string())
        .collect();

    let values: Vec<Value> = (0..columns.len())
        .map(|i| {
            let val: mysql_async::Value = row.get(i).unwrap_or(mysql_async::Value::NULL);
            crate::types::from_mysql_value(val)
        })
        .collect();

    Row::new(columns, values)
}

#[cfg(test)]
mod tests {
    // Integration tests would require a running MySQL instance
}
