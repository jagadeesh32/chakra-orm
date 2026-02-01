//! Connection manager trait for Chakra ORM
//!
//! This module defines the trait that database adapters must implement
//! to participate in connection pooling.

use async_trait::async_trait;
use chakra_core::error::Result;
use std::fmt::Debug;

/// Trait for managing database connections
///
/// This trait must be implemented by database adapters to provide
/// connection creation, validation, and cleanup.
#[async_trait]
pub trait ConnectionManager: Send + Sync + Debug {
    /// The connection type this manager creates
    type Connection: Send + Sync;

    /// Create a new connection
    async fn connect(&self) -> Result<Self::Connection>;

    /// Check if a connection is still valid
    async fn is_valid(&self, conn: &Self::Connection) -> bool;

    /// Check if a connection has expired based on its metadata
    fn has_expired(&self, conn: &Self::Connection) -> bool;

    /// Prepare a connection for use (called before returning from pool)
    async fn on_acquire(&self, _conn: &mut Self::Connection) -> Result<()> {
        Ok(())
    }

    /// Clean up a connection before returning to pool
    async fn on_release(&self, _conn: &mut Self::Connection) -> Result<()> {
        Ok(())
    }

    /// Reset connection state (e.g., rollback any open transaction)
    async fn reset(&self, conn: &mut Self::Connection) -> Result<()>;

    /// Close a connection
    async fn close(&self, conn: Self::Connection) -> Result<()>;
}

/// Connection wrapper with metadata
#[derive(Debug)]
pub struct ManagedConnection<C> {
    /// The actual connection
    pub connection: C,
    /// When the connection was created
    pub created_at: std::time::Instant,
    /// When the connection was last used
    pub last_used_at: std::time::Instant,
    /// Number of times this connection has been used
    pub use_count: u64,
    /// Unique connection ID
    pub id: u64,
}

impl<C> ManagedConnection<C> {
    /// Create a new managed connection
    pub fn new(connection: C, id: u64) -> Self {
        let now = std::time::Instant::now();
        Self {
            connection,
            created_at: now,
            last_used_at: now,
            use_count: 0,
            id,
        }
    }

    /// Mark the connection as used
    pub fn mark_used(&mut self) {
        self.last_used_at = std::time::Instant::now();
        self.use_count += 1;
    }

    /// Get connection age
    pub fn age(&self) -> std::time::Duration {
        self.created_at.elapsed()
    }

    /// Get idle time
    pub fn idle_time(&self) -> std::time::Duration {
        self.last_used_at.elapsed()
    }
}

/// Connection state in the pool
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Available for use
    Idle,
    /// Currently in use
    InUse,
    /// Being validated
    Validating,
    /// Closed/invalid
    Closed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_managed_connection() {
        let conn = ManagedConnection::new("test_connection", 1);
        assert_eq!(conn.use_count, 0);
        assert_eq!(conn.id, 1);
    }

    #[test]
    fn test_mark_used() {
        let mut conn = ManagedConnection::new("test_connection", 1);
        conn.mark_used();
        assert_eq!(conn.use_count, 1);
        conn.mark_used();
        assert_eq!(conn.use_count, 2);
    }
}
