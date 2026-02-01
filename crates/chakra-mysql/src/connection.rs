//! MySQL connection and pool management

use crate::config::MySqlConfig;
use chakra_core::error::{ChakraError, ConnectionError, Result};
use mysql_async::{prelude::*, Pool, PoolConstraints, PoolOpts};
use std::sync::Arc;
use tracing::{debug, info};

/// A MySQL connection pool
pub struct MySqlPool {
    pool: Pool,
    config: MySqlConfig,
}

impl MySqlPool {
    /// Create a new connection pool
    pub async fn new(config: MySqlConfig) -> Result<Self> {
        let pool_opts = PoolOpts::default()
            .with_constraints(
                PoolConstraints::new(config.pool_min, config.pool_max).unwrap()
            );

        let pool = Pool::new(
            mysql_async::OptsBuilder::from_opts(
                mysql_async::Opts::from_url(&config.connection_url())
                    .map_err(|e| ChakraError::Connection(ConnectionError::Configuration {
                        message: e.to_string(),
                    }))?
            ).pool_opts(pool_opts)
        );

        info!("MySQL connection pool created");

        Ok(Self { pool, config })
    }

    /// Get a connection from the pool
    pub async fn get(&self) -> Result<MySqlConnection> {
        let conn = self.pool.get_conn().await.map_err(|e| {
            ChakraError::Connection(ConnectionError::ConnectionFailed {
                message: e.to_string(),
            })
        })?;

        Ok(MySqlConnection { conn })
    }

    /// Disconnect the pool
    pub async fn disconnect(self) -> Result<()> {
        self.pool.disconnect().await.map_err(|e| {
            ChakraError::Connection(ConnectionError::ConnectionFailed {
                message: e.to_string(),
            })
        })?;
        Ok(())
    }

    /// Get the configuration
    pub fn config(&self) -> &MySqlConfig {
        &self.config
    }
}

/// A MySQL connection
pub struct MySqlConnection {
    conn: mysql_async::Conn,
}

impl MySqlConnection {
    /// Get the underlying connection
    pub fn inner(&mut self) -> &mut mysql_async::Conn {
        &mut self.conn
    }

    /// Execute a query
    pub async fn query<T, Q>(&mut self, query: Q) -> Result<Vec<T>>
    where
        Q: AsRef<str>,
        T: FromRow + Send + 'static,
    {
        self.conn
            .query(query.as_ref())
            .await
            .map_err(|e| ChakraError::internal(e.to_string()))
    }

    /// Execute a statement
    pub async fn exec<Q>(&mut self, query: Q) -> Result<()>
    where
        Q: AsRef<str>,
    {
        self.conn
            .query_drop(query.as_ref())
            .await
            .map_err(|e| ChakraError::internal(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = MySqlConfig::new("localhost", "test_db");
        assert_eq!(config.host, "localhost");
        assert_eq!(config.database, "test_db");
    }
}
