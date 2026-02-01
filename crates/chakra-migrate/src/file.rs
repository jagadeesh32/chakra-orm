//! Migration file management

use crate::migration::Migration;
use chakra_core::error::{ChakraError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Migration file on disk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationFile {
    /// File path
    pub path: PathBuf,
    /// Migration content
    pub migration: Migration,
}

impl MigrationFile {
    /// Create a new migration file
    pub fn new(path: impl Into<PathBuf>, migration: Migration) -> Self {
        Self {
            path: path.into(),
            migration,
        }
    }

    /// Get the filename
    pub fn filename(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
    }
}

/// Migration file loader
#[derive(Debug, Clone)]
pub struct MigrationLoader {
    /// Root migrations directory
    pub root: PathBuf,
    /// File extension for migrations
    pub extension: String,
}

impl MigrationLoader {
    /// Create a new loader
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            extension: "toml".to_string(),
        }
    }

    /// Set the file extension
    pub fn extension(mut self, ext: impl Into<String>) -> Self {
        self.extension = ext.into();
        self
    }

    /// Load all migrations from disk
    pub async fn load_all(&self) -> Result<Vec<MigrationFile>> {
        let mut migrations = Vec::new();

        if !self.root.exists() {
            debug!("Migrations directory does not exist: {:?}", self.root);
            return Ok(migrations);
        }

        for entry in WalkDir::new(&self.root)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == self.extension.as_str() {
                        match self.load_file(path).await {
                            Ok(mf) => migrations.push(mf),
                            Err(e) => {
                                warn!("Failed to load migration file {:?}: {}", path, e);
                            }
                        }
                    }
                }
            }
        }

        // Sort by ID
        migrations.sort_by(|a, b| a.migration.id.cmp(&b.migration.id));

        info!("Loaded {} migrations", migrations.len());
        Ok(migrations)
    }

    /// Load a single migration file
    pub async fn load_file(&self, path: &Path) -> Result<MigrationFile> {
        let content = fs::read_to_string(path).await.map_err(|e| {
            ChakraError::internal(format!("Failed to read migration file: {}", e))
        })?;

        let migration: Migration = toml::from_str(&content).map_err(|e| {
            ChakraError::internal(format!("Failed to parse migration file: {}", e))
        })?;

        Ok(MigrationFile::new(path, migration))
    }

    /// Save a migration to disk
    pub async fn save(&self, migration: &Migration, app: Option<&str>) -> Result<PathBuf> {
        // Determine directory
        let dir = match app {
            Some(app_name) => self.root.join(app_name),
            None => self.root.clone(),
        };

        // Create directory if needed
        fs::create_dir_all(&dir).await.map_err(|e| {
            ChakraError::internal(format!("Failed to create migrations directory: {}", e))
        })?;

        // Generate filename
        let filename = format!("{}_{}.{}", migration.id, migration.name, self.extension);
        let path = dir.join(&filename);

        // Serialize to TOML
        let content = toml::to_string_pretty(migration).map_err(|e| {
            ChakraError::internal(format!("Failed to serialize migration: {}", e))
        })?;

        // Write file
        fs::write(&path, content).await.map_err(|e| {
            ChakraError::internal(format!("Failed to write migration file: {}", e))
        })?;

        info!("Saved migration to {:?}", path);
        Ok(path)
    }

    /// Get path for a new migration
    pub fn new_migration_path(&self, id: &str, name: &str, app: Option<&str>) -> PathBuf {
        let dir = match app {
            Some(app_name) => self.root.join(app_name),
            None => self.root.clone(),
        };

        let filename = format!("{}_{}.{}", id, name, self.extension);
        dir.join(filename)
    }
}

/// Generate a new migration ID
pub fn generate_migration_id() -> String {
    chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string()
}

/// Generate a new migration ID with sequence number
pub fn generate_migration_id_seq(seq: u32) -> String {
    format!(
        "{}_{}",
        chrono::Utc::now().format("%Y%m%d"),
        format!("{:04}", seq)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let loader = MigrationLoader::new(temp_dir.path());

        let migration = Migration::new(generate_migration_id(), "test_migration")
            .description("A test migration");

        let path = loader.save(&migration, None).await.unwrap();
        assert!(path.exists());

        let loaded = loader.load_file(&path).await.unwrap();
        assert_eq!(loaded.migration.name, "test_migration");
    }

    #[test]
    fn test_migration_id_format() {
        let id = generate_migration_id();
        assert!(id.len() > 10);
        assert!(id.contains('_'));
    }
}
