//! Migration types and definitions

use chakra_schema::diff::MigrationOperation;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A migration definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Migration {
    /// Unique migration ID (timestamp-based)
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of changes
    pub description: Option<String>,
    /// App/module this migration belongs to
    pub app: Option<String>,
    /// Dependencies (other migration IDs)
    pub dependencies: Vec<String>,
    /// Operations in this migration
    pub operations: Vec<MigrationOperation>,
    /// Whether this migration is reversible
    pub reversible: bool,
    /// Custom SQL for forward migration (if any)
    pub raw_sql_up: Option<String>,
    /// Custom SQL for reverse migration (if any)
    pub raw_sql_down: Option<String>,
    /// Checksum of the migration content
    pub checksum: String,
    /// When this migration was created
    pub created_at: DateTime<Utc>,
    /// Arbitrary metadata
    pub metadata: HashMap<String, String>,
}

impl Migration {
    /// Create a new migration
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            app: None,
            dependencies: Vec::new(),
            operations: Vec::new(),
            reversible: true,
            raw_sql_up: None,
            raw_sql_down: None,
            checksum: String::new(),
            created_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set app
    pub fn app(mut self, app: impl Into<String>) -> Self {
        self.app = Some(app.into());
        self
    }

    /// Add a dependency
    pub fn depends_on(mut self, migration_id: impl Into<String>) -> Self {
        self.dependencies.push(migration_id.into());
        self
    }

    /// Add an operation
    pub fn operation(mut self, op: MigrationOperation) -> Self {
        self.operations.push(op);
        self
    }

    /// Add operations
    pub fn operations(mut self, ops: Vec<MigrationOperation>) -> Self {
        self.operations.extend(ops);
        self
    }

    /// Set raw SQL
    pub fn raw_sql(mut self, up: impl Into<String>, down: Option<String>) -> Self {
        self.raw_sql_up = Some(up.into());
        self.raw_sql_down = down;
        if self.raw_sql_down.is_none() {
            self.reversible = false;
        }
        self
    }

    /// Calculate and set checksum
    pub fn with_checksum(mut self) -> Self {
        self.checksum = self.calculate_checksum();
        self
    }

    /// Calculate checksum of migration content
    pub fn calculate_checksum(&self) -> String {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        // Hash operations
        for op in &self.operations {
            let json = serde_json::to_string(op).unwrap_or_default();
            hasher.update(json.as_bytes());
        }

        // Hash raw SQL if present
        if let Some(ref sql) = self.raw_sql_up {
            hasher.update(sql.as_bytes());
        }

        hex::encode(hasher.finalize())
    }

    /// Verify checksum
    pub fn verify_checksum(&self) -> bool {
        self.checksum.is_empty() || self.checksum == self.calculate_checksum()
    }

    /// Check if this migration is empty
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty() && self.raw_sql_up.is_none()
    }
}

/// Migration direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationDirection {
    /// Apply migration (forward)
    Up,
    /// Rollback migration (reverse)
    Down,
}

impl std::fmt::Display for MigrationDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationDirection::Up => write!(f, "up"),
            MigrationDirection::Down => write!(f, "down"),
        }
    }
}

/// Migration status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStatus {
    /// Not yet applied
    Pending,
    /// Currently being applied
    Running,
    /// Successfully applied
    Applied,
    /// Failed to apply
    Failed,
    /// Rolled back
    RolledBack,
}

impl std::fmt::Display for MigrationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationStatus::Pending => write!(f, "pending"),
            MigrationStatus::Running => write!(f, "running"),
            MigrationStatus::Applied => write!(f, "applied"),
            MigrationStatus::Failed => write!(f, "failed"),
            MigrationStatus::RolledBack => write!(f, "rolled_back"),
        }
    }
}

/// Result of a migration operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    /// Migration ID
    pub migration_id: String,
    /// Direction
    pub direction: MigrationDirection,
    /// Success or failure
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// SQL statements executed
    pub statements_executed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chakra_schema::schema::{Column, ColumnType, Table};

    #[test]
    fn test_migration_creation() {
        let migration = Migration::new("20240101_000000", "create_users")
            .description("Create users table")
            .app("core")
            .operation(MigrationOperation::CreateTable(
                Table::new("users").column(Column::new("id", ColumnType::BigSerial).not_null()),
            ));

        assert_eq!(migration.id, "20240101_000000");
        assert_eq!(migration.name, "create_users");
        assert_eq!(migration.operations.len(), 1);
        assert!(migration.reversible);
    }

    #[test]
    fn test_checksum() {
        let m1 = Migration::new("1", "test")
            .operation(MigrationOperation::CreateTable(Table::new("foo")))
            .with_checksum();

        let m2 = Migration::new("1", "test")
            .operation(MigrationOperation::CreateTable(Table::new("foo")))
            .with_checksum();

        let m3 = Migration::new("1", "test")
            .operation(MigrationOperation::CreateTable(Table::new("bar")))
            .with_checksum();

        assert_eq!(m1.checksum, m2.checksum);
        assert_ne!(m1.checksum, m3.checksum);
    }
}
