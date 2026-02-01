//! PostgreSQL query executor

use crate::connection::PostgresPool;
use crate::types::{row_from_postgres, to_postgres_param};
use async_trait::async_trait;
use chakra_core::error::{ChakraError, QueryError, Result};
use chakra_core::result::Row;
use chakra_core::sql::{Dialect, PostgresDialect, SqlFragment};
use chakra_core::types::Value;
use chakra_migrate::executor::SqlExecutor;
use std::sync::Arc;
use tokio_postgres::types::ToSql;
use tracing::{debug, error};

/// PostgreSQL query executor
pub struct PostgresExecutor {
    pool: Arc<PostgresPool>,
    dialect: PostgresDialect,
}

impl PostgresExecutor {
    /// Create a new executor
    pub fn new(pool: Arc<PostgresPool>) -> Self {
        Self {
            pool,
            dialect: PostgresDialect,
        }
    }

    /// Get the dialect
    pub fn dialect(&self) -> &PostgresDialect {
        &self.dialect
    }

    /// Execute a query and return rows
    pub async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>> {
        let conn = self.pool.get().await?;

        debug!("Executing query: {} with {} params", sql, params.len());

        let pg_params: Vec<Box<dyn ToSql + Sync + Send>> =
            params.iter().map(to_postgres_param).collect();

        let param_refs: Vec<&(dyn ToSql + Sync)> =
            pg_params.iter().map(|p| p.as_ref() as &(dyn ToSql + Sync)).collect();

        let rows = conn
            .client
            .query(sql, &param_refs)
            .await
            .map_err(|e| {
                error!("Query failed: {}", e);
                ChakraError::Query(QueryError::ExecutionFailed {
                    message: e.to_string(),
                })
            })?;

        Ok(rows.iter().map(row_from_postgres).collect())
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
        let conn = self.pool.get().await?;

        debug!("Executing statement: {} with {} params", sql, params.len());

        let pg_params: Vec<Box<dyn ToSql + Sync + Send>> =
            params.iter().map(to_postgres_param).collect();

        let param_refs: Vec<&(dyn ToSql + Sync)> =
            pg_params.iter().map(|p| p.as_ref() as &(dyn ToSql + Sync)).collect();

        let result = conn
            .client
            .execute(sql, &param_refs)
            .await
            .map_err(|e| {
                error!("Statement failed: {}", e);
                ChakraError::Query(QueryError::ExecutionFailed {
                    message: e.to_string(),
                })
            })?;

        Ok(result)
    }

    /// Execute a statement with a SqlFragment
    pub async fn execute_fragment(&self, fragment: &SqlFragment) -> Result<u64> {
        self.execute(&fragment.sql, &fragment.params).await
    }

    /// Execute multiple statements in a batch
    pub async fn execute_batch(&self, statements: &[&str]) -> Result<()> {
        let conn = self.pool.get().await?;

        for sql in statements {
            conn.client.batch_execute(sql).await.map_err(|e| {
                ChakraError::Query(QueryError::ExecutionFailed {
                    message: e.to_string(),
                })
            })?;
        }

        Ok(())
    }

    /// Begin a transaction
    pub async fn begin(&self) -> Result<PostgresTransaction> {
        let conn = self.pool.get().await?;

        conn.client
            .batch_execute("BEGIN")
            .await
            .map_err(|e| {
                ChakraError::Query(QueryError::ExecutionFailed {
                    message: format!("Failed to begin transaction: {}", e),
                })
            })?;

        Ok(PostgresTransaction {
            executor: self,
            committed: false,
        })
    }
}

/// A PostgreSQL transaction
pub struct PostgresTransaction<'a> {
    executor: &'a PostgresExecutor,
    committed: bool,
}

impl<'a> PostgresTransaction<'a> {
    /// Execute a query within the transaction
    pub async fn query(&self, sql: &str, params: &[Value]) -> Result<Vec<Row>> {
        self.executor.query(sql, params).await
    }

    /// Execute a statement within the transaction
    pub async fn execute(&self, sql: &str, params: &[Value]) -> Result<u64> {
        self.executor.execute(sql, params).await
    }

    /// Commit the transaction
    pub async fn commit(mut self) -> Result<()> {
        let conn = self.executor.pool.get().await?;

        conn.client
            .batch_execute("COMMIT")
            .await
            .map_err(|e| {
                ChakraError::Query(QueryError::ExecutionFailed {
                    message: format!("Failed to commit transaction: {}", e),
                })
            })?;

        self.committed = true;
        Ok(())
    }

    /// Rollback the transaction
    pub async fn rollback(mut self) -> Result<()> {
        let conn = self.executor.pool.get().await?;

        conn.client
            .batch_execute("ROLLBACK")
            .await
            .map_err(|e| {
                ChakraError::Query(QueryError::ExecutionFailed {
                    message: format!("Failed to rollback transaction: {}", e),
                })
            })?;

        self.committed = true; // Prevent rollback in drop
        Ok(())
    }
}

impl<'a> Drop for PostgresTransaction<'a> {
    fn drop(&mut self) {
        if !self.committed {
            // Transaction wasn't committed, will be rolled back by database
            debug!("Transaction dropped without commit, will be rolled back");
        }
    }
}

#[async_trait]
impl SqlExecutor for PostgresExecutor {
    async fn execute(&self, sql: &str) -> Result<u64> {
        self.execute(sql, &[]).await
    }

    async fn execute_in_transaction(&self, statements: &[&str]) -> Result<Vec<u64>> {
        let conn = self.pool.get().await?;

        conn.client.batch_execute("BEGIN").await.map_err(|e| {
            ChakraError::Query(QueryError::ExecutionFailed {
                message: e.to_string(),
            })
        })?;

        let mut results = Vec::new();

        for sql in statements {
            match conn.client.execute(*sql, &[]).await {
                Ok(count) => results.push(count),
                Err(e) => {
                    conn.client.batch_execute("ROLLBACK").await.ok();
                    return Err(ChakraError::Query(QueryError::ExecutionFailed {
                        message: e.to_string(),
                    }));
                }
            }
        }

        conn.client.batch_execute("COMMIT").await.map_err(|e| {
            ChakraError::Query(QueryError::ExecutionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(results)
    }

    async fn begin_transaction(&self) -> Result<()> {
        let conn = self.pool.get().await?;
        conn.client.batch_execute("BEGIN").await.map_err(|e| {
            ChakraError::Query(QueryError::ExecutionFailed {
                message: e.to_string(),
            })
        })?;
        Ok(())
    }

    async fn commit_transaction(&self) -> Result<()> {
        let conn = self.pool.get().await?;
        conn.client.batch_execute("COMMIT").await.map_err(|e| {
            ChakraError::Query(QueryError::ExecutionFailed {
                message: e.to_string(),
            })
        })?;
        Ok(())
    }

    async fn rollback_transaction(&self) -> Result<()> {
        let conn = self.pool.get().await?;
        conn.client.batch_execute("ROLLBACK").await.map_err(|e| {
            ChakraError::Query(QueryError::ExecutionFailed {
                message: e.to_string(),
            })
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would require a running PostgreSQL instance
}
