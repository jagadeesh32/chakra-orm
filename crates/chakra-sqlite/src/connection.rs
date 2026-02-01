//! SQLite connection management

use crate::config::SqliteConfig;
use chakra_core::error::{ChakraError, ConnectionError, Result};
use std::sync::Arc;
use tokio_rusqlite::Connection;
use tracing::{debug, info};

/// A SQLite connection
pub struct SqliteConnection {
    conn: Connection,
    config: SqliteConfig,
}

impl SqliteConnection {
    /// Open a connection with the given config
    pub async fn open(config: SqliteConfig) -> Result<Self> {
        let path = config.path.clone();
        let create = config.create_if_missing;
        let read_only = config.read_only;

        let conn = if config.is_memory() {
            Connection::open_in_memory().await
        } else {
            Connection::open(&path).await
        }
        .map_err(|e| {
            ChakraError::Connection(ConnectionError::ConnectionFailed {
                message: format!("Failed to open SQLite database: {}", e),
            })
        })?;

        // Configure the connection
        let wal_mode = config.wal_mode;
        let foreign_keys = config.foreign_keys;
        let busy_timeout = config.busy_timeout_ms;

        conn.call(move |conn| {
            // Set busy timeout
            conn.busy_timeout(std::time::Duration::from_millis(busy_timeout as u64))?;

            // Enable foreign keys
            if foreign_keys {
                conn.execute_batch("PRAGMA foreign_keys = ON;")?;
            }

            // Enable WAL mode (only for file databases)
            if wal_mode && path.to_string_lossy() != ":memory:" {
                conn.execute_batch("PRAGMA journal_mode = WAL;")?;
            }

            Ok(())
        })
        .await
        .map_err(|e| {
            ChakraError::Connection(ConnectionError::ConnectionFailed {
                message: format!("Failed to configure SQLite: {}", e),
            })
        })?;

        info!("SQLite connection opened: {:?}", config.path);

        Ok(Self { conn, config })
    }

    /// Open an in-memory connection
    pub async fn open_memory() -> Result<Self> {
        Self::open(SqliteConfig::memory()).await
    }

    /// Get the underlying connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Get the configuration
    pub fn config(&self) -> &SqliteConfig {
        &self.config
    }

    /// Execute a function on the connection
    pub async fn call<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut rusqlite::Connection) -> std::result::Result<R, rusqlite::Error>
            + Send
            + 'static,
        R: Send + 'static,
    {
        self.conn
            .call(move |conn| f(conn).map_err(tokio_rusqlite::Error::from))
            .await
            .map_err(|e| ChakraError::internal(format!("SQLite call failed: {}", e)))
    }

    /// Close the connection
    pub async fn close(self) -> Result<()> {
        self.conn.close().await.map_err(|e| {
            ChakraError::Connection(ConnectionError::ConnectionFailed {
                message: format!("Failed to close SQLite connection: {:?}", e),
            })
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_open_memory() {
        let conn = SqliteConnection::open_memory().await.unwrap();
        assert!(conn.config().is_memory());
    }

    #[tokio::test]
    async fn test_execute_query() {
        let conn = SqliteConnection::open_memory().await.unwrap();

        conn.call(|c| {
            c.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)", [])?;
            c.execute("INSERT INTO test (name) VALUES (?)", ["Alice"])?;
            Ok(())
        })
        .await
        .unwrap();

        let name: String = conn
            .call(|c| {
                c.query_row("SELECT name FROM test WHERE id = 1", [], |row| row.get(0))
            })
            .await
            .unwrap();

        assert_eq!(name, "Alice");
    }
}
