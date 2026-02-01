//! Schema diff and comparison for Chakra ORM
//!
//! This module provides schema comparison and diff generation.

use crate::ddl::{DdlGenerator, DdlStatement};
use crate::schema::{Column, ColumnType, Constraint, ForeignKey, Index, Schema, Table};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A difference between two schemas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDiff {
    /// Tables to create
    pub tables_to_create: Vec<Table>,
    /// Tables to drop
    pub tables_to_drop: Vec<String>,
    /// Table modifications
    pub table_modifications: Vec<TableDiff>,
}

impl SchemaDiff {
    /// Check if there are any differences
    pub fn is_empty(&self) -> bool {
        self.tables_to_create.is_empty()
            && self.tables_to_drop.is_empty()
            && self.table_modifications.is_empty()
    }

    /// Generate DDL statements for the diff
    pub fn to_ddl(&self, generator: &dyn DdlGenerator) -> Vec<DdlStatement> {
        let mut statements = Vec::new();

        // Drop foreign keys first (to avoid FK constraint violations)
        for table_diff in &self.table_modifications {
            for fk_name in &table_diff.foreign_keys_to_drop {
                statements.push(generator.drop_foreign_key(&table_diff.table_name, fk_name));
            }
        }

        // Drop tables
        for table_name in &self.tables_to_drop {
            statements.push(generator.drop_table(table_name, true));
        }

        // Create new tables
        for table in &self.tables_to_create {
            statements.push(generator.create_table(table));
            // Create indexes
            for index in &table.indexes {
                statements.push(generator.create_index(&table.name, index));
            }
        }

        // Modify existing tables
        for table_diff in &self.table_modifications {
            // Rename table if needed
            if let Some(new_name) = &table_diff.rename_to {
                statements.push(generator.rename_table(&table_diff.table_name, new_name));
            }

            // Drop indexes
            for index_name in &table_diff.indexes_to_drop {
                statements.push(generator.drop_index(index_name));
            }

            // Drop constraints
            for constraint_name in &table_diff.constraints_to_drop {
                statements.push(generator.drop_constraint(&table_diff.table_name, constraint_name));
            }

            // Drop columns
            for column_name in &table_diff.columns_to_drop {
                statements.push(generator.drop_column(&table_diff.table_name, column_name));
            }

            // Add columns
            for column in &table_diff.columns_to_add {
                statements.push(generator.add_column(&table_diff.table_name, column));
            }

            // Modify columns
            for (old, new) in &table_diff.columns_to_modify {
                statements.extend(generator.alter_column(&table_diff.table_name, old, new));
            }

            // Create indexes
            for index in &table_diff.indexes_to_create {
                statements.push(generator.create_index(&table_diff.table_name, index));
            }

            // Add constraints
            for constraint in &table_diff.constraints_to_add {
                statements.push(generator.add_constraint(&table_diff.table_name, constraint));
            }
        }

        // Add foreign keys last (after all tables/columns exist)
        for table_diff in &self.table_modifications {
            for fk in &table_diff.foreign_keys_to_add {
                statements.push(generator.add_foreign_key(&table_diff.table_name, fk));
            }
        }

        // Add foreign keys for new tables
        for table in &self.tables_to_create {
            for fk in &table.foreign_keys {
                statements.push(generator.add_foreign_key(&table.name, fk));
            }
        }

        statements
    }
}

/// Differences for a single table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDiff {
    /// Table name
    pub table_name: String,
    /// Rename to (if renaming)
    pub rename_to: Option<String>,
    /// Columns to add
    pub columns_to_add: Vec<Column>,
    /// Columns to drop
    pub columns_to_drop: Vec<String>,
    /// Columns to modify (old, new)
    pub columns_to_modify: Vec<(Column, Column)>,
    /// Indexes to create
    pub indexes_to_create: Vec<Index>,
    /// Indexes to drop
    pub indexes_to_drop: Vec<String>,
    /// Constraints to add
    pub constraints_to_add: Vec<Constraint>,
    /// Constraints to drop
    pub constraints_to_drop: Vec<String>,
    /// Foreign keys to add
    pub foreign_keys_to_add: Vec<ForeignKey>,
    /// Foreign keys to drop
    pub foreign_keys_to_drop: Vec<String>,
}

impl TableDiff {
    /// Create a new empty table diff
    pub fn new(table_name: impl Into<String>) -> Self {
        Self {
            table_name: table_name.into(),
            rename_to: None,
            columns_to_add: Vec::new(),
            columns_to_drop: Vec::new(),
            columns_to_modify: Vec::new(),
            indexes_to_create: Vec::new(),
            indexes_to_drop: Vec::new(),
            constraints_to_add: Vec::new(),
            constraints_to_drop: Vec::new(),
            foreign_keys_to_add: Vec::new(),
            foreign_keys_to_drop: Vec::new(),
        }
    }

    /// Check if this diff has any changes
    pub fn is_empty(&self) -> bool {
        self.rename_to.is_none()
            && self.columns_to_add.is_empty()
            && self.columns_to_drop.is_empty()
            && self.columns_to_modify.is_empty()
            && self.indexes_to_create.is_empty()
            && self.indexes_to_drop.is_empty()
            && self.constraints_to_add.is_empty()
            && self.constraints_to_drop.is_empty()
            && self.foreign_keys_to_add.is_empty()
            && self.foreign_keys_to_drop.is_empty()
    }
}

/// Schema differ for comparing two schemas
#[derive(Debug, Default)]
pub struct SchemaDiffer {
    /// Ignore column order differences
    pub ignore_column_order: bool,
    /// Ignore index name differences
    pub ignore_index_names: bool,
    /// Tables to exclude from comparison
    pub exclude_tables: HashSet<String>,
}

impl SchemaDiffer {
    /// Create a new schema differ
    pub fn new() -> Self {
        Self::default()
    }

    /// Set ignore column order
    pub fn ignore_column_order(mut self, ignore: bool) -> Self {
        self.ignore_column_order = ignore;
        self
    }

    /// Set ignore index names
    pub fn ignore_index_names(mut self, ignore: bool) -> Self {
        self.ignore_index_names = ignore;
        self
    }

    /// Exclude a table from comparison
    pub fn exclude_table(mut self, table: impl Into<String>) -> Self {
        self.exclude_tables.insert(table.into());
        self
    }

    /// Compare two schemas and return the diff
    pub fn diff(&self, from: &Schema, to: &Schema) -> SchemaDiff {
        let mut diff = SchemaDiff {
            tables_to_create: Vec::new(),
            tables_to_drop: Vec::new(),
            table_modifications: Vec::new(),
        };

        let from_tables: HashSet<&str> = from
            .tables
            .keys()
            .filter(|t| !self.exclude_tables.contains(*t))
            .map(|s| s.as_str())
            .collect();

        let to_tables: HashSet<&str> = to
            .tables
            .keys()
            .filter(|t| !self.exclude_tables.contains(*t))
            .map(|s| s.as_str())
            .collect();

        // Tables to create (in to but not in from)
        for table_name in to_tables.difference(&from_tables) {
            if let Some(table) = to.tables.get(*table_name) {
                diff.tables_to_create.push(table.clone());
            }
        }

        // Tables to drop (in from but not in to)
        for table_name in from_tables.difference(&to_tables) {
            diff.tables_to_drop.push((*table_name).to_string());
        }

        // Tables to modify (in both)
        for table_name in from_tables.intersection(&to_tables) {
            let from_table = from.tables.get(*table_name).unwrap();
            let to_table = to.tables.get(*table_name).unwrap();
            let table_diff = self.diff_tables(from_table, to_table);
            if !table_diff.is_empty() {
                diff.table_modifications.push(table_diff);
            }
        }

        diff
    }

    /// Compare two tables and return the diff
    fn diff_tables(&self, from: &Table, to: &Table) -> TableDiff {
        let mut diff = TableDiff::new(&from.name);

        // Compare columns
        let from_columns: HashMap<&str, &Column> =
            from.columns.iter().map(|c| (c.name.as_str(), c)).collect();
        let to_columns: HashMap<&str, &Column> =
            to.columns.iter().map(|c| (c.name.as_str(), c)).collect();

        let from_col_names: HashSet<&str> = from_columns.keys().copied().collect();
        let to_col_names: HashSet<&str> = to_columns.keys().copied().collect();

        // Columns to add
        for col_name in to_col_names.difference(&from_col_names) {
            diff.columns_to_add.push(to_columns[*col_name].clone());
        }

        // Columns to drop
        for col_name in from_col_names.difference(&to_col_names) {
            diff.columns_to_drop.push((*col_name).to_string());
        }

        // Columns to modify
        for col_name in from_col_names.intersection(&to_col_names) {
            let from_col = from_columns[*col_name];
            let to_col = to_columns[*col_name];
            if self.columns_differ(from_col, to_col) {
                diff.columns_to_modify
                    .push((from_col.clone(), to_col.clone()));
            }
        }

        // Compare indexes
        let from_indexes: HashMap<&str, &Index> =
            from.indexes.iter().map(|i| (i.name.as_str(), i)).collect();
        let to_indexes: HashMap<&str, &Index> =
            to.indexes.iter().map(|i| (i.name.as_str(), i)).collect();

        let from_idx_names: HashSet<&str> = from_indexes.keys().copied().collect();
        let to_idx_names: HashSet<&str> = to_indexes.keys().copied().collect();

        for idx_name in to_idx_names.difference(&from_idx_names) {
            diff.indexes_to_create.push(to_indexes[*idx_name].clone());
        }

        for idx_name in from_idx_names.difference(&to_idx_names) {
            diff.indexes_to_drop.push((*idx_name).to_string());
        }

        // Compare constraints
        let from_constraints: HashMap<&str, &Constraint> = from
            .constraints
            .iter()
            .map(|c| (c.name.as_str(), c))
            .collect();
        let to_constraints: HashMap<&str, &Constraint> = to
            .constraints
            .iter()
            .map(|c| (c.name.as_str(), c))
            .collect();

        let from_const_names: HashSet<&str> = from_constraints.keys().copied().collect();
        let to_const_names: HashSet<&str> = to_constraints.keys().copied().collect();

        for const_name in to_const_names.difference(&from_const_names) {
            diff.constraints_to_add
                .push(to_constraints[*const_name].clone());
        }

        for const_name in from_const_names.difference(&to_const_names) {
            diff.constraints_to_drop.push((*const_name).to_string());
        }

        // Compare foreign keys
        let from_fks: HashMap<String, &ForeignKey> = from
            .foreign_keys
            .iter()
            .map(|fk| {
                let name = fk
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("fk_{}_{}", from.name, fk.columns.join("_")));
                (name, fk)
            })
            .collect();
        let to_fks: HashMap<String, &ForeignKey> = to
            .foreign_keys
            .iter()
            .map(|fk| {
                let name = fk
                    .name
                    .clone()
                    .unwrap_or_else(|| format!("fk_{}_{}", to.name, fk.columns.join("_")));
                (name, fk)
            })
            .collect();

        let from_fk_names: HashSet<&str> = from_fks.keys().map(|s| s.as_str()).collect();
        let to_fk_names: HashSet<&str> = to_fks.keys().map(|s| s.as_str()).collect();

        for fk_name in to_fk_names.difference(&from_fk_names) {
            diff.foreign_keys_to_add
                .push(to_fks[*fk_name].clone());
        }

        for fk_name in from_fk_names.difference(&to_fk_names) {
            diff.foreign_keys_to_drop.push((*fk_name).to_string());
        }

        diff
    }

    /// Check if two columns differ
    fn columns_differ(&self, from: &Column, to: &Column) -> bool {
        // Compare type
        if from.column_type != to.column_type {
            return true;
        }

        // Compare nullability
        if from.nullable != to.nullable {
            return true;
        }

        // Compare default (simplified comparison)
        match (&from.default, &to.default) {
            (None, None) => {}
            (Some(_), None) | (None, Some(_)) => return true,
            (Some(a), Some(b)) => {
                if a.to_sql() != b.to_sql() {
                    return true;
                }
            }
        }

        false
    }
}

/// Builder for creating migrations from model changes
#[derive(Debug)]
pub struct MigrationBuilder {
    operations: Vec<MigrationOperation>,
}

/// A migration operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationOperation {
    CreateTable(Table),
    DropTable { name: String, cascade: bool },
    RenameTable { from: String, to: String },
    AddColumn { table: String, column: Column },
    DropColumn { table: String, column: String },
    AlterColumn { table: String, from: Column, to: Column },
    RenameColumn { table: String, from: String, to: String },
    CreateIndex { table: String, index: Index },
    DropIndex { name: String },
    AddConstraint { table: String, constraint: Constraint },
    DropConstraint { table: String, name: String },
    AddForeignKey { table: String, foreign_key: ForeignKey },
    DropForeignKey { table: String, name: String },
    RawSql { up: String, down: Option<String> },
}

impl MigrationBuilder {
    /// Create a new migration builder
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Add a create table operation
    pub fn create_table(mut self, table: Table) -> Self {
        self.operations.push(MigrationOperation::CreateTable(table));
        self
    }

    /// Add a drop table operation
    pub fn drop_table(mut self, name: impl Into<String>, cascade: bool) -> Self {
        self.operations.push(MigrationOperation::DropTable {
            name: name.into(),
            cascade,
        });
        self
    }

    /// Add an add column operation
    pub fn add_column(mut self, table: impl Into<String>, column: Column) -> Self {
        self.operations.push(MigrationOperation::AddColumn {
            table: table.into(),
            column,
        });
        self
    }

    /// Add a drop column operation
    pub fn drop_column(mut self, table: impl Into<String>, column: impl Into<String>) -> Self {
        self.operations.push(MigrationOperation::DropColumn {
            table: table.into(),
            column: column.into(),
        });
        self
    }

    /// Add a raw SQL operation
    pub fn raw_sql(mut self, up: impl Into<String>, down: Option<String>) -> Self {
        self.operations.push(MigrationOperation::RawSql {
            up: up.into(),
            down,
        });
        self
    }

    /// Get the operations
    pub fn operations(&self) -> &[MigrationOperation] {
        &self.operations
    }

    /// Build into operations
    pub fn build(self) -> Vec<MigrationOperation> {
        self.operations
    }
}

impl Default for MigrationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{ColumnType, PrimaryKey};

    #[test]
    fn test_schema_diff_new_table() {
        let from = Schema::new();
        let mut to = Schema::new();

        to.add_table(
            Table::new("users")
                .column(Column::new("id", ColumnType::BigSerial).not_null())
                .column(Column::new("name", ColumnType::Varchar(Some(100))).not_null())
                .primary_key(PrimaryKey::single("id")),
        );

        let differ = SchemaDiffer::new();
        let diff = differ.diff(&from, &to);

        assert_eq!(diff.tables_to_create.len(), 1);
        assert_eq!(diff.tables_to_create[0].name, "users");
        assert!(diff.tables_to_drop.is_empty());
    }

    #[test]
    fn test_schema_diff_drop_table() {
        let mut from = Schema::new();
        from.add_table(Table::new("old_table"));

        let to = Schema::new();

        let differ = SchemaDiffer::new();
        let diff = differ.diff(&from, &to);

        assert!(diff.tables_to_create.is_empty());
        assert_eq!(diff.tables_to_drop.len(), 1);
        assert_eq!(diff.tables_to_drop[0], "old_table");
    }

    #[test]
    fn test_schema_diff_modify_table() {
        let mut from = Schema::new();
        from.add_table(
            Table::new("users")
                .column(Column::new("id", ColumnType::BigSerial).not_null()),
        );

        let mut to = Schema::new();
        to.add_table(
            Table::new("users")
                .column(Column::new("id", ColumnType::BigSerial).not_null())
                .column(Column::new("email", ColumnType::Varchar(Some(255))).not_null()),
        );

        let differ = SchemaDiffer::new();
        let diff = differ.diff(&from, &to);

        assert_eq!(diff.table_modifications.len(), 1);
        assert_eq!(diff.table_modifications[0].columns_to_add.len(), 1);
        assert_eq!(diff.table_modifications[0].columns_to_add[0].name, "email");
    }
}
