//! Error types for Chakra ORM

use std::fmt;
use thiserror::Error;

/// Result type alias using ChakraError
pub type Result<T> = std::result::Result<T, ChakraError>;

/// Main error type for Chakra ORM
#[derive(Error, Debug)]
pub enum ChakraError {
    /// Database connection errors
    #[error("Connection error: {0}")]
    Connection(#[from] ConnectionError),

    /// Query execution errors
    #[error("Query error: {0}")]
    Query(#[from] QueryError),

    /// Transaction errors
    #[error("Transaction error: {message}")]
    Transaction {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Model definition errors
    #[error("Model error: {0}")]
    Model(#[from] ModelError),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Migration errors
    #[error("Migration error: {message}")]
    Migration {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Type conversion errors
    #[error("Type conversion error: {message}")]
    TypeConversion {
        message: String,
        from_type: String,
        to_type: String,
    },

    /// Pool errors
    #[error("Pool error: {message}")]
    Pool {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Configuration errors
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Connection-specific errors
#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Connection failed: {message}")]
    ConnectionFailed { message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Pool closed")]
    PoolClosed,

    #[error("Pool timeout after {timeout:?}")]
    PoolTimeout { timeout: std::time::Duration },

    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("SSL/TLS error: {message}")]
    SslError { message: String },
}

/// Query-specific errors
#[derive(Error, Debug)]
pub enum QueryError {
    #[error("Record not found")]
    NotFound,

    #[error("Multiple records found where one expected")]
    MultipleResults,

    #[error("Unique constraint violated on field: {field}")]
    UniqueViolation { field: String },

    #[error("Foreign key constraint violated: {constraint}")]
    ForeignKeyViolation { constraint: String },

    #[error("Check constraint violated: {constraint}")]
    CheckViolation { constraint: String },

    #[error("Not null constraint violated on field: {field}")]
    NotNullViolation { field: String },

    #[error("SQL syntax error: {message}")]
    SyntaxError {
        message: String,
        position: Option<usize>,
    },

    #[error("Query timeout after {duration_ms}ms")]
    Timeout { duration_ms: u64 },

    #[error("Query cancelled")]
    Cancelled,

    #[error("Invalid query: {message}")]
    Invalid { message: String },

    #[error("Query execution failed: {message}")]
    ExecutionFailed { message: String },
}

/// Model-specific errors
#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Model not registered: {name}")]
    NotRegistered { name: String },

    #[error("Invalid field: {field} on model {model}")]
    InvalidField { model: String, field: String },

    #[error("Missing required field: {field} on model {model}")]
    MissingField { model: String, field: String },

    #[error("Invalid relationship: {relationship} on model {model}")]
    InvalidRelationship { model: String, relationship: String },

    #[error("Relationship not loaded: {relationship}")]
    RelationshipNotLoaded { relationship: String },
}

/// Validation errors
#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Field '{field}' failed validation: {message}")]
    FieldValidation { field: String, message: String },

    #[error("Value out of range for field '{field}': {message}")]
    OutOfRange { field: String, message: String },

    #[error("Invalid format for field '{field}': {message}")]
    InvalidFormat { field: String, message: String },

    #[error("Value too long for field '{field}': max {max_length}, got {actual_length}")]
    TooLong {
        field: String,
        max_length: usize,
        actual_length: usize,
    },

    #[error("Value too short for field '{field}': min {min_length}, got {actual_length}")]
    TooShort {
        field: String,
        min_length: usize,
        actual_length: usize,
    },

    #[error("Pattern mismatch for field '{field}': expected pattern {pattern}")]
    PatternMismatch { field: String, pattern: String },
}

impl ChakraError {
    /// Create a connection error
    pub fn connection(message: impl Into<String>) -> Self {
        Self::Connection(ConnectionError::ConnectionFailed {
            message: message.into(),
        })
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// Check if this is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, ChakraError::Query(QueryError::NotFound))
    }

    /// Check if this is a unique violation
    pub fn is_unique_violation(&self) -> bool {
        matches!(self, ChakraError::Query(QueryError::UniqueViolation { .. }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ChakraError::Query(QueryError::NotFound);
        assert_eq!(err.to_string(), "Query error: Record not found");

        let err = ChakraError::connection("Failed to connect");
        assert_eq!(err.to_string(), "Connection error: Connection failed: Failed to connect");
    }

    #[test]
    fn test_error_predicates() {
        let err = ChakraError::Query(QueryError::NotFound);
        assert!(err.is_not_found());
        assert!(!err.is_unique_violation());

        let err = ChakraError::Query(QueryError::UniqueViolation {
            field: "email".to_string(),
        });
        assert!(!err.is_not_found());
        assert!(err.is_unique_violation());
    }
}
