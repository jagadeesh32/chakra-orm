//! Type conversions between Chakra and PostgreSQL

use chakra_core::types::Value;
use tokio_postgres::types::{FromSql, ToSql, Type};

/// Convert a Chakra Value to a PostgreSQL parameter
pub fn to_postgres_param(value: &Value) -> Box<dyn ToSql + Sync + Send> {
    match value {
        Value::Null => Box::new(Option::<i32>::None),
        Value::Bool(b) => Box::new(*b),
        Value::Int32(i) => Box::new(*i),
        Value::Int64(i) => Box::new(*i),
        Value::Float64(f) => Box::new(*f),
        Value::Decimal(d) => Box::new(d.to_string()),
        Value::String(s) => Box::new(s.clone()),
        Value::Bytes(b) => Box::new(b.clone()),
        Value::Uuid(u) => Box::new(*u),
        Value::DateTime(dt) => Box::new(*dt),
        Value::Date(d) => Box::new(*d),
        Value::Time(t) => Box::new(*t),
        Value::Json(j) => Box::new(j.clone()),
        Value::Array(arr) => {
            // Convert array to JSON for simplicity
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
            Box::new(json)
        }
    }
}

/// Convert a PostgreSQL row value to a Chakra Value
pub fn from_postgres_value(
    row: &tokio_postgres::Row,
    idx: usize,
    col_type: &Type,
) -> Value {
    match *col_type {
        Type::BOOL => row.get::<_, Option<bool>>(idx).map(Value::Bool).unwrap_or(Value::Null),
        Type::INT2 => row.get::<_, Option<i16>>(idx).map(|i| Value::Int32(i as i32)).unwrap_or(Value::Null),
        Type::INT4 => row.get::<_, Option<i32>>(idx).map(Value::Int32).unwrap_or(Value::Null),
        Type::INT8 => row.get::<_, Option<i64>>(idx).map(Value::Int64).unwrap_or(Value::Null),
        Type::FLOAT4 => row.get::<_, Option<f32>>(idx).map(|f| Value::Float64(f as f64)).unwrap_or(Value::Null),
        Type::FLOAT8 => row.get::<_, Option<f64>>(idx).map(Value::Float64).unwrap_or(Value::Null),
        Type::TEXT | Type::VARCHAR | Type::BPCHAR | Type::NAME => {
            row.get::<_, Option<String>>(idx).map(Value::String).unwrap_or(Value::Null)
        }
        Type::BYTEA => row.get::<_, Option<Vec<u8>>>(idx).map(Value::Bytes).unwrap_or(Value::Null),
        Type::UUID => row.get::<_, Option<uuid::Uuid>>(idx).map(Value::Uuid).unwrap_or(Value::Null),
        Type::TIMESTAMPTZ => {
            row.get::<_, Option<chrono::DateTime<chrono::Utc>>>(idx)
                .map(Value::DateTime)
                .unwrap_or(Value::Null)
        }
        Type::TIMESTAMP => {
            row.get::<_, Option<chrono::NaiveDateTime>>(idx)
                .map(|dt| Value::DateTime(chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc)))
                .unwrap_or(Value::Null)
        }
        Type::DATE => row.get::<_, Option<chrono::NaiveDate>>(idx).map(Value::Date).unwrap_or(Value::Null),
        Type::TIME => row.get::<_, Option<chrono::NaiveTime>>(idx).map(Value::Time).unwrap_or(Value::Null),
        Type::JSON | Type::JSONB => {
            row.get::<_, Option<serde_json::Value>>(idx).map(Value::Json).unwrap_or(Value::Null)
        }
        _ => {
            // Try to get as string
            row.get::<_, Option<String>>(idx).map(Value::String).unwrap_or(Value::Null)
        }
    }
}

/// Convert a Chakra Row from a PostgreSQL Row
pub fn row_from_postgres(pg_row: &tokio_postgres::Row) -> chakra_core::result::Row {
    let columns: Vec<String> = pg_row
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect();

    let values: Vec<Value> = pg_row
        .columns()
        .iter()
        .enumerate()
        .map(|(idx, col)| from_postgres_value(pg_row, idx, col.type_()))
        .collect();

    chakra_core::result::Row::new(columns, values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_postgres_param() {
        let val = Value::Int64(42);
        let _param = to_postgres_param(&val);
        // Just verify it doesn't panic
    }
}
