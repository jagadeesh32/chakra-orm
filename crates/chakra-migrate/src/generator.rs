//! Migration generator for auto-detecting schema changes

use crate::file::generate_migration_id;
use crate::migration::Migration;
use chakra_core::model::ModelMeta;
use chakra_schema::diff::{SchemaDiff, SchemaDiffer};
use chakra_schema::schema::{
    Column, ColumnDefault, ColumnType, ForeignKey, Index, PrimaryKey, Schema, Table,
};
use std::collections::HashMap;
use tracing::{debug, info};

/// Migration generator for auto-detecting schema changes
#[derive(Debug, Default)]
pub struct MigrationGenerator {
    /// Whether to generate reversible migrations
    pub reversible: bool,
    /// App name for the migration
    pub app: Option<String>,
    /// Tables to exclude from comparison
    pub exclude_tables: Vec<String>,
}

impl MigrationGenerator {
    /// Create a new generator
    pub fn new() -> Self {
        Self {
            reversible: true,
            app: None,
            exclude_tables: vec!["chakra_migrations".to_string()],
        }
    }

    /// Set app name
    pub fn app(mut self, app: impl Into<String>) -> Self {
        self.app = Some(app.into());
        self
    }

    /// Set reversible
    pub fn reversible(mut self, reversible: bool) -> Self {
        self.reversible = reversible;
        self
    }

    /// Exclude a table
    pub fn exclude_table(mut self, table: impl Into<String>) -> Self {
        self.exclude_tables.push(table.into());
        self
    }

    /// Generate a migration from model metadata
    pub fn from_models(&self, models: &[&ModelMeta], current_schema: &Schema) -> Option<Migration> {
        let target_schema = self.models_to_schema(models);
        self.from_schema_diff(current_schema, &target_schema)
    }

    /// Generate a migration from a schema diff
    pub fn from_schema_diff(&self, from: &Schema, to: &Schema) -> Option<Migration> {
        let mut differ = SchemaDiffer::new();

        for table in &self.exclude_tables {
            differ = differ.exclude_table(table);
        }

        let diff = differ.diff(from, to);

        if diff.is_empty() {
            debug!("No schema changes detected");
            return None;
        }

        let name = self.generate_name(&diff);
        let id = generate_migration_id();

        let mut migration = Migration::new(&id, &name);
        migration.app = self.app.clone();
        migration.reversible = self.reversible;

        // Convert diff to operations
        for table in &diff.tables_to_create {
            migration.operations.push(
                chakra_schema::diff::MigrationOperation::CreateTable(table.clone()),
            );
        }

        for table_name in &diff.tables_to_drop {
            migration.operations.push(
                chakra_schema::diff::MigrationOperation::DropTable {
                    name: table_name.clone(),
                    cascade: true,
                },
            );
        }

        for table_diff in &diff.table_modifications {
            for column in &table_diff.columns_to_add {
                migration.operations.push(
                    chakra_schema::diff::MigrationOperation::AddColumn {
                        table: table_diff.table_name.clone(),
                        column: column.clone(),
                    },
                );
            }

            for column_name in &table_diff.columns_to_drop {
                migration.operations.push(
                    chakra_schema::diff::MigrationOperation::DropColumn {
                        table: table_diff.table_name.clone(),
                        column: column_name.clone(),
                    },
                );
            }

            for (old, new) in &table_diff.columns_to_modify {
                migration.operations.push(
                    chakra_schema::diff::MigrationOperation::AlterColumn {
                        table: table_diff.table_name.clone(),
                        from: old.clone(),
                        to: new.clone(),
                    },
                );
            }

            for index in &table_diff.indexes_to_create {
                migration.operations.push(
                    chakra_schema::diff::MigrationOperation::CreateIndex {
                        table: table_diff.table_name.clone(),
                        index: index.clone(),
                    },
                );
            }

            for index_name in &table_diff.indexes_to_drop {
                migration.operations.push(
                    chakra_schema::diff::MigrationOperation::DropIndex {
                        name: index_name.clone(),
                    },
                );
            }

            for constraint in &table_diff.constraints_to_add {
                migration.operations.push(
                    chakra_schema::diff::MigrationOperation::AddConstraint {
                        table: table_diff.table_name.clone(),
                        constraint: constraint.clone(),
                    },
                );
            }

            for fk in &table_diff.foreign_keys_to_add {
                migration.operations.push(
                    chakra_schema::diff::MigrationOperation::AddForeignKey {
                        table: table_diff.table_name.clone(),
                        foreign_key: fk.clone(),
                    },
                );
            }
        }

        let migration = migration.with_checksum();
        info!(
            "Generated migration {} with {} operations",
            migration.id,
            migration.operations.len()
        );

        Some(migration)
    }

    /// Convert model metadata to a schema
    fn models_to_schema(&self, models: &[&ModelMeta]) -> Schema {
        let mut schema = Schema::new();

        for model in models {
            let table = self.model_to_table(model);
            schema.add_table(table);
        }

        schema
    }

    /// Convert a single model to a table
    fn model_to_table(&self, model: &ModelMeta) -> Table {
        let mut table = Table::new(&model.table);

        if let Some(ref schema_name) = model.schema {
            table.schema = Some(schema_name.clone());
        }

        // Add columns
        for field in &model.fields {
            let column_type = ColumnType::from_field_type(&field.field_type);

            let mut column = Column::new(field.column_name(), column_type);
            column.nullable = field.nullable;
            column.auto_increment = field.auto_increment;

            if let Some(ref default) = field.default {
                column.default = Some(self.convert_default(default));
            }

            table.add_column(column);
        }

        // Set primary key
        if !model.primary_key.is_empty() {
            table.primary_key = Some(PrimaryKey::new(model.primary_key.clone()));
        }

        // Add indexes
        for index_meta in &model.indexes {
            let index = Index::new(&index_meta.name, index_meta.columns.clone());
            table.add_index(if index_meta.unique {
                index.unique()
            } else {
                index
            });
        }

        // Add foreign keys from field definitions
        for field in &model.fields {
            if let Some(ref fk) = field.foreign_key {
                let foreign_key = ForeignKey::new(
                    vec![field.column_name().to_string()],
                    &fk.table,
                    vec![fk.column.clone()],
                )
                .on_delete(fk.on_delete.clone())
                .on_update(fk.on_update.clone());

                table.add_foreign_key(foreign_key);
            }
        }

        table
    }

    /// Convert a field default to a column default
    fn convert_default(&self, default: &chakra_core::model::FieldDefault) -> ColumnDefault {
        match default {
            chakra_core::model::FieldDefault::Value(v) => {
                ColumnDefault::Expression(format!("{:?}", v))
            }
            chakra_core::model::FieldDefault::Expression(expr) => {
                ColumnDefault::Expression(expr.clone())
            }
            chakra_core::model::FieldDefault::AutoIncrement => {
                ColumnDefault::Expression("DEFAULT".to_string())
            }
            chakra_core::model::FieldDefault::Uuid => ColumnDefault::GenerateUuid,
        }
    }

    /// Generate a descriptive name for the migration
    fn generate_name(&self, diff: &SchemaDiff) -> String {
        let mut parts = Vec::new();

        if !diff.tables_to_create.is_empty() {
            let tables: Vec<_> = diff.tables_to_create.iter().map(|t| t.name.as_str()).collect();
            parts.push(format!("create_{}", tables.join("_")));
        }

        if !diff.tables_to_drop.is_empty() {
            parts.push(format!("drop_{}", diff.tables_to_drop.join("_")));
        }

        for mod_diff in &diff.table_modifications {
            if !mod_diff.columns_to_add.is_empty() {
                let cols: Vec<_> = mod_diff.columns_to_add.iter().map(|c| c.name.as_str()).collect();
                parts.push(format!(
                    "add_{}_to_{}",
                    cols.join("_"),
                    mod_diff.table_name
                ));
            }

            if !mod_diff.columns_to_drop.is_empty() {
                parts.push(format!(
                    "drop_{}_from_{}",
                    mod_diff.columns_to_drop.join("_"),
                    mod_diff.table_name
                ));
            }
        }

        if parts.is_empty() {
            "schema_changes".to_string()
        } else if parts.len() == 1 {
            parts.remove(0)
        } else {
            "multiple_changes".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chakra_core::types::FieldType;

    fn create_test_model() -> ModelMeta {
        chakra_core::model::ModelMeta::builder("User", "users")
            .field(
                chakra_core::model::FieldMeta::builder("id", FieldType::BigInt)
                    .primary_key()
                    .auto_increment()
                    .build(),
            )
            .field(
                chakra_core::model::FieldMeta::builder("name", FieldType::string(100))
                    .build(),
            )
            .build()
    }

    #[test]
    fn test_model_to_table() {
        let model = create_test_model();
        let generator = MigrationGenerator::new();
        let table = generator.model_to_table(&model);

        assert_eq!(table.name, "users");
        assert_eq!(table.columns.len(), 2);
        assert!(table.primary_key.is_some());
    }

    #[test]
    fn test_generate_from_empty() {
        let model = create_test_model();
        let generator = MigrationGenerator::new().app("core");

        let current = Schema::new();
        let migration = generator.from_models(&[&model], &current);

        assert!(migration.is_some());
        let m = migration.unwrap();
        assert!(!m.operations.is_empty());
        assert_eq!(m.app, Some("core".to_string()));
    }
}
