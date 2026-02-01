//! DDL generation for Chakra ORM
//!
//! This module provides DDL statement generation for schema changes.

use crate::schema::{
    Column, ColumnDefault, ColumnType, Constraint, ConstraintType, ForeignKey, Index, PrimaryKey,
    Schema, Table,
};
use chakra_core::model::ForeignKeyAction;
use serde::{Deserialize, Serialize};

/// A DDL statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdlStatement {
    /// The SQL statement
    pub sql: String,
    /// Whether this is reversible
    pub reversible: bool,
    /// The reverse SQL statement (if reversible)
    pub reverse_sql: Option<String>,
    /// Description of what this statement does
    pub description: Option<String>,
}

impl DdlStatement {
    /// Create a new DDL statement
    pub fn new(sql: impl Into<String>) -> Self {
        Self {
            sql: sql.into(),
            reversible: false,
            reverse_sql: None,
            description: None,
        }
    }

    /// Set reversibility
    pub fn reversible(mut self, reverse_sql: impl Into<String>) -> Self {
        self.reversible = true;
        self.reverse_sql = Some(reverse_sql.into());
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// DDL generator for different database dialects
pub trait DdlGenerator: Send + Sync {
    /// Generate CREATE TABLE statement
    fn create_table(&self, table: &Table) -> DdlStatement;

    /// Generate DROP TABLE statement
    fn drop_table(&self, table_name: &str, cascade: bool) -> DdlStatement;

    /// Generate ALTER TABLE ADD COLUMN statement
    fn add_column(&self, table_name: &str, column: &Column) -> DdlStatement;

    /// Generate ALTER TABLE DROP COLUMN statement
    fn drop_column(&self, table_name: &str, column_name: &str) -> DdlStatement;

    /// Generate ALTER TABLE ALTER COLUMN statement
    fn alter_column(&self, table_name: &str, old: &Column, new: &Column) -> Vec<DdlStatement>;

    /// Generate CREATE INDEX statement
    fn create_index(&self, table_name: &str, index: &Index) -> DdlStatement;

    /// Generate DROP INDEX statement
    fn drop_index(&self, index_name: &str) -> DdlStatement;

    /// Generate ADD CONSTRAINT statement
    fn add_constraint(&self, table_name: &str, constraint: &Constraint) -> DdlStatement;

    /// Generate DROP CONSTRAINT statement
    fn drop_constraint(&self, table_name: &str, constraint_name: &str) -> DdlStatement;

    /// Generate ADD FOREIGN KEY statement
    fn add_foreign_key(&self, table_name: &str, fk: &ForeignKey) -> DdlStatement;

    /// Generate DROP FOREIGN KEY statement
    fn drop_foreign_key(&self, table_name: &str, fk_name: &str) -> DdlStatement;

    /// Generate RENAME TABLE statement
    fn rename_table(&self, old_name: &str, new_name: &str) -> DdlStatement;

    /// Generate RENAME COLUMN statement
    fn rename_column(&self, table_name: &str, old_name: &str, new_name: &str) -> DdlStatement;
}

/// PostgreSQL DDL generator
#[derive(Debug, Clone, Default)]
pub struct PostgresDdlGenerator;

impl DdlGenerator for PostgresDdlGenerator {
    fn create_table(&self, table: &Table) -> DdlStatement {
        let mut sql = String::new();
        sql.push_str("CREATE TABLE ");
        sql.push_str(&quote_identifier(&table.name));
        sql.push_str(" (\n");

        // Columns
        let mut parts = Vec::new();
        for column in &table.columns {
            parts.push(format!("    {}", self.column_definition(column)));
        }

        // Primary key
        if let Some(pk) = &table.primary_key {
            let cols: Vec<String> = pk.columns.iter().map(|c| quote_identifier(c)).collect();
            let pk_def = if let Some(name) = &pk.name {
                format!(
                    "    CONSTRAINT {} PRIMARY KEY ({})",
                    quote_identifier(name),
                    cols.join(", ")
                )
            } else {
                format!("    PRIMARY KEY ({})", cols.join(", "))
            };
            parts.push(pk_def);
        }

        // Constraints
        for constraint in &table.constraints {
            parts.push(format!("    {}", self.constraint_definition(constraint)));
        }

        // Foreign keys
        for fk in &table.foreign_keys {
            parts.push(format!("    {}", self.foreign_key_definition(fk)));
        }

        sql.push_str(&parts.join(",\n"));
        sql.push_str("\n)");

        let drop_sql = format!("DROP TABLE {}", quote_identifier(&table.name));

        DdlStatement::new(sql)
            .reversible(drop_sql)
            .description(format!("Create table {}", table.name))
    }

    fn drop_table(&self, table_name: &str, cascade: bool) -> DdlStatement {
        let sql = if cascade {
            format!("DROP TABLE {} CASCADE", quote_identifier(table_name))
        } else {
            format!("DROP TABLE {}", quote_identifier(table_name))
        };

        DdlStatement::new(sql).description(format!("Drop table {}", table_name))
    }

    fn add_column(&self, table_name: &str, column: &Column) -> DdlStatement {
        let sql = format!(
            "ALTER TABLE {} ADD COLUMN {}",
            quote_identifier(table_name),
            self.column_definition(column)
        );

        let reverse_sql = format!(
            "ALTER TABLE {} DROP COLUMN {}",
            quote_identifier(table_name),
            quote_identifier(&column.name)
        );

        DdlStatement::new(sql)
            .reversible(reverse_sql)
            .description(format!("Add column {} to {}", column.name, table_name))
    }

    fn drop_column(&self, table_name: &str, column_name: &str) -> DdlStatement {
        let sql = format!(
            "ALTER TABLE {} DROP COLUMN {}",
            quote_identifier(table_name),
            quote_identifier(column_name)
        );

        DdlStatement::new(sql).description(format!("Drop column {} from {}", column_name, table_name))
    }

    fn alter_column(&self, table_name: &str, old: &Column, new: &Column) -> Vec<DdlStatement> {
        let mut statements = Vec::new();
        let table = quote_identifier(table_name);
        let column = quote_identifier(&new.name);

        // Rename if needed
        if old.name != new.name {
            statements.push(
                DdlStatement::new(format!(
                    "ALTER TABLE {} RENAME COLUMN {} TO {}",
                    table,
                    quote_identifier(&old.name),
                    column
                ))
                .reversible(format!(
                    "ALTER TABLE {} RENAME COLUMN {} TO {}",
                    table,
                    column,
                    quote_identifier(&old.name)
                ))
                .description(format!(
                    "Rename column {} to {} in {}",
                    old.name, new.name, table_name
                )),
            );
        }

        // Change type if needed
        if old.column_type != new.column_type {
            let type_sql = new.column_type.to_postgres_sql();
            statements.push(
                DdlStatement::new(format!(
                    "ALTER TABLE {} ALTER COLUMN {} TYPE {} USING {}::{}",
                    table, column, type_sql, column, type_sql
                ))
                .description(format!(
                    "Change type of {} in {} to {}",
                    new.name, table_name, type_sql
                )),
            );
        }

        // Change nullability if needed
        if old.nullable != new.nullable {
            if new.nullable {
                statements.push(
                    DdlStatement::new(format!(
                        "ALTER TABLE {} ALTER COLUMN {} DROP NOT NULL",
                        table, column
                    ))
                    .reversible(format!(
                        "ALTER TABLE {} ALTER COLUMN {} SET NOT NULL",
                        table, column
                    ))
                    .description(format!("Make {} nullable in {}", new.name, table_name)),
                );
            } else {
                statements.push(
                    DdlStatement::new(format!(
                        "ALTER TABLE {} ALTER COLUMN {} SET NOT NULL",
                        table, column
                    ))
                    .reversible(format!(
                        "ALTER TABLE {} ALTER COLUMN {} DROP NOT NULL",
                        table, column
                    ))
                    .description(format!("Make {} not null in {}", new.name, table_name)),
                );
            }
        }

        // Change default if needed
        if old.default != new.default {
            match &new.default {
                Some(default) => {
                    statements.push(
                        DdlStatement::new(format!(
                            "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT {}",
                            table,
                            column,
                            default.to_sql()
                        ))
                        .description(format!(
                            "Set default for {} in {}",
                            new.name, table_name
                        )),
                    );
                }
                None => {
                    statements.push(
                        DdlStatement::new(format!(
                            "ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT",
                            table, column
                        ))
                        .description(format!(
                            "Drop default for {} in {}",
                            new.name, table_name
                        )),
                    );
                }
            }
        }

        statements
    }

    fn create_index(&self, table_name: &str, index: &Index) -> DdlStatement {
        let mut sql = String::new();

        if index.unique {
            sql.push_str("CREATE UNIQUE INDEX ");
        } else {
            sql.push_str("CREATE INDEX ");
        }

        sql.push_str(&quote_identifier(&index.name));
        sql.push_str(" ON ");
        sql.push_str(&quote_identifier(table_name));

        if let Some(method) = &index.method {
            sql.push_str(" USING ");
            sql.push_str(method);
        }

        sql.push_str(" (");
        let cols: Vec<String> = index
            .columns
            .iter()
            .map(|c| {
                let mut col = quote_identifier(&c.name);
                if let Some(order) = &c.order {
                    col.push_str(match order {
                        crate::schema::IndexOrder::Asc => " ASC",
                        crate::schema::IndexOrder::Desc => " DESC",
                    });
                }
                if let Some(nulls) = &c.nulls {
                    col.push_str(match nulls {
                        crate::schema::NullsOrder::First => " NULLS FIRST",
                        crate::schema::NullsOrder::Last => " NULLS LAST",
                    });
                }
                col
            })
            .collect();
        sql.push_str(&cols.join(", "));
        sql.push(')');

        if let Some(where_clause) = &index.where_clause {
            sql.push_str(" WHERE ");
            sql.push_str(where_clause);
        }

        let drop_sql = format!("DROP INDEX {}", quote_identifier(&index.name));

        DdlStatement::new(sql)
            .reversible(drop_sql)
            .description(format!("Create index {} on {}", index.name, table_name))
    }

    fn drop_index(&self, index_name: &str) -> DdlStatement {
        DdlStatement::new(format!("DROP INDEX {}", quote_identifier(index_name)))
            .description(format!("Drop index {}", index_name))
    }

    fn add_constraint(&self, table_name: &str, constraint: &Constraint) -> DdlStatement {
        let sql = format!(
            "ALTER TABLE {} ADD {}",
            quote_identifier(table_name),
            self.constraint_definition(constraint)
        );

        let reverse_sql = format!(
            "ALTER TABLE {} DROP CONSTRAINT {}",
            quote_identifier(table_name),
            quote_identifier(&constraint.name)
        );

        DdlStatement::new(sql)
            .reversible(reverse_sql)
            .description(format!(
                "Add constraint {} to {}",
                constraint.name, table_name
            ))
    }

    fn drop_constraint(&self, table_name: &str, constraint_name: &str) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} DROP CONSTRAINT {}",
            quote_identifier(table_name),
            quote_identifier(constraint_name)
        ))
        .description(format!(
            "Drop constraint {} from {}",
            constraint_name, table_name
        ))
    }

    fn add_foreign_key(&self, table_name: &str, fk: &ForeignKey) -> DdlStatement {
        let sql = format!(
            "ALTER TABLE {} ADD {}",
            quote_identifier(table_name),
            self.foreign_key_definition(fk)
        );

        let fk_name = fk
            .name
            .clone()
            .unwrap_or_else(|| format!("fk_{}_{}", table_name, fk.columns.join("_")));

        let reverse_sql = format!(
            "ALTER TABLE {} DROP CONSTRAINT {}",
            quote_identifier(table_name),
            quote_identifier(&fk_name)
        );

        DdlStatement::new(sql)
            .reversible(reverse_sql)
            .description(format!(
                "Add foreign key on {} referencing {}",
                table_name, fk.references_table
            ))
    }

    fn drop_foreign_key(&self, table_name: &str, fk_name: &str) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} DROP CONSTRAINT {}",
            quote_identifier(table_name),
            quote_identifier(fk_name)
        ))
        .description(format!(
            "Drop foreign key {} from {}",
            fk_name, table_name
        ))
    }

    fn rename_table(&self, old_name: &str, new_name: &str) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} RENAME TO {}",
            quote_identifier(old_name),
            quote_identifier(new_name)
        ))
        .reversible(format!(
            "ALTER TABLE {} RENAME TO {}",
            quote_identifier(new_name),
            quote_identifier(old_name)
        ))
        .description(format!("Rename table {} to {}", old_name, new_name))
    }

    fn rename_column(&self, table_name: &str, old_name: &str, new_name: &str) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {}",
            quote_identifier(table_name),
            quote_identifier(old_name),
            quote_identifier(new_name)
        ))
        .reversible(format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {}",
            quote_identifier(table_name),
            quote_identifier(new_name),
            quote_identifier(old_name)
        ))
        .description(format!(
            "Rename column {} to {} in {}",
            old_name, new_name, table_name
        ))
    }
}

impl PostgresDdlGenerator {
    fn column_definition(&self, column: &Column) -> String {
        let mut def = String::new();
        def.push_str(&quote_identifier(&column.name));
        def.push(' ');
        def.push_str(&column.column_type.to_postgres_sql());

        if !column.nullable {
            def.push_str(" NOT NULL");
        }

        if let Some(default) = &column.default {
            def.push_str(" DEFAULT ");
            def.push_str(&default.to_sql());
        }

        def
    }

    fn constraint_definition(&self, constraint: &Constraint) -> String {
        match &constraint.constraint_type {
            ConstraintType::Unique { columns } => {
                let cols: Vec<String> = columns.iter().map(|c| quote_identifier(c)).collect();
                format!(
                    "CONSTRAINT {} UNIQUE ({})",
                    quote_identifier(&constraint.name),
                    cols.join(", ")
                )
            }
            ConstraintType::Check { expression } => {
                format!(
                    "CONSTRAINT {} CHECK ({})",
                    quote_identifier(&constraint.name),
                    expression
                )
            }
            ConstraintType::Exclusion { expression } => {
                format!(
                    "CONSTRAINT {} EXCLUDE ({})",
                    quote_identifier(&constraint.name),
                    expression
                )
            }
        }
    }

    fn foreign_key_definition(&self, fk: &ForeignKey) -> String {
        let local_cols: Vec<String> = fk.columns.iter().map(|c| quote_identifier(c)).collect();
        let ref_cols: Vec<String> = fk
            .references_columns
            .iter()
            .map(|c| quote_identifier(c))
            .collect();

        let mut def = String::new();

        if let Some(name) = &fk.name {
            def.push_str("CONSTRAINT ");
            def.push_str(&quote_identifier(name));
            def.push(' ');
        }

        def.push_str("FOREIGN KEY (");
        def.push_str(&local_cols.join(", "));
        def.push_str(") REFERENCES ");
        def.push_str(&quote_identifier(&fk.references_table));
        def.push_str(" (");
        def.push_str(&ref_cols.join(", "));
        def.push(')');

        if fk.on_delete != ForeignKeyAction::NoAction {
            def.push_str(" ON DELETE ");
            def.push_str(fk.on_delete.as_sql());
        }

        if fk.on_update != ForeignKeyAction::NoAction {
            def.push_str(" ON UPDATE ");
            def.push_str(fk.on_update.as_sql());
        }

        def
    }
}

/// Quote an identifier
fn quote_identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

/// MySQL DDL generator
#[derive(Debug, Clone, Default)]
pub struct MySqlDdlGenerator;

impl DdlGenerator for MySqlDdlGenerator {
    fn create_table(&self, table: &Table) -> DdlStatement {
        let mut sql = String::new();
        sql.push_str("CREATE TABLE ");
        sql.push_str(&quote_mysql_identifier(&table.name));
        sql.push_str(" (\n");

        let mut parts = Vec::new();
        for column in &table.columns {
            parts.push(format!("    {}", self.column_definition(column)));
        }

        if let Some(pk) = &table.primary_key {
            let cols: Vec<String> = pk.columns.iter().map(|c| quote_mysql_identifier(c)).collect();
            parts.push(format!("    PRIMARY KEY ({})", cols.join(", ")));
        }

        sql.push_str(&parts.join(",\n"));
        sql.push_str("\n) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4");

        DdlStatement::new(sql)
            .reversible(format!("DROP TABLE {}", quote_mysql_identifier(&table.name)))
            .description(format!("Create table {}", table.name))
    }

    fn drop_table(&self, table_name: &str, cascade: bool) -> DdlStatement {
        let sql = if cascade {
            format!("DROP TABLE {} CASCADE", quote_mysql_identifier(table_name))
        } else {
            format!("DROP TABLE {}", quote_mysql_identifier(table_name))
        };
        DdlStatement::new(sql)
    }

    fn add_column(&self, table_name: &str, column: &Column) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} ADD COLUMN {}",
            quote_mysql_identifier(table_name),
            self.column_definition(column)
        ))
        .reversible(format!(
            "ALTER TABLE {} DROP COLUMN {}",
            quote_mysql_identifier(table_name),
            quote_mysql_identifier(&column.name)
        ))
    }

    fn drop_column(&self, table_name: &str, column_name: &str) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} DROP COLUMN {}",
            quote_mysql_identifier(table_name),
            quote_mysql_identifier(column_name)
        ))
    }

    fn alter_column(&self, table_name: &str, _old: &Column, new: &Column) -> Vec<DdlStatement> {
        vec![DdlStatement::new(format!(
            "ALTER TABLE {} MODIFY COLUMN {}",
            quote_mysql_identifier(table_name),
            self.column_definition(new)
        ))]
    }

    fn create_index(&self, table_name: &str, index: &Index) -> DdlStatement {
        let mut sql = if index.unique {
            "CREATE UNIQUE INDEX ".to_string()
        } else {
            "CREATE INDEX ".to_string()
        };

        sql.push_str(&quote_mysql_identifier(&index.name));
        sql.push_str(" ON ");
        sql.push_str(&quote_mysql_identifier(table_name));
        sql.push_str(" (");
        let cols: Vec<String> = index
            .columns
            .iter()
            .map(|c| quote_mysql_identifier(&c.name))
            .collect();
        sql.push_str(&cols.join(", "));
        sql.push(')');

        DdlStatement::new(sql).reversible(format!(
            "DROP INDEX {} ON {}",
            quote_mysql_identifier(&index.name),
            quote_mysql_identifier(table_name)
        ))
    }

    fn drop_index(&self, index_name: &str) -> DdlStatement {
        DdlStatement::new(format!("DROP INDEX {}", quote_mysql_identifier(index_name)))
    }

    fn add_constraint(&self, table_name: &str, constraint: &Constraint) -> DdlStatement {
        let sql = match &constraint.constraint_type {
            ConstraintType::Unique { columns } => {
                let cols: Vec<String> = columns.iter().map(|c| quote_mysql_identifier(c)).collect();
                format!(
                    "ALTER TABLE {} ADD CONSTRAINT {} UNIQUE ({})",
                    quote_mysql_identifier(table_name),
                    quote_mysql_identifier(&constraint.name),
                    cols.join(", ")
                )
            }
            ConstraintType::Check { expression } => {
                format!(
                    "ALTER TABLE {} ADD CONSTRAINT {} CHECK ({})",
                    quote_mysql_identifier(table_name),
                    quote_mysql_identifier(&constraint.name),
                    expression
                )
            }
            ConstraintType::Exclusion { .. } => {
                // MySQL doesn't support exclusion constraints
                "-- Exclusion constraints not supported in MySQL".to_string()
            }
        };
        DdlStatement::new(sql)
    }

    fn drop_constraint(&self, table_name: &str, constraint_name: &str) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} DROP CONSTRAINT {}",
            quote_mysql_identifier(table_name),
            quote_mysql_identifier(constraint_name)
        ))
    }

    fn add_foreign_key(&self, table_name: &str, fk: &ForeignKey) -> DdlStatement {
        let local_cols: Vec<String> = fk
            .columns
            .iter()
            .map(|c| quote_mysql_identifier(c))
            .collect();
        let ref_cols: Vec<String> = fk
            .references_columns
            .iter()
            .map(|c| quote_mysql_identifier(c))
            .collect();

        let fk_name = fk
            .name
            .clone()
            .unwrap_or_else(|| format!("fk_{}_{}", table_name, fk.columns.join("_")));

        DdlStatement::new(format!(
            "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({}) ON DELETE {} ON UPDATE {}",
            quote_mysql_identifier(table_name),
            quote_mysql_identifier(&fk_name),
            local_cols.join(", "),
            quote_mysql_identifier(&fk.references_table),
            ref_cols.join(", "),
            fk.on_delete.as_sql(),
            fk.on_update.as_sql()
        ))
    }

    fn drop_foreign_key(&self, table_name: &str, fk_name: &str) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} DROP FOREIGN KEY {}",
            quote_mysql_identifier(table_name),
            quote_mysql_identifier(fk_name)
        ))
    }

    fn rename_table(&self, old_name: &str, new_name: &str) -> DdlStatement {
        DdlStatement::new(format!(
            "RENAME TABLE {} TO {}",
            quote_mysql_identifier(old_name),
            quote_mysql_identifier(new_name)
        ))
        .reversible(format!(
            "RENAME TABLE {} TO {}",
            quote_mysql_identifier(new_name),
            quote_mysql_identifier(old_name)
        ))
    }

    fn rename_column(&self, table_name: &str, old_name: &str, new_name: &str) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {}",
            quote_mysql_identifier(table_name),
            quote_mysql_identifier(old_name),
            quote_mysql_identifier(new_name)
        ))
        .reversible(format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {}",
            quote_mysql_identifier(table_name),
            quote_mysql_identifier(new_name),
            quote_mysql_identifier(old_name)
        ))
    }
}

impl MySqlDdlGenerator {
    fn column_definition(&self, column: &Column) -> String {
        let mut def = quote_mysql_identifier(&column.name);
        def.push(' ');
        def.push_str(&column.column_type.to_mysql_sql());

        if !column.nullable {
            def.push_str(" NOT NULL");
        }

        if column.auto_increment {
            def.push_str(" AUTO_INCREMENT");
        }

        if let Some(default) = &column.default {
            def.push_str(" DEFAULT ");
            def.push_str(&default.to_sql());
        }

        def
    }
}

/// Quote MySQL identifier with backticks
fn quote_mysql_identifier(name: &str) -> String {
    format!("`{}`", name.replace('`', "``"))
}

/// SQLite DDL generator
#[derive(Debug, Clone, Default)]
pub struct SqliteDdlGenerator;

impl DdlGenerator for SqliteDdlGenerator {
    fn create_table(&self, table: &Table) -> DdlStatement {
        let mut sql = String::new();
        sql.push_str("CREATE TABLE ");
        sql.push_str(&quote_identifier(&table.name));
        sql.push_str(" (\n");

        let mut parts = Vec::new();
        for column in &table.columns {
            parts.push(format!("    {}", self.column_definition(column, table)));
        }

        // Add composite primary key if not already defined on column
        if let Some(pk) = &table.primary_key {
            if pk.columns.len() > 1 {
                let cols: Vec<String> = pk.columns.iter().map(|c| quote_identifier(c)).collect();
                parts.push(format!("    PRIMARY KEY ({})", cols.join(", ")));
            }
        }

        sql.push_str(&parts.join(",\n"));
        sql.push_str("\n)");

        DdlStatement::new(sql)
            .reversible(format!("DROP TABLE {}", quote_identifier(&table.name)))
    }

    fn drop_table(&self, table_name: &str, _cascade: bool) -> DdlStatement {
        DdlStatement::new(format!("DROP TABLE {}", quote_identifier(table_name)))
    }

    fn add_column(&self, table_name: &str, column: &Column) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} ADD COLUMN {} {}{}",
            quote_identifier(table_name),
            quote_identifier(&column.name),
            column.column_type.to_sqlite_sql(),
            if column.nullable { "" } else { " NOT NULL" }
        ))
    }

    fn drop_column(&self, table_name: &str, column_name: &str) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} DROP COLUMN {}",
            quote_identifier(table_name),
            quote_identifier(column_name)
        ))
    }

    fn alter_column(&self, _table_name: &str, _old: &Column, _new: &Column) -> Vec<DdlStatement> {
        // SQLite doesn't support ALTER COLUMN directly
        // Would need to recreate the table
        vec![DdlStatement::new(
            "-- SQLite requires table recreation for column modifications",
        )]
    }

    fn create_index(&self, table_name: &str, index: &Index) -> DdlStatement {
        let mut sql = if index.unique {
            "CREATE UNIQUE INDEX ".to_string()
        } else {
            "CREATE INDEX ".to_string()
        };

        sql.push_str(&quote_identifier(&index.name));
        sql.push_str(" ON ");
        sql.push_str(&quote_identifier(table_name));
        sql.push_str(" (");
        let cols: Vec<String> = index
            .columns
            .iter()
            .map(|c| quote_identifier(&c.name))
            .collect();
        sql.push_str(&cols.join(", "));
        sql.push(')');

        DdlStatement::new(sql).reversible(format!("DROP INDEX {}", quote_identifier(&index.name)))
    }

    fn drop_index(&self, index_name: &str) -> DdlStatement {
        DdlStatement::new(format!("DROP INDEX {}", quote_identifier(index_name)))
    }

    fn add_constraint(&self, _table_name: &str, _constraint: &Constraint) -> DdlStatement {
        DdlStatement::new("-- SQLite doesn't support adding constraints after table creation")
    }

    fn drop_constraint(&self, _table_name: &str, _constraint_name: &str) -> DdlStatement {
        DdlStatement::new("-- SQLite doesn't support dropping constraints")
    }

    fn add_foreign_key(&self, _table_name: &str, _fk: &ForeignKey) -> DdlStatement {
        DdlStatement::new(
            "-- SQLite doesn't support adding foreign keys after table creation",
        )
    }

    fn drop_foreign_key(&self, _table_name: &str, _fk_name: &str) -> DdlStatement {
        DdlStatement::new("-- SQLite doesn't support dropping foreign keys")
    }

    fn rename_table(&self, old_name: &str, new_name: &str) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} RENAME TO {}",
            quote_identifier(old_name),
            quote_identifier(new_name)
        ))
        .reversible(format!(
            "ALTER TABLE {} RENAME TO {}",
            quote_identifier(new_name),
            quote_identifier(old_name)
        ))
    }

    fn rename_column(&self, table_name: &str, old_name: &str, new_name: &str) -> DdlStatement {
        DdlStatement::new(format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {}",
            quote_identifier(table_name),
            quote_identifier(old_name),
            quote_identifier(new_name)
        ))
        .reversible(format!(
            "ALTER TABLE {} RENAME COLUMN {} TO {}",
            quote_identifier(table_name),
            quote_identifier(new_name),
            quote_identifier(old_name)
        ))
    }
}

impl SqliteDdlGenerator {
    fn column_definition(&self, column: &Column, table: &Table) -> String {
        let mut def = quote_identifier(&column.name);
        def.push(' ');
        def.push_str(&column.column_type.to_sqlite_sql());

        // Check if this is a single-column primary key
        let is_pk = table.primary_key.as_ref().map_or(false, |pk| {
            pk.columns.len() == 1 && pk.columns[0] == column.name
        });

        if is_pk {
            def.push_str(" PRIMARY KEY");
            if column.auto_increment {
                def.push_str(" AUTOINCREMENT");
            }
        } else if !column.nullable {
            def.push_str(" NOT NULL");
        }

        if let Some(default) = &column.default {
            def.push_str(" DEFAULT ");
            def.push_str(&default.to_sql());
        }

        def
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_postgres_create_table() {
        let table = Table::new("users")
            .column(Column::new("id", ColumnType::BigSerial).not_null())
            .column(Column::new("name", ColumnType::Varchar(Some(100))).not_null())
            .primary_key(PrimaryKey::single("id"));

        let gen = PostgresDdlGenerator;
        let stmt = gen.create_table(&table);

        assert!(stmt.sql.contains("CREATE TABLE"));
        assert!(stmt.sql.contains("\"users\""));
        assert!(stmt.sql.contains("BIGSERIAL"));
        assert!(stmt.reversible);
    }

    #[test]
    fn test_postgres_add_column() {
        let column = Column::new("email", ColumnType::Varchar(Some(255))).not_null();

        let gen = PostgresDdlGenerator;
        let stmt = gen.add_column("users", &column);

        assert!(stmt.sql.contains("ALTER TABLE"));
        assert!(stmt.sql.contains("ADD COLUMN"));
        assert!(stmt.sql.contains("VARCHAR(255)"));
        assert!(stmt.reversible);
    }
}
