//! SQLite configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// SQLite connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig {
    /// Database file path
    pub path: PathBuf,
    /// Create database if it doesn't exist
    pub create_if_missing: bool,
    /// Open in read-only mode
    pub read_only: bool,
    /// Enable WAL mode
    pub wal_mode: bool,
    /// Busy timeout in milliseconds
    pub busy_timeout_ms: u32,
    /// Enable foreign keys
    pub foreign_keys: bool,
}

impl SqliteConfig {
    /// Create a new config for a file database
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            create_if_missing: true,
            read_only: false,
            wal_mode: true,
            busy_timeout_ms: 5000,
            foreign_keys: true,
        }
    }

    /// Create a config for an in-memory database
    pub fn memory() -> Self {
        Self {
            path: PathBuf::from(":memory:"),
            create_if_missing: true,
            read_only: false,
            wal_mode: false,
            busy_timeout_ms: 5000,
            foreign_keys: true,
        }
    }

    /// Set read-only mode
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Set WAL mode
    pub fn wal_mode(mut self, wal: bool) -> Self {
        self.wal_mode = wal;
        self
    }

    /// Set busy timeout
    pub fn busy_timeout(mut self, ms: u32) -> Self {
        self.busy_timeout_ms = ms;
        self
    }

    /// Set foreign keys
    pub fn foreign_keys(mut self, enabled: bool) -> Self {
        self.foreign_keys = enabled;
        self
    }

    /// Check if this is an in-memory database
    pub fn is_memory(&self) -> bool {
        self.path.to_string_lossy() == ":memory:"
    }
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self::memory()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_config() {
        let config = SqliteConfig::memory();
        assert!(config.is_memory());
        assert!(!config.wal_mode);
    }

    #[test]
    fn test_file_config() {
        let config = SqliteConfig::new("test.db");
        assert!(!config.is_memory());
        assert!(config.wal_mode);
    }
}
