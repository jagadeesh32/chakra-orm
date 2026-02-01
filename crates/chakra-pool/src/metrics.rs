//! Pool metrics for Chakra ORM
//!
//! This module provides metrics collection for the connection pool.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Pool metrics
#[derive(Debug, Default)]
pub struct PoolMetrics {
    /// Total connections created
    pub connections_created: AtomicU64,
    /// Total connections closed
    pub connections_closed: AtomicU64,
    /// Total connection acquires
    pub acquires_total: AtomicU64,
    /// Successful connection acquires
    pub acquires_success: AtomicU64,
    /// Failed connection acquires (timeout)
    pub acquires_timeout: AtomicU64,
    /// Total connection releases
    pub releases_total: AtomicU64,
    /// Total validations performed
    pub validations_total: AtomicU64,
    /// Failed validations
    pub validations_failed: AtomicU64,
    /// Current idle connections
    pub idle_connections: AtomicU64,
    /// Current in-use connections
    pub in_use_connections: AtomicU64,
    /// Total acquire wait time in microseconds
    pub total_acquire_wait_us: AtomicU64,
    /// Maximum acquire wait time in microseconds
    pub max_acquire_wait_us: AtomicU64,
}

impl PoolMetrics {
    /// Create new metrics
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a connection creation
    pub fn record_connection_created(&self) {
        self.connections_created.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a connection close
    pub fn record_connection_closed(&self) {
        self.connections_closed.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a successful acquire
    pub fn record_acquire_success(&self, wait_time: Duration) {
        self.acquires_total.fetch_add(1, Ordering::Relaxed);
        self.acquires_success.fetch_add(1, Ordering::Relaxed);
        self.idle_connections.fetch_sub(1, Ordering::Relaxed);
        self.in_use_connections.fetch_add(1, Ordering::Relaxed);

        let wait_us = wait_time.as_micros() as u64;
        self.total_acquire_wait_us
            .fetch_add(wait_us, Ordering::Relaxed);

        // Update max (not atomic, but close enough for metrics)
        let current_max = self.max_acquire_wait_us.load(Ordering::Relaxed);
        if wait_us > current_max {
            self.max_acquire_wait_us.store(wait_us, Ordering::Relaxed);
        }
    }

    /// Record a failed acquire (timeout)
    pub fn record_acquire_timeout(&self) {
        self.acquires_total.fetch_add(1, Ordering::Relaxed);
        self.acquires_timeout.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a connection release
    pub fn record_release(&self) {
        self.releases_total.fetch_add(1, Ordering::Relaxed);
        self.in_use_connections.fetch_sub(1, Ordering::Relaxed);
        self.idle_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a validation
    pub fn record_validation(&self, success: bool) {
        self.validations_total.fetch_add(1, Ordering::Relaxed);
        if !success {
            self.validations_failed.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Set idle connection count
    pub fn set_idle_connections(&self, count: u64) {
        self.idle_connections.store(count, Ordering::Relaxed);
    }

    /// Set in-use connection count
    pub fn set_in_use_connections(&self, count: u64) {
        self.in_use_connections.store(count, Ordering::Relaxed);
    }

    /// Get snapshot of metrics
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            connections_created: self.connections_created.load(Ordering::Relaxed),
            connections_closed: self.connections_closed.load(Ordering::Relaxed),
            acquires_total: self.acquires_total.load(Ordering::Relaxed),
            acquires_success: self.acquires_success.load(Ordering::Relaxed),
            acquires_timeout: self.acquires_timeout.load(Ordering::Relaxed),
            releases_total: self.releases_total.load(Ordering::Relaxed),
            validations_total: self.validations_total.load(Ordering::Relaxed),
            validations_failed: self.validations_failed.load(Ordering::Relaxed),
            idle_connections: self.idle_connections.load(Ordering::Relaxed),
            in_use_connections: self.in_use_connections.load(Ordering::Relaxed),
            avg_acquire_wait: self.average_acquire_wait(),
            max_acquire_wait: Duration::from_micros(
                self.max_acquire_wait_us.load(Ordering::Relaxed),
            ),
        }
    }

    /// Get average acquire wait time
    pub fn average_acquire_wait(&self) -> Duration {
        let total = self.total_acquire_wait_us.load(Ordering::Relaxed);
        let count = self.acquires_success.load(Ordering::Relaxed);
        if count > 0 {
            Duration::from_micros(total / count)
        } else {
            Duration::ZERO
        }
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.connections_created.store(0, Ordering::Relaxed);
        self.connections_closed.store(0, Ordering::Relaxed);
        self.acquires_total.store(0, Ordering::Relaxed);
        self.acquires_success.store(0, Ordering::Relaxed);
        self.acquires_timeout.store(0, Ordering::Relaxed);
        self.releases_total.store(0, Ordering::Relaxed);
        self.validations_total.store(0, Ordering::Relaxed);
        self.validations_failed.store(0, Ordering::Relaxed);
        self.total_acquire_wait_us.store(0, Ordering::Relaxed);
        self.max_acquire_wait_us.store(0, Ordering::Relaxed);
    }
}

/// Snapshot of pool metrics at a point in time
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub connections_created: u64,
    pub connections_closed: u64,
    pub acquires_total: u64,
    pub acquires_success: u64,
    pub acquires_timeout: u64,
    pub releases_total: u64,
    pub validations_total: u64,
    pub validations_failed: u64,
    pub idle_connections: u64,
    pub in_use_connections: u64,
    pub avg_acquire_wait: Duration,
    pub max_acquire_wait: Duration,
}

impl MetricsSnapshot {
    /// Get total connection count
    pub fn total_connections(&self) -> u64 {
        self.idle_connections + self.in_use_connections
    }

    /// Get pool utilization (0.0 - 1.0)
    pub fn utilization(&self) -> f64 {
        let total = self.total_connections();
        if total > 0 {
            self.in_use_connections as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get success rate (0.0 - 1.0)
    pub fn success_rate(&self) -> f64 {
        if self.acquires_total > 0 {
            self.acquires_success as f64 / self.acquires_total as f64
        } else {
            1.0
        }
    }

    /// Get validation failure rate (0.0 - 1.0)
    pub fn validation_failure_rate(&self) -> f64 {
        if self.validations_total > 0 {
            self.validations_failed as f64 / self.validations_total as f64
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_recording() {
        let metrics = PoolMetrics::new();

        metrics.record_connection_created();
        metrics.record_acquire_success(Duration::from_millis(5));
        metrics.record_release();

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.connections_created, 1);
        assert_eq!(snapshot.acquires_success, 1);
        assert_eq!(snapshot.releases_total, 1);
    }

    #[test]
    fn test_utilization() {
        let metrics = PoolMetrics::new();
        metrics.set_idle_connections(5);
        metrics.set_in_use_connections(5);

        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.utilization(), 0.5);
    }
}
