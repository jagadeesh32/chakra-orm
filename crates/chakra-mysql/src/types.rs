//! Type conversions between Chakra and MySQL

use chakra_core::types::Value;
use mysql_async::Value as MySqlValue;

/// Convert a Chakra Value to a MySQL Value
pub fn to_mysql_value(value: &Value) -> MySqlValue {
    match value {
        Value::Null => MySqlValue::NULL,
        Value::Bool(b) => MySqlValue::from(*b),
        Value::Int32(i) => MySqlValue::from(*i),
        Value::Int64(i) => MySqlValue::from(*i),
        Value::Float64(f) => MySqlValue::from(*f),
        Value::Decimal(d) => MySqlValue::from(d.to_string()),
        Value::String(s) => MySqlValue::from(s.clone()),
        Value::Bytes(b) => MySqlValue::from(b.clone()),
        Value::Uuid(u) => MySqlValue::from(u.to_string()),
        Value::DateTime(dt) => MySqlValue::from(dt.format("%Y-%m-%d %H:%M:%S%.6f").to_string()),
        Value::Date(d) => MySqlValue::from(d.format("%Y-%m-%d").to_string()),
        Value::Time(t) => MySqlValue::from(t.format("%H:%M:%S%.6f").to_string()),
        Value::Json(j) => MySqlValue::from(j.to_string()),
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
            MySqlValue::from(json.to_string())
        }
    }
}

/// Convert a MySQL Value to a Chakra Value
pub fn from_mysql_value(value: MySqlValue) -> Value {
    match value {
        MySqlValue::NULL => Value::Null,
        MySqlValue::Int(i) => Value::Int64(i),
        MySqlValue::UInt(u) => Value::Int64(u as i64),
        MySqlValue::Float(f) => Value::Float64(f as f64),
        MySqlValue::Double(d) => Value::Float64(d),
        MySqlValue::Bytes(b) => {
            // Try to convert to string first
            match String::from_utf8(b.clone()) {
                Ok(s) => Value::String(s),
                Err(_) => Value::Bytes(b),
            }
        }
        _ => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_mysql_value() {
        let val = Value::Int64(42);
        let mysql_val = to_mysql_value(&val);
        // Just verify it doesn't panic
        assert!(!matches!(mysql_val, MySqlValue::NULL));
    }
}
