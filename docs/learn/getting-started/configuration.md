---
title: Configuration
description: Configure Chakra ORM for your application
---

# Configuration

Chakra ORM can be configured via TOML file, environment variables, or programmatically.

## Configuration File

Create `chakra.toml` in your project root:

```toml
# chakra.toml

[database]
url = "postgresql://user:password@localhost:5432/mydb"
ssl_mode = "prefer"  # disable, prefer, require
connect_timeout = "10s"
statement_timeout = "30s"

[pool]
min_connections = 5
max_connections = 20
acquire_timeout = "30s"
idle_timeout = "10m"
max_lifetime = "30m"

[migrations]
directory = "migrations"
table_name = "_chakra_migrations"
lock_timeout = "30s"

[logging]
level = "info"        # trace, debug, info, warn, error
queries = false       # Log all queries
slow_query_ms = 1000  # Log queries slower than this
```

## Environment Variables

Environment variables override file settings:

```bash
# Database
export DATABASE_URL="postgresql://user:pass@localhost:5432/db"
export CHAKRA_DATABASE_SSL_MODE="require"

# Pool
export CHAKRA_POOL_MAX_CONNECTIONS="50"
export CHAKRA_POOL_ACQUIRE_TIMEOUT="60s"

# Logging
export CHAKRA_LOG="debug"
export CHAKRA_LOG_QUERIES="1"
```

## Programmatic Configuration

=== "Python"

    ```python
    from chakra import configure, Config, PoolConfig, LoggingConfig

    configure(Config(
        database_url="postgresql://localhost/db",
        pool=PoolConfig(
            min_connections=5,
            max_connections=20,
            acquire_timeout="30s",
        ),
        logging=LoggingConfig(
            level="debug",
            queries=True,
        ),
    ))
    ```

=== "Rust"

    ```rust
    use chakra::{Config, PoolConfig};

    let config = Config::builder()
        .database_url("postgresql://localhost/db")
        .pool(PoolConfig::builder()
            .min_connections(5)
            .max_connections(20)
            .acquire_timeout(Duration::from_secs(30))
            .build())
        .build();

    let pool = chakra::connect_with_config(config).await?;
    ```

## Database Configuration

### Connection URL Format

```
driver://user:password@host:port/database?options
```

Examples:

```toml
# PostgreSQL
url = "postgresql://user:pass@localhost:5432/mydb"
url = "postgresql://user:pass@localhost:5432/mydb?sslmode=require"

# MySQL
url = "mysql://user:pass@localhost:3306/mydb"
url = "mysql://user:pass@localhost:3306/mydb?charset=utf8mb4"

# SQLite
url = "sqlite:///path/to/database.db"
url = "sqlite:///:memory:"  # In-memory

# Oracle
url = "oracle://user:pass@localhost:1521/service"
```

### SSL Configuration

```toml
[database]
url = "postgresql://localhost/db"
ssl_mode = "require"      # disable, prefer, require, verify-ca, verify-full
ssl_cert = "/path/to/client-cert.pem"
ssl_key = "/path/to/client-key.pem"
ssl_root_cert = "/path/to/ca-cert.pem"
```

## Pool Configuration

| Option | Default | Description |
|--------|---------|-------------|
| `min_connections` | 1 | Minimum idle connections |
| `max_connections` | 10 | Maximum total connections |
| `acquire_timeout` | 30s | Max wait for connection |
| `idle_timeout` | 10m | Close idle connections after |
| `max_lifetime` | 30m | Recycle connections after |
| `health_check_interval` | 30s | Connection health check period |

### Sizing Guidelines

```toml
[pool]
# For web applications: 2 * CPU cores
max_connections = 16

# For background workers: smaller pool
max_connections = 5

# For high-concurrency: larger pool, but consider DB limits
max_connections = 50
```

## Logging Configuration

### Log Levels

| Level | Description |
|-------|-------------|
| `trace` | Very verbose, including internal operations |
| `debug` | Detailed debugging information |
| `info` | General operational messages |
| `warn` | Warning conditions |
| `error` | Error conditions only |

### Query Logging

```toml
[logging]
level = "info"
queries = true          # Log all queries
query_params = false    # Log query parameters (careful with sensitive data!)
slow_query_ms = 100     # Log queries slower than 100ms
```

Output:

```
[2024-06-15 10:30:00] DEBUG chakra::query
  SELECT users.id, users.username FROM users WHERE users.is_active = $1
  params: [true]
  duration: 2.3ms
  rows: 42
```

## Migration Configuration

```toml
[migrations]
directory = "migrations"          # Migration files location
table_name = "_chakra_migrations" # History table name
lock_timeout = "30s"              # Migration lock timeout
```

## Multiple Environments

### Using Environment Variables

```python
# config.py
import os
from chakra import Config

def get_config():
    env = os.getenv("ENVIRONMENT", "development")

    if env == "production":
        return Config(
            database_url=os.environ["DATABASE_URL"],
            pool_max_connections=50,
            logging_level="warn",
        )
    else:
        return Config(
            database_url="postgresql://localhost/dev_db",
            pool_max_connections=5,
            logging_level="debug",
            logging_queries=True,
        )
```

### Multiple Config Files

```bash
# Development
chakra --config chakra.dev.toml migrate apply

# Production
chakra --config chakra.prod.toml migrate apply
```

## Configuration Precedence

1. Programmatic configuration (highest priority)
2. Environment variables
3. `chakra.toml` file
4. Default values (lowest priority)
