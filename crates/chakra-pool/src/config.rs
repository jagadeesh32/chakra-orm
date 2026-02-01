//! Pool configuration for Chakra ORM
//!
//! This module provides pool configuration options.

use std::time::Duration;

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of connections in the pool
    pub min_connections: u32,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Maximum time to wait for a connection
    pub acquire_timeout: Duration,
    /// Maximum time a connection can be idle before being closed
    pub idle_timeout: Option<Duration>,
    /// Maximum lifetime of a connection
    pub max_lifetime: Option<Duration>,
    /// How often to run the health check
    pub health_check_interval: Duration,
    /// Whether to test connections on checkout
    pub test_on_checkout: bool,
    /// Whether to test connections on checkin
    pub test_on_checkin: bool,
    /// Connection string/URL
    pub connection_string: String,
    /// Application name for connection identification
    pub application_name: Option<String>,
}

impl PoolConfig {
    /// Create a new pool config with the given connection string
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            min_connections: 1,
            max_connections: 10,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(600)),
            max_lifetime: Some(Duration::from_secs(1800)),
            health_check_interval: Duration::from_secs(30),
            test_on_checkout: true,
            test_on_checkin: false,
            connection_string: connection_string.into(),
            application_name: None,
        }
    }

    /// Set minimum connections
    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = min;
        self
    }

    /// Set maximum connections
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Set acquire timeout
    pub fn acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }

    /// Set idle timeout
    pub fn idle_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.idle_timeout = timeout;
        self
    }

    /// Set max lifetime
    pub fn max_lifetime(mut self, lifetime: Option<Duration>) -> Self {
        self.max_lifetime = lifetime;
        self
    }

    /// Set health check interval
    pub fn health_check_interval(mut self, interval: Duration) -> Self {
        self.health_check_interval = interval;
        self
    }

    /// Set test on checkout
    pub fn test_on_checkout(mut self, test: bool) -> Self {
        self.test_on_checkout = test;
        self
    }

    /// Set test on checkin
    pub fn test_on_checkin(mut self, test: bool) -> Self {
        self.test_on_checkin = test;
        self
    }

    /// Set application name
    pub fn application_name(mut self, name: impl Into<String>) -> Self {
        self.application_name = Some(name.into());
        self
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.min_connections > self.max_connections {
            return Err(ConfigError::InvalidRange {
                field: "connections",
                min: self.min_connections,
                max: self.max_connections,
            });
        }

        if self.max_connections == 0 {
            return Err(ConfigError::InvalidValue {
                field: "max_connections",
                message: "must be greater than 0".to_string(),
            });
        }

        if self.connection_string.is_empty() {
            return Err(ConfigError::MissingField {
                field: "connection_string",
            });
        }

        Ok(())
    }
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self::new("")
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid range for {field}: min ({min}) > max ({max})")]
    InvalidRange { field: &'static str, min: u32, max: u32 },

    #[error("Invalid value for {field}: {message}")]
    InvalidValue { field: &'static str, message: String },

    #[error("Missing required field: {field}")]
    MissingField { field: &'static str },
}

/// Builder for pool configuration from environment or file
#[derive(Debug, Default)]
pub struct PoolConfigBuilder {
    config: PoolConfig,
}

impl PoolConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set connection string
    pub fn connection_string(mut self, url: impl Into<String>) -> Self {
        self.config.connection_string = url.into();
        self
    }

    /// Set from DATABASE_URL environment variable
    pub fn from_env(mut self) -> Self {
        if let Ok(url) = std::env::var("DATABASE_URL") {
            self.config.connection_string = url;
        }
        self
    }

    /// Set pool size (both min and max)
    pub fn pool_size(mut self, size: u32) -> Self {
        self.config.min_connections = size;
        self.config.max_connections = size;
        self
    }

    /// Build the config
    pub fn build(self) -> Result<PoolConfig, ConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = PoolConfig::new("postgres://localhost/test")
            .min_connections(5)
            .max_connections(10);

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_range() {
        let config = PoolConfig::new("postgres://localhost/test")
            .min_connections(10)
            .max_connections(5);

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_connection_string() {
        let config = PoolConfig::default();
        assert!(config.validate().is_err());
    }
}
