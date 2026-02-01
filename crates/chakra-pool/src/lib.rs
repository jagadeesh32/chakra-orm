//! Connection pooling for Chakra ORM
//!
//! This crate provides:
//! - Generic connection pool implementation
//! - Connection lifecycle management
//! - Health checking and validation
//! - Pool metrics and monitoring

pub mod config;
pub mod manager;
pub mod metrics;
pub mod pool;

pub use config::PoolConfig;
pub use manager::ConnectionManager;
pub use metrics::PoolMetrics;
pub use pool::{Pool, PooledConnection};
