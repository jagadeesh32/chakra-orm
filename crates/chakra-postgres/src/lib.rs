//! PostgreSQL adapter for Chakra ORM
//!
//! This crate provides:
//! - PostgreSQL connection management
//! - Query execution
//! - Schema introspection
//! - Transaction support

pub mod config;
pub mod connection;
pub mod executor;
pub mod introspect;
pub mod types;

pub use config::PostgresConfig;
pub use connection::{PostgresConnection, PostgresPool};
pub use executor::PostgresExecutor;
pub use introspect::PostgresIntrospector;

use chakra_core::error::Result;
use tokio_postgres::Client;

/// Create a PostgreSQL connection pool
pub async fn connect(config: PostgresConfig) -> Result<PostgresPool> {
    PostgresPool::new(config).await
}
