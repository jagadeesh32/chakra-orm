//! Connection pool implementation for Chakra ORM
//!
//! This module provides the core connection pool.

use crate::config::PoolConfig;
use crate::manager::{ConnectionManager, ConnectionState, ManagedConnection};
use crate::metrics::PoolMetrics;
use chakra_core::error::{ChakraError, Result};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Semaphore, SemaphorePermit};
use tracing::{debug, error, info, trace, warn};

/// A connection pool
pub struct Pool<M: ConnectionManager> {
    /// The connection manager
    manager: Arc<M>,
    /// Pool configuration
    config: PoolConfig,
    /// Available connections
    connections: Mutex<VecDeque<ManagedConnection<M::Connection>>>,
    /// Semaphore to limit concurrent connections
    semaphore: Arc<Semaphore>,
    /// Pool metrics
    metrics: Arc<PoolMetrics>,
    /// Next connection ID
    next_id: AtomicU64,
    /// Whether the pool is closed
    closed: std::sync::atomic::AtomicBool,
}

impl<M: ConnectionManager + 'static> Pool<M> {
    /// Create a new connection pool
    pub async fn new(manager: M, config: PoolConfig) -> Result<Arc<Self>> {
        config.validate().map_err(|e| {
            ChakraError::Connection(chakra_core::error::ConnectionError::Configuration {
                message: e.to_string(),
            })
        })?;

        let pool = Arc::new(Self {
            manager: Arc::new(manager),
            semaphore: Arc::new(Semaphore::new(config.max_connections as usize)),
            connections: Mutex::new(VecDeque::new()),
            metrics: Arc::new(PoolMetrics::new()),
            next_id: AtomicU64::new(1),
            closed: std::sync::atomic::AtomicBool::new(false),
            config,
        });

        // Initialize minimum connections
        pool.initialize_connections().await?;

        // Start background maintenance task
        pool.start_maintenance_task();

        info!(
            "Pool created with min={}, max={} connections",
            pool.config.min_connections, pool.config.max_connections
        );

        Ok(pool)
    }

    /// Initialize minimum number of connections
    async fn initialize_connections(self: &Arc<Self>) -> Result<()> {
        for _ in 0..self.config.min_connections {
            match self.create_connection().await {
                Ok(conn) => {
                    self.connections.lock().push_back(conn);
                    self.metrics.set_idle_connections(
                        self.connections.lock().len() as u64,
                    );
                }
                Err(e) => {
                    warn!("Failed to create initial connection: {}", e);
                }
            }
        }
        Ok(())
    }

    /// Create a new connection
    async fn create_connection(&self) -> Result<ManagedConnection<M::Connection>> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let conn = self.manager.connect().await?;
        self.metrics.record_connection_created();
        debug!(connection_id = id, "Created new connection");
        Ok(ManagedConnection::new(conn, id))
    }

    /// Start the background maintenance task
    fn start_maintenance_task(self: &Arc<Self>) {
        let pool = Arc::clone(self);
        let interval = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            loop {
                interval_timer.tick().await;

                if pool.is_closed() {
                    break;
                }

                pool.run_maintenance().await;
            }
        });
    }

    /// Run pool maintenance (cleanup idle connections, health checks)
    async fn run_maintenance(&self) {
        trace!("Running pool maintenance");

        // Collect connections - separate expired from those to check
        let (expired_connections, connections_to_check): (Vec<_>, Vec<_>) = {
            let mut connections = self.connections.lock();
            let mut expired = Vec::new();
            let mut to_check = Vec::new();

            while let Some(conn) = connections.pop_front() {
                // Check if connection has expired
                if self.is_connection_expired(&conn) {
                    debug!(
                        connection_id = conn.id,
                        "Connection expired, will close"
                    );
                    expired.push(conn);
                } else {
                    to_check.push(conn);
                }
            }

            (expired, to_check)
        };
        // MutexGuard is dropped here before any await

        // Close expired connections
        for conn in expired_connections {
            if let Err(e) = self.manager.close(conn.connection).await {
                error!("Failed to close expired connection: {}", e);
            }
            self.metrics.record_connection_closed();
        }

        // Validate connections
        for mut conn in connections_to_check {
            let is_valid = self.manager.is_valid(&conn.connection).await;
            self.metrics.record_validation(is_valid);

            if is_valid {
                self.connections.lock().push_back(conn);
            } else {
                debug!(
                    connection_id = conn.id,
                    "Connection failed validation, closing"
                );
                if let Err(e) = self.manager.close(conn.connection).await {
                    error!("Failed to close invalid connection: {}", e);
                }
                self.metrics.record_connection_closed();
            }
        }

        // Update metrics
        self.metrics
            .set_idle_connections(self.connections.lock().len() as u64);

        // Ensure minimum connections
        self.ensure_minimum_connections().await;
    }

    /// Ensure we have at least min_connections
    async fn ensure_minimum_connections(&self) {
        let current = self.connections.lock().len() as u32;
        if current < self.config.min_connections {
            let needed = self.config.min_connections - current;
            for _ in 0..needed {
                match self.create_connection().await {
                    Ok(conn) => {
                        self.connections.lock().push_back(conn);
                    }
                    Err(e) => {
                        warn!("Failed to create connection for minimum pool: {}", e);
                        break;
                    }
                }
            }
        }
    }

    /// Check if a connection has expired
    fn is_connection_expired(&self, conn: &ManagedConnection<M::Connection>) -> bool {
        // Check max lifetime
        if let Some(max_lifetime) = self.config.max_lifetime {
            if conn.age() > max_lifetime {
                return true;
            }
        }

        // Check idle timeout
        if let Some(idle_timeout) = self.config.idle_timeout {
            if conn.idle_time() > idle_timeout {
                return true;
            }
        }

        // Check manager-specific expiration
        self.manager.has_expired(&conn.connection)
    }

    /// Acquire a connection from the pool
    pub async fn acquire(self: &Arc<Self>) -> Result<PooledConnection<M>> {
        if self.is_closed() {
            return Err(ChakraError::Connection(
                chakra_core::error::ConnectionError::PoolClosed,
            ));
        }

        let start = Instant::now();

        // Acquire semaphore permit with timeout
        let permit = tokio::time::timeout(
            self.config.acquire_timeout,
            self.semaphore.clone().acquire_owned(),
        )
        .await
        .map_err(|_| {
            self.metrics.record_acquire_timeout();
            ChakraError::Connection(chakra_core::error::ConnectionError::PoolTimeout {
                timeout: self.config.acquire_timeout,
            })
        })?
        .map_err(|_| {
            ChakraError::Connection(chakra_core::error::ConnectionError::PoolClosed)
        })?;

        // Try to get an existing connection
        let conn = loop {
            let conn = self.connections.lock().pop_front();

            match conn {
                Some(mut conn) => {
                    // Validate if configured
                    if self.config.test_on_checkout {
                        if !self.manager.is_valid(&conn.connection).await {
                            self.metrics.record_validation(false);
                            if let Err(e) = self.manager.close(conn.connection).await {
                                error!("Failed to close invalid connection: {}", e);
                            }
                            self.metrics.record_connection_closed();
                            continue;
                        }
                        self.metrics.record_validation(true);
                    }

                    // Run on_acquire hook
                    if let Err(e) = self.manager.on_acquire(&mut conn.connection).await {
                        warn!("on_acquire failed: {}", e);
                        if let Err(e) = self.manager.close(conn.connection).await {
                            error!("Failed to close connection: {}", e);
                        }
                        self.metrics.record_connection_closed();
                        continue;
                    }

                    conn.mark_used();
                    break conn;
                }
                None => {
                    // Create a new connection
                    break self.create_connection().await?;
                }
            }
        };

        let wait_time = start.elapsed();
        self.metrics.record_acquire_success(wait_time);

        debug!(
            connection_id = conn.id,
            wait_ms = wait_time.as_millis(),
            "Connection acquired"
        );

        Ok(PooledConnection {
            pool: Arc::clone(self),
            connection: Some(conn),
            permit: Some(permit),
        })
    }

    /// Release a connection back to the pool
    async fn release(&self, mut conn: ManagedConnection<M::Connection>) {
        // Check if pool is closed
        if self.is_closed() {
            if let Err(e) = self.manager.close(conn.connection).await {
                error!("Failed to close connection on pool shutdown: {}", e);
            }
            self.metrics.record_connection_closed();
            return;
        }

        // Validate if configured
        if self.config.test_on_checkin {
            if !self.manager.is_valid(&conn.connection).await {
                self.metrics.record_validation(false);
                if let Err(e) = self.manager.close(conn.connection).await {
                    error!("Failed to close invalid connection: {}", e);
                }
                self.metrics.record_connection_closed();
                return;
            }
            self.metrics.record_validation(true);
        }

        // Reset connection state
        if let Err(e) = self.manager.reset(&mut conn.connection).await {
            warn!("Failed to reset connection: {}", e);
            if let Err(e) = self.manager.close(conn.connection).await {
                error!("Failed to close connection: {}", e);
            }
            self.metrics.record_connection_closed();
            return;
        }

        // Run on_release hook
        if let Err(e) = self.manager.on_release(&mut conn.connection).await {
            warn!("on_release failed: {}", e);
            if let Err(e) = self.manager.close(conn.connection).await {
                error!("Failed to close connection: {}", e);
            }
            self.metrics.record_connection_closed();
            return;
        }

        // Return to pool
        self.connections.lock().push_back(conn);
        self.metrics.record_release();

        trace!("Connection released back to pool");
    }

    /// Get pool metrics
    pub fn metrics(&self) -> &PoolMetrics {
        &self.metrics
    }

    /// Get current pool status
    pub fn status(&self) -> PoolStatus {
        let idle = self.connections.lock().len() as u32;
        let available_permits = self.semaphore.available_permits() as u32;
        let in_use = self.config.max_connections - available_permits;

        PoolStatus {
            idle_connections: idle,
            in_use_connections: in_use,
            max_connections: self.config.max_connections,
            is_closed: self.is_closed(),
        }
    }

    /// Check if the pool is closed
    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::Relaxed)
    }

    /// Close the pool
    pub async fn close(&self) {
        if self
            .closed
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return; // Already closed
        }

        info!("Closing connection pool");

        // Close all idle connections
        let connections: Vec<_> = {
            let mut lock = self.connections.lock();
            lock.drain(..).collect()
        };

        for conn in connections {
            if let Err(e) = self.manager.close(conn.connection).await {
                error!("Failed to close connection: {}", e);
            }
            self.metrics.record_connection_closed();
        }

        info!("Connection pool closed");
    }
}

/// Pool status
#[derive(Debug, Clone)]
pub struct PoolStatus {
    pub idle_connections: u32,
    pub in_use_connections: u32,
    pub max_connections: u32,
    pub is_closed: bool,
}

/// A pooled connection that returns to the pool when dropped
pub struct PooledConnection<M: ConnectionManager + 'static> {
    pool: Arc<Pool<M>>,
    connection: Option<ManagedConnection<M::Connection>>,
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
}

impl<M: ConnectionManager + 'static> PooledConnection<M> {
    /// Get the connection ID
    pub fn id(&self) -> u64 {
        self.connection.as_ref().map(|c| c.id).unwrap_or(0)
    }

    /// Get connection age
    pub fn age(&self) -> Duration {
        self.connection
            .as_ref()
            .map(|c| c.age())
            .unwrap_or(Duration::ZERO)
    }

    /// Get use count
    pub fn use_count(&self) -> u64 {
        self.connection.as_ref().map(|c| c.use_count).unwrap_or(0)
    }

    /// Detach the connection from the pool (it won't be returned)
    pub fn detach(mut self) -> Option<M::Connection> {
        self.connection.take().map(|c| c.connection)
    }
}

impl<M: ConnectionManager + 'static> Deref for PooledConnection<M> {
    type Target = M::Connection;

    fn deref(&self) -> &Self::Target {
        &self
            .connection
            .as_ref()
            .expect("connection already taken")
            .connection
    }
}

impl<M: ConnectionManager + 'static> DerefMut for PooledConnection<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self
            .connection
            .as_mut()
            .expect("connection already taken")
            .connection
    }
}

impl<M: ConnectionManager + 'static> Drop for PooledConnection<M> {
    fn drop(&mut self) {
        if let Some(conn) = self.connection.take() {
            let pool = Arc::clone(&self.pool);
            // Spawn a task to release the connection
            tokio::spawn(async move {
                pool.release(conn).await;
            });
        }
        // Permit is automatically released when dropped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock connection manager for testing
    #[derive(Debug)]
    struct MockManager;

    #[async_trait::async_trait]
    impl ConnectionManager for MockManager {
        type Connection = u64;

        async fn connect(&self) -> Result<Self::Connection> {
            Ok(rand::random())
        }

        async fn is_valid(&self, _conn: &Self::Connection) -> bool {
            true
        }

        fn has_expired(&self, _conn: &Self::Connection) -> bool {
            false
        }

        async fn reset(&self, _conn: &mut Self::Connection) -> Result<()> {
            Ok(())
        }

        async fn close(&self, _conn: Self::Connection) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_pool_creation() {
        let config = PoolConfig::new("test://localhost")
            .min_connections(2)
            .max_connections(5);

        let pool = Pool::new(MockManager, config).await.unwrap();
        let status = pool.status();

        assert_eq!(status.idle_connections, 2);
        assert_eq!(status.max_connections, 5);
        assert!(!status.is_closed);
    }

    #[tokio::test]
    async fn test_acquire_release() {
        let config = PoolConfig::new("test://localhost")
            .min_connections(1)
            .max_connections(2);

        let pool = Pool::new(MockManager, config).await.unwrap();

        {
            let conn = pool.acquire().await.unwrap();
            assert!(conn.id() > 0);
        }

        // Connection should be released after drop
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
