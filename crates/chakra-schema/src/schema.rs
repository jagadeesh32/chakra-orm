//! Schema representation for Chakra ORM
//!
//! This module provides database-agnostic schema representation.

use chakra_core::model::ForeignKeyAction;
use chakra_core::types::FieldType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete database schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Schema {
    /// Schema name (e.g., "public" for PostgreSQL)
    pub name: Option<String>,
    /// Tables in the schema
    pub tables: HashMap<String, Table>,
    /// Custom types (enums, composites)
    pub types: HashMap<String, CustomType>,
    /// Extensions (PostgreSQL-specific)
    pub extensions: Vec<String>,
}

impl Schema {
    /// Create a new empty schema
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a schema with a name
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Self::default()
        }
    }

    /// Add a table
    pub fn add_table(&mut self, table: Table) {
        self.tables.insert(table.name.clone(), table);
    }

    /// Get a table by name
    pub fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }

    /// Get a mutable table by name
    pub fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.tables.get_mut(name)
    }

    /// Check if table exists
    pub fn has_table(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }

    /// Remove a table
    pub fn remove_table(&mut self, name: &str) -> Option<Table> {
        self.tables.remove(name)
    }

    /// Get all table names
    pub fn table_names(&self) -> Vec<&str> {
        self.tables.keys().map(|s| s.as_str()).collect()
    }
}

/// A database table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// Table name
    pub name: String,
    /// Schema name
    pub schema: Option<String>,
    /// Columns
    pub columns: Vec<Column>,
    /// Primary key columns
    pub primary_key: Option<PrimaryKey>,
    /// Indexes
    pub indexes: Vec<Index>,
    /// Constraints
    pub constraints: Vec<Constraint>,
    /// Foreign keys
    pub foreign_keys: Vec<ForeignKey>,
    /// Table comment
    pub comment: Option<String>,
}

impl Table {
    /// Create a new table
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            schema: None,
            columns: Vec::new(),
            primary_key: None,
            indexes: Vec::new(),
            constraints: Vec::new(),
            foreign_keys: Vec::new(),
            comment: None,
        }
    }

    /// Set schema
    pub fn schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    /// Add a column
    pub fn add_column(&mut self, column: Column) {
        self.columns.push(column);
    }

    /// Add a column (builder pattern)
    pub fn column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    /// Set primary key
    pub fn primary_key(mut self, pk: PrimaryKey) -> Self {
        self.primary_key = Some(pk);
        self
    }

    /// Add an index
    pub fn add_index(&mut self, index: Index) {
        self.indexes.push(index);
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    /// Add a foreign key
    pub fn add_foreign_key(&mut self, fk: ForeignKey) {
        self.foreign_keys.push(fk);
    }

    /// Get column by name
    pub fn get_column(&self, name: &str) -> Option<&Column> {
        self.columns.iter().find(|c| c.name == name)
    }

    /// Get column by name (mutable)
    pub fn get_column_mut(&mut self, name: &str) -> Option<&mut Column> {
        self.columns.iter_mut().find(|c| c.name == name)
    }

    /// Get qualified name (schema.table)
    pub fn qualified_name(&self) -> String {
        match &self.schema {
            Some(schema) => format!("{}.{}", schema, self.name),
            None => self.name.clone(),
        }
    }
}

/// A database column
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Column name
    pub name: String,
    /// Column type
    pub column_type: ColumnType,
    /// Is nullable?
    pub nullable: bool,
    /// Default value
    pub default: Option<ColumnDefault>,
    /// Is auto-increment/serial?
    pub auto_increment: bool,
    /// Column comment
    pub comment: Option<String>,
}

impl Column {
    /// Create a new column
    pub fn new(name: impl Into<String>, column_type: ColumnType) -> Self {
        Self {
            name: name.into(),
            column_type,
            nullable: true,
            default: None,
            auto_increment: false,
            comment: None,
        }
    }

    /// Set nullable
    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    /// Set not null
    pub fn not_null(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Set default value
    pub fn default(mut self, default: ColumnDefault) -> Self {
        self.default = Some(default);
        self
    }

    /// Set default expression
    pub fn default_expr(mut self, expr: impl Into<String>) -> Self {
        self.default = Some(ColumnDefault::Expression(expr.into()));
        self
    }

    /// Set auto-increment
    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self
    }

    /// Set comment
    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }
}

/// Column type representation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnType {
    /// Small integer (2 bytes)
    SmallInt,
    /// Integer (4 bytes)
    Integer,
    /// Big integer (8 bytes)
    BigInt,
    /// Decimal/numeric with precision and scale
    Decimal { precision: u32, scale: u32 },
    /// Real/float (4 bytes)
    Real,
    /// Double precision (8 bytes)
    DoublePrecision,
    /// Fixed-length character
    Char(u32),
    /// Variable-length character
    Varchar(Option<u32>),
    /// Unlimited text
    Text,
    /// Boolean
    Boolean,
    /// Date
    Date,
    /// Time
    Time { with_timezone: bool },
    /// Timestamp
    Timestamp { with_timezone: bool },
    /// Interval
    Interval,
    /// UUID
    Uuid,
    /// JSON
    Json,
    /// JSONB (PostgreSQL)
    Jsonb,
    /// Binary data
    Bytea,
    /// Array of another type
    Array(Box<ColumnType>),
    /// Custom/enum type
    Custom(String),
    /// Serial (auto-increment integer)
    Serial,
    /// Big serial (auto-increment big integer)
    BigSerial,
}

impl ColumnType {
    /// Convert from FieldType
    pub fn from_field_type(field_type: &FieldType) -> Self {
        match field_type {
            FieldType::SmallInt => ColumnType::SmallInt,
            FieldType::Integer => ColumnType::Integer,
            FieldType::BigInt => ColumnType::BigInt,
            FieldType::Decimal { precision, scale } => ColumnType::Decimal {
                precision: *precision,
                scale: *scale,
            },
            FieldType::Float => ColumnType::Real,
            FieldType::Double => ColumnType::DoublePrecision,
            FieldType::Char { length } => ColumnType::Char(*length as u32),
            FieldType::String { max_length } => {
                ColumnType::Varchar(max_length.map(|l| l as u32))
            }
            FieldType::Text => ColumnType::Text,
            FieldType::Boolean => ColumnType::Boolean,
            FieldType::Date => ColumnType::Date,
            FieldType::Time => ColumnType::Time { with_timezone: false },
            FieldType::Timestamp => ColumnType::Timestamp { with_timezone: false },
            FieldType::TimestampTz => ColumnType::Timestamp { with_timezone: true },
            FieldType::Uuid => ColumnType::Uuid,
            FieldType::Json => ColumnType::Json,
            FieldType::JsonB => ColumnType::Jsonb,
            FieldType::Binary { .. } => ColumnType::Bytea,
            FieldType::Array { element_type } => {
                ColumnType::Array(Box::new(ColumnType::from_field_type(element_type)))
            }
            FieldType::Enum { .. } => ColumnType::Text, // Simplified for now
        }
    }

    /// Get SQL representation for PostgreSQL
    pub fn to_postgres_sql(&self) -> String {
        match self {
            ColumnType::SmallInt => "SMALLINT".to_string(),
            ColumnType::Integer => "INTEGER".to_string(),
            ColumnType::BigInt => "BIGINT".to_string(),
            ColumnType::Decimal { precision, scale } => {
                format!("DECIMAL({}, {})", precision, scale)
            }
            ColumnType::Real => "REAL".to_string(),
            ColumnType::DoublePrecision => "DOUBLE PRECISION".to_string(),
            ColumnType::Char(len) => format!("CHAR({})", len),
            ColumnType::Varchar(Some(len)) => format!("VARCHAR({})", len),
            ColumnType::Varchar(None) => "VARCHAR".to_string(),
            ColumnType::Text => "TEXT".to_string(),
            ColumnType::Boolean => "BOOLEAN".to_string(),
            ColumnType::Date => "DATE".to_string(),
            ColumnType::Time { with_timezone } => {
                if *with_timezone {
                    "TIME WITH TIME ZONE".to_string()
                } else {
                    "TIME".to_string()
                }
            }
            ColumnType::Timestamp { with_timezone } => {
                if *with_timezone {
                    "TIMESTAMP WITH TIME ZONE".to_string()
                } else {
                    "TIMESTAMP".to_string()
                }
            }
            ColumnType::Interval => "INTERVAL".to_string(),
            ColumnType::Uuid => "UUID".to_string(),
            ColumnType::Json => "JSON".to_string(),
            ColumnType::Jsonb => "JSONB".to_string(),
            ColumnType::Bytea => "BYTEA".to_string(),
            ColumnType::Array(inner) => format!("{}[]", inner.to_postgres_sql()),
            ColumnType::Custom(name) => name.clone(),
            ColumnType::Serial => "SERIAL".to_string(),
            ColumnType::BigSerial => "BIGSERIAL".to_string(),
        }
    }

    /// Get SQL representation for MySQL
    pub fn to_mysql_sql(&self) -> String {
        match self {
            ColumnType::SmallInt => "SMALLINT".to_string(),
            ColumnType::Integer => "INT".to_string(),
            ColumnType::BigInt => "BIGINT".to_string(),
            ColumnType::Decimal { precision, scale } => {
                format!("DECIMAL({}, {})", precision, scale)
            }
            ColumnType::Real => "FLOAT".to_string(),
            ColumnType::DoublePrecision => "DOUBLE".to_string(),
            ColumnType::Char(len) => format!("CHAR({})", len),
            ColumnType::Varchar(Some(len)) => format!("VARCHAR({})", len),
            ColumnType::Varchar(None) => "VARCHAR(255)".to_string(),
            ColumnType::Text => "TEXT".to_string(),
            ColumnType::Boolean => "TINYINT(1)".to_string(),
            ColumnType::Date => "DATE".to_string(),
            ColumnType::Time { .. } => "TIME".to_string(),
            ColumnType::Timestamp { .. } => "TIMESTAMP".to_string(),
            ColumnType::Interval => "VARCHAR(255)".to_string(), // MySQL doesn't have INTERVAL
            ColumnType::Uuid => "CHAR(36)".to_string(),
            ColumnType::Json => "JSON".to_string(),
            ColumnType::Jsonb => "JSON".to_string(),
            ColumnType::Bytea => "BLOB".to_string(),
            ColumnType::Array(_) => "JSON".to_string(), // MySQL uses JSON for arrays
            ColumnType::Custom(name) => name.clone(),
            ColumnType::Serial => "INT AUTO_INCREMENT".to_string(),
            ColumnType::BigSerial => "BIGINT AUTO_INCREMENT".to_string(),
        }
    }

    /// Get SQL representation for SQLite
    pub fn to_sqlite_sql(&self) -> String {
        match self {
            ColumnType::SmallInt | ColumnType::Integer | ColumnType::BigInt => {
                "INTEGER".to_string()
            }
            ColumnType::Decimal { .. } | ColumnType::Real | ColumnType::DoublePrecision => {
                "REAL".to_string()
            }
            ColumnType::Char(_) | ColumnType::Varchar(_) | ColumnType::Text => "TEXT".to_string(),
            ColumnType::Boolean => "INTEGER".to_string(),
            ColumnType::Date | ColumnType::Time { .. } | ColumnType::Timestamp { .. } => {
                "TEXT".to_string()
            }
            ColumnType::Interval => "TEXT".to_string(),
            ColumnType::Uuid => "TEXT".to_string(),
            ColumnType::Json | ColumnType::Jsonb => "TEXT".to_string(),
            ColumnType::Bytea => "BLOB".to_string(),
            ColumnType::Array(_) => "TEXT".to_string(), // SQLite uses JSON text for arrays
            ColumnType::Custom(name) => name.clone(),
            ColumnType::Serial | ColumnType::BigSerial => "INTEGER".to_string(),
        }
    }
}

/// Default value for a column
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColumnDefault {
    /// Null value
    Null,
    /// Boolean value
    Boolean(bool),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// SQL expression
    Expression(String),
    /// Current timestamp
    CurrentTimestamp,
    /// Generate UUID
    GenerateUuid,
}

impl ColumnDefault {
    /// Get SQL representation
    pub fn to_sql(&self) -> String {
        match self {
            ColumnDefault::Null => "NULL".to_string(),
            ColumnDefault::Boolean(b) => {
                if *b {
                    "TRUE".to_string()
                } else {
                    "FALSE".to_string()
                }
            }
            ColumnDefault::Integer(i) => i.to_string(),
            ColumnDefault::Float(f) => f.to_string(),
            ColumnDefault::String(s) => format!("'{}'", s.replace('\'', "''")),
            ColumnDefault::Expression(expr) => expr.clone(),
            ColumnDefault::CurrentTimestamp => "CURRENT_TIMESTAMP".to_string(),
            ColumnDefault::GenerateUuid => "gen_random_uuid()".to_string(),
        }
    }
}

/// Primary key definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimaryKey {
    /// Constraint name
    pub name: Option<String>,
    /// Columns in the primary key
    pub columns: Vec<String>,
}

impl PrimaryKey {
    /// Create a new primary key
    pub fn new(columns: Vec<String>) -> Self {
        Self {
            name: None,
            columns,
        }
    }

    /// Create with a single column
    pub fn single(column: impl Into<String>) -> Self {
        Self {
            name: None,
            columns: vec![column.into()],
        }
    }

    /// Set constraint name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Index definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Index {
    /// Index name
    pub name: String,
    /// Columns in the index
    pub columns: Vec<IndexColumn>,
    /// Is unique?
    pub unique: bool,
    /// Index method (btree, hash, gin, etc.)
    pub method: Option<String>,
    /// Partial index condition
    pub where_clause: Option<String>,
}

impl Index {
    /// Create a new index
    pub fn new(name: impl Into<String>, columns: Vec<impl Into<String>>) -> Self {
        Self {
            name: name.into(),
            columns: columns
                .into_iter()
                .map(|c| IndexColumn {
                    name: c.into(),
                    order: None,
                    nulls: None,
                })
                .collect(),
            unique: false,
            method: None,
            where_clause: None,
        }
    }

    /// Set unique
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Set method
    pub fn method(mut self, method: impl Into<String>) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Set where clause
    pub fn where_clause(mut self, clause: impl Into<String>) -> Self {
        self.where_clause = Some(clause.into());
        self
    }
}

/// Column in an index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexColumn {
    /// Column name
    pub name: String,
    /// Sort order
    pub order: Option<IndexOrder>,
    /// Nulls ordering
    pub nulls: Option<NullsOrder>,
}

/// Index sort order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndexOrder {
    Asc,
    Desc,
}

/// Nulls ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NullsOrder {
    First,
    Last,
}

/// Constraint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    /// Constraint name
    pub name: String,
    /// Constraint type
    pub constraint_type: ConstraintType,
}

/// Constraint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    /// Unique constraint
    Unique { columns: Vec<String> },
    /// Check constraint
    Check { expression: String },
    /// Exclusion constraint (PostgreSQL)
    Exclusion { expression: String },
}

/// Foreign key definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKey {
    /// Constraint name
    pub name: Option<String>,
    /// Columns in this table
    pub columns: Vec<String>,
    /// Referenced table
    pub references_table: String,
    /// Referenced columns
    pub references_columns: Vec<String>,
    /// On delete action
    pub on_delete: ForeignKeyAction,
    /// On update action
    pub on_update: ForeignKeyAction,
}

impl ForeignKey {
    /// Create a new foreign key
    pub fn new(
        columns: Vec<String>,
        references_table: impl Into<String>,
        references_columns: Vec<String>,
    ) -> Self {
        Self {
            name: None,
            columns,
            references_table: references_table.into(),
            references_columns,
            on_delete: ForeignKeyAction::NoAction,
            on_update: ForeignKeyAction::NoAction,
        }
    }

    /// Set constraint name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set on delete action
    pub fn on_delete(mut self, action: ForeignKeyAction) -> Self {
        self.on_delete = action;
        self
    }

    /// Set on update action
    pub fn on_update(mut self, action: ForeignKeyAction) -> Self {
        self.on_update = action;
        self
    }
}

/// Custom type (enum, composite, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomType {
    /// Enum type
    Enum {
        name: String,
        values: Vec<String>,
    },
    /// Composite type
    Composite {
        name: String,
        fields: Vec<(String, ColumnType)>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_creation() {
        let mut schema = Schema::with_name("public");

        let table = Table::new("users")
            .column(Column::new("id", ColumnType::BigSerial).not_null())
            .column(Column::new("name", ColumnType::Varchar(Some(100))).not_null())
            .column(Column::new("email", ColumnType::Varchar(Some(255))).not_null())
            .primary_key(PrimaryKey::single("id"));

        schema.add_table(table);

        assert!(schema.has_table("users"));
        assert_eq!(schema.get_table("users").unwrap().columns.len(), 3);
    }

    #[test]
    fn test_column_type_sql() {
        assert_eq!(ColumnType::BigInt.to_postgres_sql(), "BIGINT");
        assert_eq!(ColumnType::BigInt.to_mysql_sql(), "BIGINT");
        assert_eq!(ColumnType::BigInt.to_sqlite_sql(), "INTEGER");

        assert_eq!(
            ColumnType::Varchar(Some(100)).to_postgres_sql(),
            "VARCHAR(100)"
        );
        assert_eq!(
            ColumnType::Timestamp { with_timezone: true }.to_postgres_sql(),
            "TIMESTAMP WITH TIME ZONE"
        );
    }
}
