//! PostgreSQL connection and pool management

use crate::config::PostgresConfig;
use async_trait::async_trait;
use chakra_core::error::{ChakraError, ConnectionError, Result};
use chakra_pool::manager::ConnectionManager;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio_postgres::{Client, NoTls};
use tracing::{debug, error, info};

/// A PostgreSQL connection
pub struct PostgresConnection {
    /// The underlying client
    pub client: Client,
    /// When the connection was created
    pub created_at: Instant,
    /// Connection ID
    pub id: u64,
}

impl PostgresConnection {
    /// Create a new connection wrapper
    pub fn new(client: Client, id: u64) -> Self {
        Self {
            client,
            created_at: Instant::now(),
            id,
        }
    }

    /// Check if the connection is valid
    pub async fn is_valid(&self) -> bool {
        self.client.simple_query("SELECT 1").await.is_ok()
    }

    /// Get connection age
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }
}

/// PostgreSQL connection manager
#[derive(Debug)]
pub struct PostgresConnectionManager {
    config: PostgresConfig,
    next_id: AtomicU64,
}

impl PostgresConnectionManager {
    /// Create a new connection manager
    pub fn new(config: PostgresConfig) -> Self {
        Self {
            config,
            next_id: AtomicU64::new(1),
        }
    }
}

#[async_trait]
impl ConnectionManager for PostgresConnectionManager {
    type Connection = PostgresConnection;

    async fn connect(&self) -> Result<Self::Connection> {
        let conn_str = self.config.connection_string();
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        debug!(connection_id = id, "Creating PostgreSQL connection");

        let (client, connection) = tokio_postgres::connect(&conn_str, NoTls)
            .await
            .map_err(|e| {
                ChakraError::Connection(ConnectionError::ConnectionFailed {
                    message: e.to_string(),
                })
            })?;

        // Spawn the connection handler
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("PostgreSQL connection error: {}", e);
            }
        });

        // Set schema if specified
        if let Some(ref schema) = self.config.schema {
            client
                .simple_query(&format!("SET search_path TO {}", schema))
                .await
                .map_err(|e| {
                    ChakraError::Connection(ConnectionError::ConnectionFailed {
                        message: format!("Failed to set schema: {}", e),
                    })
                })?;
        }

        info!(connection_id = id, "PostgreSQL connection established");
        Ok(PostgresConnection::new(client, id))
    }

    async fn is_valid(&self, conn: &Self::Connection) -> bool {
        conn.is_valid().await
    }

    fn has_expired(&self, conn: &Self::Connection) -> bool {
        if let Some(max_lifetime) = self.config.pool.max_lifetime {
            conn.age() > max_lifetime
        } else {
            false
        }
    }

    async fn reset(&self, conn: &mut Self::Connection) -> Result<()> {
        // Reset session state
        conn.client
            .simple_query("DISCARD ALL")
            .await
            .map_err(|e| {
                ChakraError::Connection(ConnectionError::ConnectionFailed {
                    message: format!("Failed to reset connection: {}", e),
                })
            })?;

        // Re-set schema if needed
        if let Some(ref schema) = self.config.schema {
            conn.client
                .simple_query(&format!("SET search_path TO {}", schema))
                .await
                .map_err(|e| {
                    ChakraError::Connection(ConnectionError::ConnectionFailed {
                        message: format!("Failed to set schema: {}", e),
                    })
                })?;
        }

        Ok(())
    }

    async fn close(&self, conn: Self::Connection) -> Result<()> {
        debug!(connection_id = conn.id, "Closing PostgreSQL connection");
        // Connection is closed when dropped
        drop(conn);
        Ok(())
    }
}

/// PostgreSQL connection pool
pub struct PostgresPool {
    pool: Arc<chakra_pool::Pool<PostgresConnectionManager>>,
    config: PostgresConfig,
}

impl PostgresPool {
    /// Create a new connection pool
    pub async fn new(config: PostgresConfig) -> Result<Self> {
        let manager = PostgresConnectionManager::new(config.clone());

        let pool_config = chakra_pool::PoolConfig::new(&config.connection_string())
            .min_connections(config.pool.min_size as u32)
            .max_connections(config.pool.max_size as u32)
            .acquire_timeout(config.pool.connection_timeout)
            .idle_timeout(config.pool.idle_timeout)
            .max_lifetime(config.pool.max_lifetime);

        let pool = chakra_pool::Pool::new(manager, pool_config).await?;

        Ok(Self { pool, config })
    }

    /// Get a connection from the pool
    pub async fn get(&self) -> Result<chakra_pool::PooledConnection<PostgresConnectionManager>> {
        self.pool.acquire().await
    }

    /// Get pool status
    pub fn status(&self) -> chakra_pool::pool::PoolStatus {
        self.pool.status()
    }

    /// Get pool metrics
    pub fn metrics(&self) -> &chakra_pool::PoolMetrics {
        self.pool.metrics()
    }

    /// Close the pool
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Get the configuration
    pub fn config(&self) -> &PostgresConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_manager_creation() {
        let config = PostgresConfig::new("localhost", "test_db");
        let manager = PostgresConnectionManager::new(config);
        assert_eq!(manager.next_id.load(Ordering::Relaxed), 1);
    }
}
