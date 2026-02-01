//! SQLite adapter for Chakra ORM
//!
//! This crate provides:
//! - SQLite connection management
//! - Query execution
//! - Schema introspection
//! - Transaction support

pub mod config;
pub mod connection;
pub mod executor;
pub mod types;

pub use config::SqliteConfig;
pub use connection::SqliteConnection;
pub use executor::SqliteExecutor;

use chakra_core::error::Result;

/// Create a SQLite connection
pub async fn connect(config: SqliteConfig) -> Result<SqliteConnection> {
    SqliteConnection::open(config).await
}

/// Create an in-memory SQLite connection
pub async fn connect_memory() -> Result<SqliteConnection> {
    SqliteConnection::open_memory().await
}
