//! Type system for Chakra ORM
//!
//! This module defines the core types used throughout the ORM:
//! - `Value` - Runtime representation of database values
//! - `FieldType` - Schema-level field type definitions

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Runtime representation of a database value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// Null value
    Null,
    /// Boolean value
    Bool(bool),
    /// 32-bit integer
    Int32(i32),
    /// 64-bit integer
    Int64(i64),
    /// 64-bit floating point
    Float64(f64),
    /// Decimal with arbitrary precision
    Decimal(Decimal),
    /// UTF-8 string
    String(String),
    /// Binary data
    Bytes(Vec<u8>),
    /// UUID
    Uuid(Uuid),
    /// Date and time with timezone
    DateTime(DateTime<Utc>),
    /// Date only
    Date(NaiveDate),
    /// Time only
    Time(NaiveTime),
    /// JSON value
    Json(serde_json::Value),
    /// Array of values
    Array(Vec<Value>),
}

impl Value {
    /// Check if this value is null
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    /// Try to get as bool
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get as i32
    pub fn as_i32(&self) -> Option<i32> {
        match self {
            Value::Int32(i) => Some(*i),
            Value::Int64(i) => i32::try_from(*i).ok(),
            _ => None,
        }
    }

    /// Try to get as i64
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Int32(i) => Some(*i as i64),
            Value::Int64(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get as f64
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Float64(f) => Some(*f),
            Value::Int32(i) => Some(*i as f64),
            Value::Int64(i) => Some(*i as f64),
            _ => None,
        }
    }

    /// Try to get as string reference
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get as bytes reference
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b),
            _ => None,
        }
    }

    /// Get the type name for this value
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int32(_) => "i32",
            Value::Int64(_) => "i64",
            Value::Float64(_) => "f64",
            Value::Decimal(_) => "decimal",
            Value::String(_) => "string",
            Value::Bytes(_) => "bytes",
            Value::Uuid(_) => "uuid",
            Value::DateTime(_) => "datetime",
            Value::Date(_) => "date",
            Value::Time(_) => "time",
            Value::Json(_) => "json",
            Value::Array(_) => "array",
        }
    }
}

// Implement From for common types
impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Value::Int32(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int64(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float64(v)
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::String(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::String(v.to_string())
    }
}

impl From<Uuid> for Value {
    fn from(v: Uuid) -> Self {
        Value::Uuid(v)
    }
}

impl From<DateTime<Utc>> for Value {
    fn from(v: DateTime<Utc>) -> Self {
        Value::DateTime(v)
    }
}

impl From<serde_json::Value> for Value {
    fn from(v: serde_json::Value) -> Self {
        Value::Json(v)
    }
}

impl<T: Into<Value>> From<Option<T>> for Value {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(v) => v.into(),
            None => Value::Null,
        }
    }
}

impl<T: Into<Value>> From<Vec<T>> for Value {
    fn from(v: Vec<T>) -> Self {
        Value::Array(v.into_iter().map(Into::into).collect())
    }
}

/// Schema-level field type definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FieldType {
    /// Boolean
    Boolean,
    /// Small integer (16-bit)
    SmallInt,
    /// Integer (32-bit)
    Integer,
    /// Big integer (64-bit)
    BigInt,
    /// Single precision float
    Float,
    /// Double precision float
    Double,
    /// Fixed precision decimal
    Decimal { precision: u32, scale: u32 },
    /// Variable length string
    String { max_length: Option<usize> },
    /// Fixed length string
    Char { length: usize },
    /// Unlimited text
    Text,
    /// Binary data
    Binary { max_length: Option<usize> },
    /// UUID
    Uuid,
    /// Date only
    Date,
    /// Time only
    Time,
    /// Timestamp without timezone
    Timestamp,
    /// Timestamp with timezone
    TimestampTz,
    /// JSON
    Json,
    /// JSONB (PostgreSQL)
    JsonB,
    /// Array of another type
    Array { element_type: Box<FieldType> },
    /// Enum with possible values
    Enum { values: Vec<String> },
}

impl FieldType {
    /// Create a string field with max length
    pub fn string(max_length: usize) -> Self {
        FieldType::String {
            max_length: Some(max_length),
        }
    }

    /// Create an unlimited text field
    pub fn text() -> Self {
        FieldType::Text
    }

    /// Create a decimal field
    pub fn decimal(precision: u32, scale: u32) -> Self {
        FieldType::Decimal { precision, scale }
    }

    /// Create an array field
    pub fn array(element_type: FieldType) -> Self {
        FieldType::Array {
            element_type: Box::new(element_type),
        }
    }

    /// Get the SQL type name for PostgreSQL
    pub fn to_postgres_type(&self) -> String {
        match self {
            FieldType::Boolean => "BOOLEAN".to_string(),
            FieldType::SmallInt => "SMALLINT".to_string(),
            FieldType::Integer => "INTEGER".to_string(),
            FieldType::BigInt => "BIGINT".to_string(),
            FieldType::Float => "REAL".to_string(),
            FieldType::Double => "DOUBLE PRECISION".to_string(),
            FieldType::Decimal { precision, scale } => {
                format!("NUMERIC({}, {})", precision, scale)
            }
            FieldType::String { max_length: Some(n) } => format!("VARCHAR({})", n),
            FieldType::String { max_length: None } => "VARCHAR".to_string(),
            FieldType::Char { length } => format!("CHAR({})", length),
            FieldType::Text => "TEXT".to_string(),
            FieldType::Binary { .. } => "BYTEA".to_string(),
            FieldType::Uuid => "UUID".to_string(),
            FieldType::Date => "DATE".to_string(),
            FieldType::Time => "TIME".to_string(),
            FieldType::Timestamp => "TIMESTAMP".to_string(),
            FieldType::TimestampTz => "TIMESTAMPTZ".to_string(),
            FieldType::Json => "JSON".to_string(),
            FieldType::JsonB => "JSONB".to_string(),
            FieldType::Array { element_type } => {
                format!("{}[]", element_type.to_postgres_type())
            }
            FieldType::Enum { .. } => "VARCHAR(255)".to_string(), // Simplified for now
        }
    }

    /// Get the SQL type name for MySQL
    pub fn to_mysql_type(&self) -> String {
        match self {
            FieldType::Boolean => "TINYINT(1)".to_string(),
            FieldType::SmallInt => "SMALLINT".to_string(),
            FieldType::Integer => "INT".to_string(),
            FieldType::BigInt => "BIGINT".to_string(),
            FieldType::Float => "FLOAT".to_string(),
            FieldType::Double => "DOUBLE".to_string(),
            FieldType::Decimal { precision, scale } => {
                format!("DECIMAL({}, {})", precision, scale)
            }
            FieldType::String { max_length: Some(n) } => format!("VARCHAR({})", n),
            FieldType::String { max_length: None } => "VARCHAR(255)".to_string(),
            FieldType::Char { length } => format!("CHAR({})", length),
            FieldType::Text => "TEXT".to_string(),
            FieldType::Binary { max_length: Some(n) } => format!("VARBINARY({})", n),
            FieldType::Binary { max_length: None } => "BLOB".to_string(),
            FieldType::Uuid => "CHAR(36)".to_string(),
            FieldType::Date => "DATE".to_string(),
            FieldType::Time => "TIME".to_string(),
            FieldType::Timestamp | FieldType::TimestampTz => "DATETIME".to_string(),
            FieldType::Json | FieldType::JsonB => "JSON".to_string(),
            FieldType::Array { .. } => "JSON".to_string(), // MySQL doesn't have native arrays
            FieldType::Enum { values } => {
                format!("ENUM({})", values.iter().map(|v| format!("'{}'", v)).collect::<Vec<_>>().join(", "))
            }
        }
    }

    /// Get the SQL type name for SQLite
    pub fn to_sqlite_type(&self) -> String {
        match self {
            FieldType::Boolean => "INTEGER".to_string(),
            FieldType::SmallInt | FieldType::Integer | FieldType::BigInt => "INTEGER".to_string(),
            FieldType::Float | FieldType::Double | FieldType::Decimal { .. } => "REAL".to_string(),
            FieldType::String { .. } | FieldType::Char { .. } | FieldType::Text => "TEXT".to_string(),
            FieldType::Binary { .. } => "BLOB".to_string(),
            FieldType::Uuid => "TEXT".to_string(),
            FieldType::Date | FieldType::Time | FieldType::Timestamp | FieldType::TimestampTz => {
                "TEXT".to_string()
            }
            FieldType::Json | FieldType::JsonB => "TEXT".to_string(),
            FieldType::Array { .. } => "TEXT".to_string(), // Store as JSON
            FieldType::Enum { .. } => "TEXT".to_string(),
        }
    }
}

/// Type registry for custom types
#[derive(Debug, Default)]
pub struct TypeRegistry {
    custom_types: HashMap<String, FieldType>,
}

impl TypeRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a custom type
    pub fn register(&mut self, name: impl Into<String>, field_type: FieldType) {
        self.custom_types.insert(name.into(), field_type);
    }

    /// Get a registered type
    pub fn get(&self, name: &str) -> Option<&FieldType> {
        self.custom_types.get(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_conversions() {
        let v: Value = 42i32.into();
        assert_eq!(v.as_i32(), Some(42));
        assert_eq!(v.as_i64(), Some(42));

        let v: Value = "hello".into();
        assert_eq!(v.as_str(), Some("hello"));

        let v: Value = true.into();
        assert_eq!(v.as_bool(), Some(true));
    }

    #[test]
    fn test_field_type_postgres() {
        assert_eq!(FieldType::Integer.to_postgres_type(), "INTEGER");
        assert_eq!(FieldType::string(100).to_postgres_type(), "VARCHAR(100)");
        assert_eq!(
            FieldType::decimal(10, 2).to_postgres_type(),
            "NUMERIC(10, 2)"
        );
    }

    #[test]
    fn test_optional_value() {
        let v: Value = Some(42i32).into();
        assert_eq!(v.as_i32(), Some(42));

        let v: Value = Option::<i32>::None.into();
        assert!(v.is_null());
    }
}
