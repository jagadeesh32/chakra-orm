//! Schema introspection for Chakra ORM
//!
//! This module provides traits and implementations for introspecting database schemas.

use crate::schema::{
    Column, ColumnDefault, ColumnType, Constraint, ConstraintType, ForeignKey, Index,
    IndexColumn, IndexOrder, NullsOrder, PrimaryKey, Schema, Table,
};
use async_trait::async_trait;
use chakra_core::error::Result;
use serde::{Deserialize, Serialize};

/// Trait for schema introspection
#[async_trait]
pub trait SchemaIntrospector: Send + Sync {
    /// Introspect the entire database schema
    async fn introspect(&self) -> Result<Schema>;

    /// Introspect a specific schema (namespace)
    async fn introspect_schema(&self, schema_name: &str) -> Result<Schema>;

    /// Introspect a single table
    async fn introspect_table(&self, table_name: &str) -> Result<Table>;

    /// Get list of all schemas
    async fn list_schemas(&self) -> Result<Vec<String>>;

    /// Get list of all tables in a schema
    async fn list_tables(&self, schema_name: Option<&str>) -> Result<Vec<String>>;

    /// Check if a table exists
    async fn table_exists(&self, table_name: &str) -> Result<bool>;
}

/// Raw table information from introspection query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTableInfo {
    pub schema_name: Option<String>,
    pub table_name: String,
    pub table_type: String,
    pub comment: Option<String>,
}

/// Raw column information from introspection query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawColumnInfo {
    pub table_name: String,
    pub column_name: String,
    pub ordinal_position: i32,
    pub column_default: Option<String>,
    pub is_nullable: bool,
    pub data_type: String,
    pub character_maximum_length: Option<i32>,
    pub numeric_precision: Option<i32>,
    pub numeric_scale: Option<i32>,
    pub is_identity: bool,
    pub identity_generation: Option<String>,
    pub comment: Option<String>,
}

impl RawColumnInfo {
    /// Convert to Column
    pub fn to_column(&self) -> Column {
        let column_type = parse_column_type(
            &self.data_type,
            self.character_maximum_length,
            self.numeric_precision,
            self.numeric_scale,
        );

        let default = self.column_default.as_ref().map(|d| parse_default(d));

        Column {
            name: self.column_name.clone(),
            column_type,
            nullable: self.is_nullable,
            default,
            auto_increment: self.is_identity
                || self
                    .column_default
                    .as_ref()
                    .map(|d| d.contains("nextval"))
                    .unwrap_or(false),
            comment: self.comment.clone(),
        }
    }
}

/// Raw index information from introspection query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawIndexInfo {
    pub table_name: String,
    pub index_name: String,
    pub is_unique: bool,
    pub is_primary: bool,
    pub index_type: Option<String>,
    pub columns: Vec<RawIndexColumnInfo>,
    pub where_clause: Option<String>,
}

/// Raw index column information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawIndexColumnInfo {
    pub column_name: String,
    pub ordinal_position: i32,
    pub sort_order: Option<String>,
    pub nulls_order: Option<String>,
}

impl RawIndexInfo {
    /// Convert to Index
    pub fn to_index(&self) -> Index {
        Index {
            name: self.index_name.clone(),
            columns: self
                .columns
                .iter()
                .map(|c| IndexColumn {
                    name: c.column_name.clone(),
                    order: c.sort_order.as_ref().and_then(|o| match o.as_str() {
                        "ASC" => Some(IndexOrder::Asc),
                        "DESC" => Some(IndexOrder::Desc),
                        _ => None,
                    }),
                    nulls: c.nulls_order.as_ref().and_then(|o| match o.as_str() {
                        "FIRST" => Some(NullsOrder::First),
                        "LAST" => Some(NullsOrder::Last),
                        _ => None,
                    }),
                })
                .collect(),
            unique: self.is_unique,
            method: self.index_type.clone(),
            where_clause: self.where_clause.clone(),
        }
    }
}

/// Raw constraint information from introspection query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawConstraintInfo {
    pub table_name: String,
    pub constraint_name: String,
    pub constraint_type: String,
    pub columns: Vec<String>,
    pub check_expression: Option<String>,
    pub references_table: Option<String>,
    pub references_columns: Option<Vec<String>>,
    pub on_delete: Option<String>,
    pub on_update: Option<String>,
}

impl RawConstraintInfo {
    /// Convert to PrimaryKey if applicable
    pub fn to_primary_key(&self) -> Option<PrimaryKey> {
        if self.constraint_type == "PRIMARY KEY" {
            Some(PrimaryKey {
                name: Some(self.constraint_name.clone()),
                columns: self.columns.clone(),
            })
        } else {
            None
        }
    }

    /// Convert to Constraint if applicable
    pub fn to_constraint(&self) -> Option<Constraint> {
        match self.constraint_type.as_str() {
            "UNIQUE" => Some(Constraint {
                name: self.constraint_name.clone(),
                constraint_type: ConstraintType::Unique {
                    columns: self.columns.clone(),
                },
            }),
            "CHECK" => self.check_expression.as_ref().map(|expr| Constraint {
                name: self.constraint_name.clone(),
                constraint_type: ConstraintType::Check {
                    expression: expr.clone(),
                },
            }),
            _ => None,
        }
    }

    /// Convert to ForeignKey if applicable
    pub fn to_foreign_key(&self) -> Option<ForeignKey> {
        if self.constraint_type == "FOREIGN KEY" {
            Some(ForeignKey {
                name: Some(self.constraint_name.clone()),
                columns: self.columns.clone(),
                references_table: self.references_table.clone().unwrap_or_default(),
                references_columns: self.references_columns.clone().unwrap_or_default(),
                on_delete: parse_fk_action(self.on_delete.as_deref()),
                on_update: parse_fk_action(self.on_update.as_deref()),
            })
        } else {
            None
        }
    }
}

/// Parse column type from database type string
fn parse_column_type(
    data_type: &str,
    char_length: Option<i32>,
    precision: Option<i32>,
    scale: Option<i32>,
) -> ColumnType {
    let dt = data_type.to_uppercase();
    let dt = dt.as_str();

    match dt {
        "SMALLINT" | "INT2" => ColumnType::SmallInt,
        "INTEGER" | "INT" | "INT4" => ColumnType::Integer,
        "BIGINT" | "INT8" => ColumnType::BigInt,
        "SERIAL" => ColumnType::Serial,
        "BIGSERIAL" => ColumnType::BigSerial,
        "DECIMAL" | "NUMERIC" => ColumnType::Decimal {
            precision: precision.unwrap_or(18) as u32,
            scale: scale.unwrap_or(2) as u32,
        },
        "REAL" | "FLOAT4" => ColumnType::Real,
        "DOUBLE PRECISION" | "FLOAT8" | "FLOAT" | "DOUBLE" => ColumnType::DoublePrecision,
        "CHAR" | "CHARACTER" | "BPCHAR" => {
            ColumnType::Char(char_length.map(|l| l as u32).unwrap_or(1))
        }
        "VARCHAR" | "CHARACTER VARYING" => {
            ColumnType::Varchar(char_length.map(|l| l as u32))
        }
        "TEXT" => ColumnType::Text,
        "BOOLEAN" | "BOOL" => ColumnType::Boolean,
        "DATE" => ColumnType::Date,
        "TIME" => ColumnType::Time {
            with_timezone: false,
        },
        "TIME WITH TIME ZONE" | "TIMETZ" => ColumnType::Time {
            with_timezone: true,
        },
        "TIMESTAMP" | "TIMESTAMP WITHOUT TIME ZONE" => ColumnType::Timestamp {
            with_timezone: false,
        },
        "TIMESTAMP WITH TIME ZONE" | "TIMESTAMPTZ" => ColumnType::Timestamp {
            with_timezone: true,
        },
        "INTERVAL" => ColumnType::Interval,
        "UUID" => ColumnType::Uuid,
        "JSON" => ColumnType::Json,
        "JSONB" => ColumnType::Jsonb,
        "BYTEA" | "BLOB" => ColumnType::Bytea,
        _ if dt.ends_with("[]") => {
            let inner = &dt[..dt.len() - 2];
            ColumnType::Array(Box::new(parse_column_type(
                inner, char_length, precision, scale,
            )))
        }
        _ => ColumnType::Custom(data_type.to_string()),
    }
}

/// Parse default value expression
fn parse_default(default: &str) -> ColumnDefault {
    let trimmed = default.trim();
    let upper = trimmed.to_uppercase();

    if upper == "NULL" {
        ColumnDefault::Null
    } else if upper == "TRUE" || upper == "'T'" || upper == "1" {
        ColumnDefault::Boolean(true)
    } else if upper == "FALSE" || upper == "'F'" || upper == "0" {
        ColumnDefault::Boolean(false)
    } else if upper == "CURRENT_TIMESTAMP" || upper == "NOW()" {
        ColumnDefault::CurrentTimestamp
    } else if upper.contains("GEN_RANDOM_UUID") || upper.contains("UUID_GENERATE") {
        ColumnDefault::GenerateUuid
    } else if let Ok(i) = trimmed.parse::<i64>() {
        ColumnDefault::Integer(i)
    } else if let Ok(f) = trimmed.parse::<f64>() {
        ColumnDefault::Float(f)
    } else if trimmed.starts_with('\'') && trimmed.ends_with('\'') {
        ColumnDefault::String(trimmed[1..trimmed.len() - 1].replace("''", "'"))
    } else {
        ColumnDefault::Expression(default.to_string())
    }
}

/// Parse foreign key action
fn parse_fk_action(action: Option<&str>) -> chakra_core::model::ForeignKeyAction {
    match action {
        Some("CASCADE") => chakra_core::model::ForeignKeyAction::Cascade,
        Some("SET NULL") => chakra_core::model::ForeignKeyAction::SetNull,
        Some("SET DEFAULT") => chakra_core::model::ForeignKeyAction::SetDefault,
        Some("RESTRICT") => chakra_core::model::ForeignKeyAction::Restrict,
        _ => chakra_core::model::ForeignKeyAction::NoAction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_column_type() {
        assert_eq!(parse_column_type("INTEGER", None, None, None), ColumnType::Integer);
        assert_eq!(
            parse_column_type("VARCHAR", Some(100), None, None),
            ColumnType::Varchar(Some(100))
        );
        assert_eq!(
            parse_column_type("DECIMAL", None, Some(10), Some(2)),
            ColumnType::Decimal {
                precision: 10,
                scale: 2
            }
        );
    }

    #[test]
    fn test_parse_default() {
        assert!(matches!(parse_default("NULL"), ColumnDefault::Null));
        assert!(matches!(parse_default("TRUE"), ColumnDefault::Boolean(true)));
        assert!(matches!(parse_default("42"), ColumnDefault::Integer(42)));
        assert!(matches!(
            parse_default("CURRENT_TIMESTAMP"),
            ColumnDefault::CurrentTimestamp
        ));
    }
}
