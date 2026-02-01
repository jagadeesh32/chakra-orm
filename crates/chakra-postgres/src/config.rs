//! PostgreSQL configuration

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// PostgreSQL connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    /// Host name
    pub host: String,
    /// Port number
    pub port: u16,
    /// Database name
    pub database: String,
    /// Username
    pub user: String,
    /// Password
    pub password: Option<String>,
    /// Schema name
    pub schema: Option<String>,
    /// SSL mode
    pub ssl_mode: SslMode,
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Application name
    pub application_name: Option<String>,
    /// Pool configuration
    pub pool: PoolConfig,
}

impl PostgresConfig {
    /// Create a new config with defaults
    pub fn new(host: impl Into<String>, database: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: 5432,
            database: database.into(),
            user: "postgres".to_string(),
            password: None,
            schema: None,
            ssl_mode: SslMode::Prefer,
            connect_timeout: Duration::from_secs(30),
            application_name: Some("chakra-orm".to_string()),
            pool: PoolConfig::default(),
        }
    }

    /// Parse from a connection URL
    pub fn from_url(url: &str) -> Result<Self, ConfigError> {
        // Parse URL like: postgres://user:pass@host:port/database
        let url = url.strip_prefix("postgres://").or_else(|| url.strip_prefix("postgresql://"))
            .ok_or_else(|| ConfigError::InvalidUrl("URL must start with postgres:// or postgresql://".into()))?;

        let (auth, rest) = if url.contains('@') {
            let parts: Vec<&str> = url.splitn(2, '@').collect();
            (Some(parts[0]), parts[1])
        } else {
            (None, url)
        };

        let (host_port, database) = if rest.contains('/') {
            let parts: Vec<&str> = rest.splitn(2, '/').collect();
            (parts[0], Some(parts[1]))
        } else {
            (rest, None)
        };

        let (host, port) = if host_port.contains(':') {
            let parts: Vec<&str> = host_port.splitn(2, ':').collect();
            (parts[0].to_string(), parts[1].parse().unwrap_or(5432))
        } else {
            (host_port.to_string(), 5432)
        };

        let (user, password) = if let Some(auth) = auth {
            if auth.contains(':') {
                let parts: Vec<&str> = auth.splitn(2, ':').collect();
                (parts[0].to_string(), Some(parts[1].to_string()))
            } else {
                (auth.to_string(), None)
            }
        } else {
            ("postgres".to_string(), None)
        };

        let database = database
            .map(|d| d.split('?').next().unwrap_or(d).to_string())
            .unwrap_or_else(|| "postgres".to_string());

        Ok(Self {
            host,
            port,
            database,
            user,
            password,
            schema: None,
            ssl_mode: SslMode::Prefer,
            connect_timeout: Duration::from_secs(30),
            application_name: Some("chakra-orm".to_string()),
            pool: PoolConfig::default(),
        })
    }

    /// Set port
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set user
    pub fn user(mut self, user: impl Into<String>) -> Self {
        self.user = user.into();
        self
    }

    /// Set password
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Set schema
    pub fn schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    /// Set SSL mode
    pub fn ssl_mode(mut self, mode: SslMode) -> Self {
        self.ssl_mode = mode;
        self
    }

    /// Set pool size
    pub fn pool_size(mut self, size: usize) -> Self {
        self.pool.max_size = size;
        self
    }

    /// Build connection string
    pub fn connection_string(&self) -> String {
        let mut s = format!(
            "host={} port={} dbname={} user={}",
            self.host, self.port, self.database, self.user
        );

        if let Some(ref password) = self.password {
            s.push_str(&format!(" password={}", password));
        }

        if let Some(ref app_name) = self.application_name {
            s.push_str(&format!(" application_name={}", app_name));
        }

        s.push_str(&format!(" connect_timeout={}", self.connect_timeout.as_secs()));

        s
    }
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self::new("localhost", "postgres")
    }
}

/// SSL mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SslMode {
    /// Disable SSL
    Disable,
    /// Allow SSL but don't require it
    Allow,
    /// Prefer SSL but don't require it
    Prefer,
    /// Require SSL
    Require,
    /// Require SSL with CA verification
    VerifyCa,
    /// Require SSL with full verification
    VerifyFull,
}

impl Default for SslMode {
    fn default() -> Self {
        SslMode::Prefer
    }
}

/// Pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Maximum pool size
    pub max_size: usize,
    /// Minimum pool size
    pub min_size: usize,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Idle timeout
    pub idle_timeout: Option<Duration>,
    /// Max lifetime
    pub max_lifetime: Option<Duration>,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_size: 10,
            min_size: 1,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Some(Duration::from_secs(600)),
            max_lifetime: Some(Duration::from_secs(1800)),
        }
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("Missing required field: {0}")]
    MissingField(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_url() {
        let config = PostgresConfig::from_url("postgres://user:pass@localhost:5432/mydb").unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.database, "mydb");
        assert_eq!(config.user, "user");
        assert_eq!(config.password, Some("pass".to_string()));
    }

    #[test]
    fn test_connection_string() {
        let config = PostgresConfig::new("localhost", "mydb")
            .user("testuser")
            .password("secret");

        let conn_str = config.connection_string();
        assert!(conn_str.contains("host=localhost"));
        assert!(conn_str.contains("dbname=mydb"));
        assert!(conn_str.contains("user=testuser"));
        assert!(conn_str.contains("password=secret"));
    }
}
