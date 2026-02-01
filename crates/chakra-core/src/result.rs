//! Result handling for Chakra ORM
//!
//! This module provides:
//! - `Row` - A database row
//! - `FromRow` - Trait for deserializing rows
//! - `RowStream` - Async stream of rows

use crate::error::{ChakraError, Result};
use crate::types::Value;
use std::collections::HashMap;

/// A database row
#[derive(Debug, Clone)]
pub struct Row {
    columns: Vec<String>,
    values: HashMap<String, Value>,
}

impl Row {
    /// Create a new row from columns and values
    pub fn new(columns: Vec<String>, values: Vec<Value>) -> Self {
        let values_map: HashMap<String, Value> = columns
            .iter()
            .cloned()
            .zip(values.into_iter())
            .collect();
        Self {
            columns,
            values: values_map,
        }
    }

    /// Create from a HashMap
    pub fn from_map(values: HashMap<String, Value>) -> Self {
        let columns: Vec<String> = values.keys().cloned().collect();
        Self { columns, values }
    }

    /// Get a value by column name
    pub fn get(&self, column: &str) -> Option<&Value> {
        self.values.get(column)
    }

    /// Get a value by column index
    pub fn get_by_index(&self, index: usize) -> Option<&Value> {
        self.columns.get(index).and_then(|col| self.values.get(col))
    }

    /// Get value as a specific type
    pub fn get_as<T: FromValue>(&self, column: &str) -> Result<T> {
        let value = self.get(column).ok_or_else(|| {
            ChakraError::internal(format!("Column not found: {}", column))
        })?;
        T::from_value(value)
    }

    /// Try to get value, returning None if column doesn't exist
    pub fn try_get<T: FromValue>(&self, column: &str) -> Result<Option<T>> {
        match self.get(column) {
            Some(Value::Null) => Ok(None),
            Some(value) => Ok(Some(T::from_value(value)?)),
            None => Ok(None),
        }
    }

    /// Get column names
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// Get all values
    pub fn values(&self) -> &HashMap<String, Value> {
        &self.values
    }

    /// Check if column exists
    pub fn has_column(&self, column: &str) -> bool {
        self.values.contains_key(column)
    }

    /// Number of columns
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }
}

/// Trait for converting from Value
pub trait FromValue: Sized {
    fn from_value(value: &Value) -> Result<Self>;
}

impl FromValue for bool {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Bool(b) => Ok(*b),
            Value::Int32(i) => Ok(*i != 0),
            Value::Int64(i) => Ok(*i != 0),
            _ => Err(ChakraError::TypeConversion {
                message: "Cannot convert to bool".to_string(),
                from_type: value.type_name().to_string(),
                to_type: "bool".to_string(),
            }),
        }
    }
}

impl FromValue for i32 {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Int32(i) => Ok(*i),
            Value::Int64(i) => i32::try_from(*i).map_err(|_| ChakraError::TypeConversion {
                message: "Integer overflow".to_string(),
                from_type: "i64".to_string(),
                to_type: "i32".to_string(),
            }),
            _ => Err(ChakraError::TypeConversion {
                message: "Cannot convert to i32".to_string(),
                from_type: value.type_name().to_string(),
                to_type: "i32".to_string(),
            }),
        }
    }
}

impl FromValue for i64 {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Int32(i) => Ok(*i as i64),
            Value::Int64(i) => Ok(*i),
            _ => Err(ChakraError::TypeConversion {
                message: "Cannot convert to i64".to_string(),
                from_type: value.type_name().to_string(),
                to_type: "i64".to_string(),
            }),
        }
    }
}

impl FromValue for f64 {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Float64(f) => Ok(*f),
            Value::Int32(i) => Ok(*i as f64),
            Value::Int64(i) => Ok(*i as f64),
            _ => Err(ChakraError::TypeConversion {
                message: "Cannot convert to f64".to_string(),
                from_type: value.type_name().to_string(),
                to_type: "f64".to_string(),
            }),
        }
    }
}

impl FromValue for String {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::String(s) => Ok(s.clone()),
            _ => Err(ChakraError::TypeConversion {
                message: "Cannot convert to String".to_string(),
                from_type: value.type_name().to_string(),
                to_type: "String".to_string(),
            }),
        }
    }
}

impl FromValue for chrono::DateTime<chrono::Utc> {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::DateTime(dt) => Ok(*dt),
            _ => Err(ChakraError::TypeConversion {
                message: "Cannot convert to DateTime".to_string(),
                from_type: value.type_name().to_string(),
                to_type: "DateTime".to_string(),
            }),
        }
    }
}

impl FromValue for uuid::Uuid {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Uuid(u) => Ok(*u),
            Value::String(s) => uuid::Uuid::parse_str(s).map_err(|_| ChakraError::TypeConversion {
                message: "Invalid UUID string".to_string(),
                from_type: "String".to_string(),
                to_type: "Uuid".to_string(),
            }),
            _ => Err(ChakraError::TypeConversion {
                message: "Cannot convert to Uuid".to_string(),
                from_type: value.type_name().to_string(),
                to_type: "Uuid".to_string(),
            }),
        }
    }
}

impl FromValue for serde_json::Value {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Json(j) => Ok(j.clone()),
            Value::String(s) => serde_json::from_str(s).map_err(|e| ChakraError::TypeConversion {
                message: format!("Invalid JSON: {}", e),
                from_type: "String".to_string(),
                to_type: "Json".to_string(),
            }),
            _ => Err(ChakraError::TypeConversion {
                message: "Cannot convert to Json".to_string(),
                from_type: value.type_name().to_string(),
                to_type: "Json".to_string(),
            }),
        }
    }
}

impl<T: FromValue> FromValue for Option<T> {
    fn from_value(value: &Value) -> Result<Self> {
        match value {
            Value::Null => Ok(None),
            other => Ok(Some(T::from_value(other)?)),
        }
    }
}

/// Trait for types that can be constructed from a database row
pub trait FromRow: Sized {
    fn from_row(row: &Row) -> Result<Self>;
}

/// Async stream of rows
pub struct RowStream<T> {
    _marker: std::marker::PhantomData<T>,
    // In a real implementation, this would hold the async stream
    // For now, we use a simple vector
    rows: Vec<Row>,
    index: usize,
}

impl<T: FromRow> RowStream<T> {
    /// Create a new stream from rows
    pub fn new(rows: Vec<Row>) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            rows,
            index: 0,
        }
    }

    /// Collect all rows
    pub async fn collect(self) -> Result<Vec<T>> {
        self.rows.iter().map(T::from_row).collect()
    }
}

impl<T: FromRow> Iterator for RowStream<T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.rows.len() {
            let row = &self.rows[self.index];
            self.index += 1;
            Some(T::from_row(row))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_row_get() {
        let row = Row::new(
            vec!["id".to_string(), "name".to_string()],
            vec![Value::Int64(1), Value::String("Alice".to_string())],
        );

        assert_eq!(row.get("id"), Some(&Value::Int64(1)));
        assert_eq!(row.get("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(row.get("nonexistent"), None);
    }

    #[test]
    fn test_row_get_as() {
        let row = Row::new(
            vec!["id".to_string(), "name".to_string()],
            vec![Value::Int64(42), Value::String("Bob".to_string())],
        );

        let id: i64 = row.get_as("id").unwrap();
        assert_eq!(id, 42);

        let name: String = row.get_as("name").unwrap();
        assert_eq!(name, "Bob");
    }

    #[test]
    fn test_from_value_option() {
        let null = Value::Null;
        let some = Value::Int64(42);

        let opt_none: Option<i64> = Option::from_value(&null).unwrap();
        assert_eq!(opt_none, None);

        let opt_some: Option<i64> = Option::from_value(&some).unwrap();
        assert_eq!(opt_some, Some(42));
    }
}
