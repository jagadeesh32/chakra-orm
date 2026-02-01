//! Migration executor for applying and rolling back migrations

use crate::history::{MigrationHistory, MigrationRecord};
use crate::migration::{Migration, MigrationDirection, MigrationResult, MigrationStatus};
use crate::planner::PlannedMigration;
use async_trait::async_trait;
use chakra_core::error::{ChakraError, Result};
use chakra_schema::ddl::{DdlGenerator, DdlStatement};
use std::time::Instant;
use tracing::{debug, error, info, warn};

/// Trait for executing SQL statements
#[async_trait]
pub trait SqlExecutor: Send + Sync {
    /// Execute a single SQL statement
    async fn execute(&self, sql: &str) -> Result<u64>;

    /// Execute multiple statements in a transaction
    async fn execute_in_transaction(&self, statements: &[&str]) -> Result<Vec<u64>>;

    /// Begin a transaction
    async fn begin_transaction(&self) -> Result<()>;

    /// Commit a transaction
    async fn commit_transaction(&self) -> Result<()>;

    /// Rollback a transaction
    async fn rollback_transaction(&self) -> Result<()>;
}

/// Migration executor
pub struct MigrationExecutor<'a> {
    /// SQL executor
    executor: &'a dyn SqlExecutor,
    /// DDL generator
    ddl_generator: &'a dyn DdlGenerator,
    /// Migration history
    history: &'a dyn MigrationHistory,
    /// Whether to use transactions
    use_transactions: bool,
    /// Whether to run in dry-run mode
    dry_run: bool,
}

impl<'a> MigrationExecutor<'a> {
    /// Create a new executor
    pub fn new(
        executor: &'a dyn SqlExecutor,
        ddl_generator: &'a dyn DdlGenerator,
        history: &'a dyn MigrationHistory,
    ) -> Self {
        Self {
            executor,
            ddl_generator,
            history,
            use_transactions: true,
            dry_run: false,
        }
    }

    /// Set whether to use transactions
    pub fn use_transactions(mut self, use_tx: bool) -> Self {
        self.use_transactions = use_tx;
        self
    }

    /// Set dry-run mode
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Execute a plan
    pub async fn execute_plan(&self, plan: &[PlannedMigration]) -> Vec<MigrationResult> {
        let mut results = Vec::new();

        // Acquire lock
        let lock = match self.history.acquire_lock().await {
            Ok(l) => l,
            Err(e) => {
                error!("Failed to acquire migration lock: {}", e);
                return results;
            }
        };

        for planned in plan {
            let result = self.execute_one(planned).await;
            let success = result.success;
            results.push(result);

            if !success {
                warn!("Migration {} failed, stopping execution", planned.migration.id);
                break;
            }
        }

        // Release lock
        if let Err(e) = self.history.release_lock(lock).await {
            error!("Failed to release migration lock: {}", e);
        }

        results
    }

    /// Execute a single migration
    async fn execute_one(&self, planned: &PlannedMigration) -> MigrationResult {
        let migration = &planned.migration;
        let direction = planned.direction;
        let start = Instant::now();

        info!(
            "Running migration {} {} ({})",
            migration.id,
            direction,
            migration.name
        );

        // Generate SQL statements
        let statements = match direction {
            MigrationDirection::Up => self.generate_up_statements(migration),
            MigrationDirection::Down => self.generate_down_statements(migration),
        };

        if self.dry_run {
            info!("DRY RUN: Would execute {} statements", statements.len());
            for (i, stmt) in statements.iter().enumerate() {
                debug!("  {}: {}", i + 1, stmt.sql);
            }
            return MigrationResult {
                migration_id: migration.id.clone(),
                direction,
                success: true,
                error: None,
                duration_ms: start.elapsed().as_millis() as u64,
                statements_executed: 0,
            };
        }

        // Execute statements
        let result = if self.use_transactions {
            self.execute_with_transaction(&statements).await
        } else {
            self.execute_without_transaction(&statements).await
        };

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(count) => {
                // Record in history
                let record = MigrationRecord::new(&migration.id, &migration.name)
                    .applied(duration_ms, count);

                match direction {
                    MigrationDirection::Up => {
                        if let Err(e) = self.history.record_applied(record).await {
                            error!("Failed to record migration: {}", e);
                        }
                    }
                    MigrationDirection::Down => {
                        if let Err(e) = self.history.record_rollback(&migration.id).await {
                            error!("Failed to record rollback: {}", e);
                        }
                    }
                }

                info!(
                    "Migration {} completed in {}ms ({} statements)",
                    migration.id, duration_ms, count
                );

                MigrationResult {
                    migration_id: migration.id.clone(),
                    direction,
                    success: true,
                    error: None,
                    duration_ms,
                    statements_executed: count,
                }
            }
            Err(e) => {
                error!("Migration {} failed: {}", migration.id, e);

                // Record failure
                let record = MigrationRecord::new(&migration.id, &migration.name)
                    .failed(e.to_string());

                if let Err(e) = self.history.record_applied(record).await {
                    error!("Failed to record migration failure: {}", e);
                }

                MigrationResult {
                    migration_id: migration.id.clone(),
                    direction,
                    success: false,
                    error: Some(e.to_string()),
                    duration_ms,
                    statements_executed: 0,
                }
            }
        }
    }

    /// Generate up (forward) statements
    fn generate_up_statements(&self, migration: &Migration) -> Vec<DdlStatement> {
        let mut statements = Vec::new();

        // Add raw SQL if present
        if let Some(ref sql) = migration.raw_sql_up {
            statements.push(DdlStatement::new(sql));
        }

        // Generate from operations
        for op in &migration.operations {
            statements.extend(self.operation_to_statements(op, MigrationDirection::Up));
        }

        statements
    }

    /// Generate down (reverse) statements
    fn generate_down_statements(&self, migration: &Migration) -> Vec<DdlStatement> {
        let mut statements = Vec::new();

        // Add raw SQL if present
        if let Some(ref sql) = migration.raw_sql_down {
            statements.push(DdlStatement::new(sql));
        }

        // Generate from operations in reverse order
        for op in migration.operations.iter().rev() {
            statements.extend(self.operation_to_statements(op, MigrationDirection::Down));
        }

        statements
    }

    /// Convert an operation to DDL statements
    fn operation_to_statements(
        &self,
        op: &chakra_schema::diff::MigrationOperation,
        direction: MigrationDirection,
    ) -> Vec<DdlStatement> {
        use chakra_schema::diff::MigrationOperation::*;

        match (op, direction) {
            (CreateTable(table), MigrationDirection::Up) => {
                vec![self.ddl_generator.create_table(table)]
            }
            (CreateTable(table), MigrationDirection::Down) => {
                vec![self.ddl_generator.drop_table(&table.name, true)]
            }
            (DropTable { name, cascade }, MigrationDirection::Up) => {
                vec![self.ddl_generator.drop_table(name, *cascade)]
            }
            (RenameTable { from, to }, MigrationDirection::Up) => {
                vec![self.ddl_generator.rename_table(from, to)]
            }
            (RenameTable { from, to }, MigrationDirection::Down) => {
                vec![self.ddl_generator.rename_table(to, from)]
            }
            (AddColumn { table, column }, MigrationDirection::Up) => {
                vec![self.ddl_generator.add_column(table, column)]
            }
            (AddColumn { table, column }, MigrationDirection::Down) => {
                vec![self.ddl_generator.drop_column(table, &column.name)]
            }
            (DropColumn { table, column }, MigrationDirection::Up) => {
                vec![self.ddl_generator.drop_column(table, column)]
            }
            (AlterColumn { table, from, to }, MigrationDirection::Up) => {
                self.ddl_generator.alter_column(table, from, to)
            }
            (AlterColumn { table, from, to }, MigrationDirection::Down) => {
                self.ddl_generator.alter_column(table, to, from)
            }
            (RenameColumn { table, from, to }, MigrationDirection::Up) => {
                vec![self.ddl_generator.rename_column(table, from, to)]
            }
            (RenameColumn { table, from, to }, MigrationDirection::Down) => {
                vec![self.ddl_generator.rename_column(table, to, from)]
            }
            (CreateIndex { table, index }, MigrationDirection::Up) => {
                vec![self.ddl_generator.create_index(table, index)]
            }
            (CreateIndex { table, index }, MigrationDirection::Down) => {
                vec![self.ddl_generator.drop_index(&index.name)]
            }
            (DropIndex { name }, MigrationDirection::Up) => {
                vec![self.ddl_generator.drop_index(name)]
            }
            (AddConstraint { table, constraint }, MigrationDirection::Up) => {
                vec![self.ddl_generator.add_constraint(table, constraint)]
            }
            (AddConstraint { table, constraint }, MigrationDirection::Down) => {
                vec![self.ddl_generator.drop_constraint(table, &constraint.name)]
            }
            (DropConstraint { table, name }, MigrationDirection::Up) => {
                vec![self.ddl_generator.drop_constraint(table, name)]
            }
            (AddForeignKey { table, foreign_key }, MigrationDirection::Up) => {
                vec![self.ddl_generator.add_foreign_key(table, foreign_key)]
            }
            (AddForeignKey { table, foreign_key }, MigrationDirection::Down) => {
                let fk_name = foreign_key
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("fk_{}_{}", table, foreign_key.columns.join("_")));
                vec![self.ddl_generator.drop_foreign_key(table, &fk_name)]
            }
            (DropForeignKey { table, name }, MigrationDirection::Up) => {
                vec![self.ddl_generator.drop_foreign_key(table, name)]
            }
            (RawSql { up, down }, MigrationDirection::Up) => {
                vec![DdlStatement::new(up)]
            }
            (RawSql { up, down }, MigrationDirection::Down) => {
                down.as_ref()
                    .map(|sql| vec![DdlStatement::new(sql)])
                    .unwrap_or_default()
            }
            _ => vec![],
        }
    }

    /// Execute statements with a transaction
    async fn execute_with_transaction(&self, statements: &[DdlStatement]) -> Result<usize> {
        self.executor.begin_transaction().await?;

        let mut executed = 0;
        for stmt in statements {
            debug!("Executing: {}", stmt.sql);
            match self.executor.execute(&stmt.sql).await {
                Ok(_) => executed += 1,
                Err(e) => {
                    error!("Statement failed: {}", e);
                    self.executor.rollback_transaction().await?;
                    return Err(e);
                }
            }
        }

        self.executor.commit_transaction().await?;
        Ok(executed)
    }

    /// Execute statements without a transaction
    async fn execute_without_transaction(&self, statements: &[DdlStatement]) -> Result<usize> {
        let mut executed = 0;
        for stmt in statements {
            debug!("Executing: {}", stmt.sql);
            self.executor.execute(&stmt.sql).await?;
            executed += 1;
        }
        Ok(executed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::InMemoryHistory;
    use chakra_schema::ddl::PostgresDdlGenerator;
    use chakra_schema::schema::{Column, ColumnType, Table};

    // Mock SQL executor for testing
    struct MockExecutor {
        statements: tokio::sync::Mutex<Vec<String>>,
    }

    impl MockExecutor {
        fn new() -> Self {
            Self {
                statements: tokio::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl SqlExecutor for MockExecutor {
        async fn execute(&self, sql: &str) -> Result<u64> {
            self.statements.lock().await.push(sql.to_string());
            Ok(1)
        }

        async fn execute_in_transaction(&self, statements: &[&str]) -> Result<Vec<u64>> {
            for sql in statements {
                self.statements.lock().await.push(sql.to_string());
            }
            Ok(vec![1; statements.len()])
        }

        async fn begin_transaction(&self) -> Result<()> {
            self.statements.lock().await.push("BEGIN".to_string());
            Ok(())
        }

        async fn commit_transaction(&self) -> Result<()> {
            self.statements.lock().await.push("COMMIT".to_string());
            Ok(())
        }

        async fn rollback_transaction(&self) -> Result<()> {
            self.statements.lock().await.push("ROLLBACK".to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_execute_migration() {
        let executor = MockExecutor::new();
        let ddl_gen = PostgresDdlGenerator;
        let history = InMemoryHistory::new();

        let migration = Migration::new("001", "create_users")
            .operation(chakra_schema::diff::MigrationOperation::CreateTable(
                Table::new("users")
                    .column(Column::new("id", ColumnType::BigSerial).not_null()),
            ));

        let planned = PlannedMigration {
            migration,
            direction: MigrationDirection::Up,
        };

        let exec = MigrationExecutor::new(&executor, &ddl_gen, &history);
        let results = exec.execute_plan(&[planned]).await;

        assert_eq!(results.len(), 1);
        assert!(results[0].success);

        let stmts = executor.statements.lock().await;
        assert!(stmts.iter().any(|s| s.contains("CREATE TABLE")));
    }
}
