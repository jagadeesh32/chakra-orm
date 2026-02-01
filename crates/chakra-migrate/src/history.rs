//! Migration history tracking

use crate::migration::{MigrationDirection, MigrationStatus};
use async_trait::async_trait;
use chakra_core::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A record of a migration that was applied
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationRecord {
    /// Migration ID
    pub id: String,
    /// Migration name
    pub name: String,
    /// App/module name
    pub app: Option<String>,
    /// Status
    pub status: MigrationStatus,
    /// Checksum when applied
    pub checksum: String,
    /// When the migration was applied
    pub applied_at: DateTime<Utc>,
    /// How long it took in milliseconds
    pub duration_ms: u64,
    /// Number of statements executed
    pub statements_count: usize,
    /// Error message if failed
    pub error_message: Option<String>,
}

impl MigrationRecord {
    /// Create a new record
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            app: None,
            status: MigrationStatus::Pending,
            checksum: String::new(),
            applied_at: Utc::now(),
            duration_ms: 0,
            statements_count: 0,
            error_message: None,
        }
    }

    /// Mark as applied
    pub fn applied(mut self, duration_ms: u64, statements_count: usize) -> Self {
        self.status = MigrationStatus::Applied;
        self.duration_ms = duration_ms;
        self.statements_count = statements_count;
        self.applied_at = Utc::now();
        self
    }

    /// Mark as failed
    pub fn failed(mut self, error: impl Into<String>) -> Self {
        self.status = MigrationStatus::Failed;
        self.error_message = Some(error.into());
        self.applied_at = Utc::now();
        self
    }
}

/// Trait for migration history storage
#[async_trait]
pub trait MigrationHistory: Send + Sync {
    /// Initialize the history storage (create table, etc.)
    async fn initialize(&self) -> Result<()>;

    /// Get all applied migrations
    async fn get_applied(&self) -> Result<Vec<MigrationRecord>>;

    /// Get a specific migration record
    async fn get(&self, migration_id: &str) -> Result<Option<MigrationRecord>>;

    /// Check if a migration has been applied
    async fn is_applied(&self, migration_id: &str) -> Result<bool>;

    /// Record a migration as applied
    async fn record_applied(&self, record: MigrationRecord) -> Result<()>;

    /// Record a migration as rolled back
    async fn record_rollback(&self, migration_id: &str) -> Result<()>;

    /// Get the last applied migration
    async fn last_applied(&self) -> Result<Option<MigrationRecord>>;

    /// Lock migrations (for concurrent safety)
    async fn acquire_lock(&self) -> Result<MigrationLock>;

    /// Release migrations lock
    async fn release_lock(&self, lock: MigrationLock) -> Result<()>;
}

/// A lock for migration operations
#[derive(Debug)]
pub struct MigrationLock {
    pub id: String,
    pub acquired_at: DateTime<Utc>,
}

impl MigrationLock {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            acquired_at: Utc::now(),
        }
    }
}

impl Default for MigrationLock {
    fn default() -> Self {
        Self::new()
    }
}

/// In-memory migration history (for testing)
#[derive(Debug, Default)]
pub struct InMemoryHistory {
    records: tokio::sync::RwLock<HashMap<String, MigrationRecord>>,
    locked: tokio::sync::RwLock<Option<String>>,
}

impl InMemoryHistory {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl MigrationHistory for InMemoryHistory {
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }

    async fn get_applied(&self) -> Result<Vec<MigrationRecord>> {
        let records = self.records.read().await;
        let mut applied: Vec<_> = records
            .values()
            .filter(|r| r.status == MigrationStatus::Applied)
            .cloned()
            .collect();
        applied.sort_by(|a, b| a.applied_at.cmp(&b.applied_at));
        Ok(applied)
    }

    async fn get(&self, migration_id: &str) -> Result<Option<MigrationRecord>> {
        let records = self.records.read().await;
        Ok(records.get(migration_id).cloned())
    }

    async fn is_applied(&self, migration_id: &str) -> Result<bool> {
        let records = self.records.read().await;
        Ok(records
            .get(migration_id)
            .map(|r| r.status == MigrationStatus::Applied)
            .unwrap_or(false))
    }

    async fn record_applied(&self, record: MigrationRecord) -> Result<()> {
        let mut records = self.records.write().await;
        records.insert(record.id.clone(), record);
        Ok(())
    }

    async fn record_rollback(&self, migration_id: &str) -> Result<()> {
        let mut records = self.records.write().await;
        if let Some(record) = records.get_mut(migration_id) {
            record.status = MigrationStatus::RolledBack;
        }
        Ok(())
    }

    async fn last_applied(&self) -> Result<Option<MigrationRecord>> {
        let applied = self.get_applied().await?;
        Ok(applied.last().cloned())
    }

    async fn acquire_lock(&self) -> Result<MigrationLock> {
        let mut locked = self.locked.write().await;
        if locked.is_some() {
            return Err(chakra_core::error::ChakraError::internal(
                "Migration lock already held",
            ));
        }
        let lock = MigrationLock::new();
        *locked = Some(lock.id.clone());
        Ok(lock)
    }

    async fn release_lock(&self, lock: MigrationLock) -> Result<()> {
        let mut locked = self.locked.write().await;
        if locked.as_ref() == Some(&lock.id) {
            *locked = None;
        }
        Ok(())
    }
}

/// SQL for creating the migration history table (PostgreSQL)
pub const POSTGRES_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS chakra_migrations (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    app VARCHAR(255),
    status VARCHAR(50) NOT NULL,
    checksum VARCHAR(64) NOT NULL,
    applied_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    duration_ms BIGINT NOT NULL DEFAULT 0,
    statements_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_chakra_migrations_applied_at
ON chakra_migrations(applied_at);

CREATE INDEX IF NOT EXISTS idx_chakra_migrations_status
ON chakra_migrations(status);
"#;

/// SQL for creating the migration history table (MySQL)
pub const MYSQL_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS chakra_migrations (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    app VARCHAR(255),
    status VARCHAR(50) NOT NULL,
    checksum VARCHAR(64) NOT NULL,
    applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    duration_ms BIGINT NOT NULL DEFAULT 0,
    statements_count INT NOT NULL DEFAULT 0,
    error_message TEXT
);

CREATE INDEX idx_chakra_migrations_applied_at
ON chakra_migrations(applied_at);

CREATE INDEX idx_chakra_migrations_status
ON chakra_migrations(status);
"#;

/// SQL for creating the migration history table (SQLite)
pub const SQLITE_HISTORY_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS chakra_migrations (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    app TEXT,
    status TEXT NOT NULL,
    checksum TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    duration_ms INTEGER NOT NULL DEFAULT 0,
    statements_count INTEGER NOT NULL DEFAULT 0,
    error_message TEXT
);

CREATE INDEX IF NOT EXISTS idx_chakra_migrations_applied_at
ON chakra_migrations(applied_at);

CREATE INDEX IF NOT EXISTS idx_chakra_migrations_status
ON chakra_migrations(status);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_history() {
        let history = InMemoryHistory::new();

        history.initialize().await.unwrap();

        let record = MigrationRecord::new("001", "test").applied(100, 5);
        history.record_applied(record).await.unwrap();

        assert!(history.is_applied("001").await.unwrap());
        assert!(!history.is_applied("002").await.unwrap());

        let applied = history.get_applied().await.unwrap();
        assert_eq!(applied.len(), 1);
    }

    #[tokio::test]
    async fn test_migration_lock() {
        let history = InMemoryHistory::new();

        let lock1 = history.acquire_lock().await.unwrap();
        assert!(history.acquire_lock().await.is_err());

        history.release_lock(lock1).await.unwrap();
        let _lock2 = history.acquire_lock().await.unwrap();
    }
}
