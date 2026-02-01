//! MySQL configuration

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// MySQL connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySqlConfig {
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
    /// Connection timeout
    pub connect_timeout: Duration,
    /// Pool configuration
    pub pool_min: usize,
    pub pool_max: usize,
}

impl MySqlConfig {
    /// Create a new config with defaults
    pub fn new(host: impl Into<String>, database: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port: 3306,
            database: database.into(),
            user: "root".to_string(),
            password: None,
            connect_timeout: Duration::from_secs(30),
            pool_min: 1,
            pool_max: 10,
        }
    }

    /// Parse from a connection URL
    pub fn from_url(url: &str) -> Result<Self, ConfigError> {
        let url = url.strip_prefix("mysql://")
            .ok_or_else(|| ConfigError::InvalidUrl("URL must start with mysql://".into()))?;

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
            (parts[0].to_string(), parts[1].parse().unwrap_or(3306))
        } else {
            (host_port.to_string(), 3306)
        };

        let (user, password) = if let Some(auth) = auth {
            if auth.contains(':') {
                let parts: Vec<&str> = auth.splitn(2, ':').collect();
                (parts[0].to_string(), Some(parts[1].to_string()))
            } else {
                (auth.to_string(), None)
            }
        } else {
            ("root".to_string(), None)
        };

        let database = database
            .map(|d| d.split('?').next().unwrap_or(d).to_string())
            .unwrap_or_else(|| "mysql".to_string());

        Ok(Self {
            host,
            port,
            database,
            user,
            password,
            connect_timeout: Duration::from_secs(30),
            pool_min: 1,
            pool_max: 10,
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

    /// Set pool size
    pub fn pool_size(mut self, min: usize, max: usize) -> Self {
        self.pool_min = min;
        self.pool_max = max;
        self
    }

    /// Build connection URL for mysql_async
    pub fn connection_url(&self) -> String {
        let auth = if let Some(ref password) = self.password {
            format!("{}:{}@", self.user, password)
        } else {
            format!("{}@", self.user)
        };

        format!(
            "mysql://{}{}:{}/{}",
            auth, self.host, self.port, self.database
        )
    }
}

impl Default for MySqlConfig {
    fn default() -> Self {
        Self::new("localhost", "mysql")
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_url() {
        let config = MySqlConfig::from_url("mysql://user:pass@localhost:3306/mydb").unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 3306);
        assert_eq!(config.database, "mydb");
        assert_eq!(config.user, "user");
        assert_eq!(config.password, Some("pass".to_string()));
    }
}
