//! MySQL/MariaDB adapter for Chakra ORM
//!
//! This crate provides:
//! - MySQL/MariaDB connection management
//! - Query execution
//! - Schema introspection
//! - Transaction support

pub mod config;
pub mod connection;
pub mod executor;
pub mod types;

pub use config::MySqlConfig;
pub use connection::{MySqlConnection, MySqlPool};
pub use executor::MySqlExecutor;

use chakra_core::error::Result;

/// Create a MySQL connection pool
pub async fn connect(config: MySqlConfig) -> Result<MySqlPool> {
    MySqlPool::new(config).await
}
