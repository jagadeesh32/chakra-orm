//! Type conversions between Chakra and SQLite

use chakra_core::types::Value;
use rusqlite::types::{ToSql, Value as SqliteValue, ValueRef};

/// Convert a Chakra Value to a SQLite Value
pub fn to_sqlite_value(value: &Value) -> SqliteValue {
    match value {
        Value::Null => SqliteValue::Null,
        Value::Bool(b) => SqliteValue::Integer(if *b { 1 } else { 0 }),
        Value::Int32(i) => SqliteValue::Integer(*i as i64),
        Value::Int64(i) => SqliteValue::Integer(*i),
        Value::Float64(f) => SqliteValue::Real(*f),
        Value::Decimal(d) => SqliteValue::Text(d.to_string()),
        Value::String(s) => SqliteValue::Text(s.clone()),
        Value::Bytes(b) => SqliteValue::Blob(b.clone()),
        Value::Uuid(u) => SqliteValue::Text(u.to_string()),
        Value::DateTime(dt) => SqliteValue::Text(dt.to_rfc3339()),
        Value::Date(d) => SqliteValue::Text(d.to_string()),
        Value::Time(t) => SqliteValue::Text(t.to_string()),
        Value::Json(j) => SqliteValue::Text(j.to_string()),
        Value::Array(arr) => {
            let json = serde_json::Value::Array(
                arr.iter()
                    .map(|v| match v {
                        Value::String(s) => serde_json::Value::String(s.clone()),
                        Value::Int64(i) => serde_json::json!(i),
                        Value::Bool(b) => serde_json::json!(b),
                        _ => serde_json::Value::Null,
                    })
                    .collect(),
            );
            SqliteValue::Text(json.to_string())
        }
    }
}

/// Convert a SQLite ValueRef to a Chakra Value
pub fn from_sqlite_value(value: ValueRef<'_>) -> Value {
    match value {
        ValueRef::Null => Value::Null,
        ValueRef::Integer(i) => Value::Int64(i),
        ValueRef::Real(f) => Value::Float64(f),
        ValueRef::Text(t) => {
            let s = String::from_utf8_lossy(t).to_string();
            // Try to parse as datetime
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
                return Value::DateTime(dt.with_timezone(&chrono::Utc));
            }
            // Try to parse as UUID
            if let Ok(u) = uuid::Uuid::parse_str(&s) {
                return Value::Uuid(u);
            }
            // Try to parse as JSON
            if s.starts_with('{') || s.starts_with('[') {
                if let Ok(j) = serde_json::from_str(&s) {
                    return Value::Json(j);
                }
            }
            Value::String(s)
        }
        ValueRef::Blob(b) => Value::Bytes(b.to_vec()),
    }
}

/// Convert a row to a Chakra Row
pub fn row_to_chakra(
    row: &rusqlite::Row<'_>,
    column_names: &[String],
) -> Result<chakra_core::result::Row, rusqlite::Error> {
    let values: Result<Vec<Value>, rusqlite::Error> = (0..column_names.len())
        .map(|i| {
            let val = row.get_ref(i)?;
            Ok::<Value, rusqlite::Error>(from_sqlite_value(val))
        })
        .collect();

    Ok(chakra_core::result::Row::new(column_names.to_vec(), values?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_sqlite_value() {
        let val = Value::Int64(42);
        let sqlite_val = to_sqlite_value(&val);
        assert!(matches!(sqlite_val, SqliteValue::Integer(42)));
    }

    #[test]
    fn test_bool_conversion() {
        let val = Value::Bool(true);
        let sqlite_val = to_sqlite_value(&val);
        assert!(matches!(sqlite_val, SqliteValue::Integer(1)));
    }
}
