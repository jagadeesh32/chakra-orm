//! Migration planning and dependency resolution

use crate::file::MigrationFile;
use crate::history::MigrationHistory;
use crate::migration::{Migration, MigrationDirection};
use chakra_core::error::{ChakraError, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, info, warn};

/// A planned migration operation
#[derive(Debug, Clone)]
pub struct PlannedMigration {
    /// The migration to run
    pub migration: Migration,
    /// Direction (up or down)
    pub direction: MigrationDirection,
}

/// Migration planner for determining which migrations to run
#[derive(Debug)]
pub struct MigrationPlanner {
    /// All available migrations
    migrations: HashMap<String, Migration>,
    /// Migration dependency graph
    dependencies: HashMap<String, Vec<String>>,
}

impl MigrationPlanner {
    /// Create a new planner from migration files
    pub fn new(files: Vec<MigrationFile>) -> Self {
        let mut migrations = HashMap::new();
        let mut dependencies = HashMap::new();

        for file in files {
            let id = file.migration.id.clone();
            dependencies.insert(id.clone(), file.migration.dependencies.clone());
            migrations.insert(id, file.migration);
        }

        Self {
            migrations,
            dependencies,
        }
    }

    /// Plan migrations to apply (up)
    pub async fn plan_up(
        &self,
        history: &dyn MigrationHistory,
        target: Option<&str>,
    ) -> Result<Vec<PlannedMigration>> {
        let applied = history.get_applied().await?;
        let applied_ids: HashSet<_> = applied.iter().map(|r| r.id.as_str()).collect();

        // Find pending migrations
        let pending: Vec<_> = self
            .migrations
            .values()
            .filter(|m| !applied_ids.contains(m.id.as_str()))
            .cloned()
            .collect();

        if pending.is_empty() {
            info!("No pending migrations");
            return Ok(vec![]);
        }

        // Sort by dependencies (topological sort)
        let sorted = self.topological_sort(&pending)?;

        // Filter to target if specified
        let to_run = if let Some(target_id) = target {
            let mut result = Vec::new();
            for m in sorted {
                result.push(m.clone());
                if m.id == target_id {
                    break;
                }
            }
            result
        } else {
            sorted
        };

        let planned: Vec<_> = to_run
            .into_iter()
            .map(|m| PlannedMigration {
                migration: m,
                direction: MigrationDirection::Up,
            })
            .collect();

        info!("Planned {} migrations to apply", planned.len());
        Ok(planned)
    }

    /// Plan migrations to rollback (down)
    pub async fn plan_down(
        &self,
        history: &dyn MigrationHistory,
        count: usize,
    ) -> Result<Vec<PlannedMigration>> {
        let applied = history.get_applied().await?;

        if applied.is_empty() {
            info!("No migrations to rollback");
            return Ok(vec![]);
        }

        // Take the last N migrations in reverse order
        let to_rollback: Vec<_> = applied
            .into_iter()
            .rev()
            .take(count)
            .filter_map(|r| self.migrations.get(&r.id).cloned())
            .collect();

        // Check if they're reversible
        for m in &to_rollback {
            if !m.reversible {
                return Err(ChakraError::internal(format!(
                    "Migration {} is not reversible",
                    m.id
                )));
            }
        }

        let planned: Vec<_> = to_rollback
            .into_iter()
            .map(|m| PlannedMigration {
                migration: m,
                direction: MigrationDirection::Down,
            })
            .collect();

        info!("Planned {} migrations to rollback", planned.len());
        Ok(planned)
    }

    /// Plan migrations to a specific target (up or down as needed)
    pub async fn plan_to(
        &self,
        history: &dyn MigrationHistory,
        target: &str,
    ) -> Result<Vec<PlannedMigration>> {
        let applied = history.get_applied().await?;
        let applied_ids: HashSet<_> = applied.iter().map(|r| r.id.as_str()).collect();

        // Find the target migration
        if !self.migrations.contains_key(target) {
            return Err(ChakraError::internal(format!(
                "Migration {} not found",
                target
            )));
        }

        if applied_ids.contains(target) {
            // Need to rollback to this point
            let mut to_rollback = Vec::new();

            for record in applied.iter().rev() {
                if record.id == target {
                    break;
                }
                if let Some(m) = self.migrations.get(&record.id) {
                    to_rollback.push(PlannedMigration {
                        migration: m.clone(),
                        direction: MigrationDirection::Down,
                    });
                }
            }

            Ok(to_rollback)
        } else {
            // Need to apply up to this point
            self.plan_up(history, Some(target)).await
        }
    }

    /// Topological sort of migrations based on dependencies
    fn topological_sort(&self, migrations: &[Migration]) -> Result<Vec<Migration>> {
        let ids: HashSet<_> = migrations.iter().map(|m| m.id.as_str()).collect();
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();

        // Build graph
        for m in migrations {
            in_degree.entry(m.id.as_str()).or_insert(0);
            graph.entry(m.id.as_str()).or_insert_with(Vec::new);

            for dep in &m.dependencies {
                if ids.contains(dep.as_str()) {
                    *in_degree.entry(m.id.as_str()).or_insert(0) += 1;
                    graph
                        .entry(dep.as_str())
                        .or_insert_with(Vec::new)
                        .push(m.id.as_str());
                }
            }
        }

        // Kahn's algorithm
        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();

        while let Some(id) = queue.pop_front() {
            if let Some(m) = migrations.iter().find(|m| m.id == id) {
                result.push(m.clone());
            }

            if let Some(dependents) = graph.get(id) {
                for &dep in dependents {
                    if let Some(degree) = in_degree.get_mut(dep) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dep);
                        }
                    }
                }
            }
        }

        if result.len() != migrations.len() {
            return Err(ChakraError::internal(
                "Circular dependency detected in migrations",
            ));
        }

        Ok(result)
    }

    /// Validate migration dependencies
    pub fn validate(&self) -> Result<()> {
        for (id, deps) in &self.dependencies {
            for dep in deps {
                if !self.migrations.contains_key(dep) {
                    warn!(
                        "Migration {} depends on missing migration {}",
                        id, dep
                    );
                }
            }
        }

        // Check for circular dependencies
        let all_migrations: Vec<_> = self.migrations.values().cloned().collect();
        self.topological_sort(&all_migrations)?;

        Ok(())
    }

    /// Get pending migrations count
    pub async fn pending_count(&self, history: &dyn MigrationHistory) -> Result<usize> {
        let applied = history.get_applied().await?;
        let applied_ids: HashSet<_> = applied.iter().map(|r| r.id.as_str()).collect();

        Ok(self
            .migrations
            .keys()
            .filter(|id| !applied_ids.contains(id.as_str()))
            .count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::InMemoryHistory;

    fn create_test_migration(id: &str, deps: Vec<&str>) -> MigrationFile {
        let mut m = Migration::new(id, format!("migration_{}", id));
        m.dependencies = deps.into_iter().map(String::from).collect();
        MigrationFile {
            path: format!("{}.toml", id).into(),
            migration: m,
        }
    }

    #[tokio::test]
    async fn test_plan_up() {
        let files = vec![
            create_test_migration("001", vec![]),
            create_test_migration("002", vec!["001"]),
            create_test_migration("003", vec!["002"]),
        ];

        let planner = MigrationPlanner::new(files);
        let history = InMemoryHistory::new();

        let plan = planner.plan_up(&history, None).await.unwrap();
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].migration.id, "001");
        assert_eq!(plan[1].migration.id, "002");
        assert_eq!(plan[2].migration.id, "003");
    }

    #[tokio::test]
    async fn test_circular_dependency() {
        let files = vec![
            create_test_migration("001", vec!["002"]),
            create_test_migration("002", vec!["001"]),
        ];

        let planner = MigrationPlanner::new(files);
        assert!(planner.validate().is_err());
    }
}
