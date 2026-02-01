---
title: Connection Pooling
description: Efficient database connection management
tags:
  - pool
  - connections
  - performance
---

# Connection Pooling

Chakra ORM includes a high-performance async connection pool built on deadpool, designed for high-concurrency applications.

## Why Connection Pooling?

Database connections are expensive:

- **TCP handshake** — Network round trips
- **TLS negotiation** — Encryption setup
- **Authentication** — Credential verification
- **Session setup** — Server-side resources

A connection pool maintains ready-to-use connections, eliminating this overhead.

## Configuration

```toml
# chakra.toml
[pool]
min_connections = 5       # Minimum connections to maintain
max_connections = 20      # Maximum connections allowed
acquire_timeout = "30s"   # Timeout waiting for a connection
idle_timeout = "10m"      # Close connections idle longer than this
max_lifetime = "30m"      # Recycle connections after this time
```

### Configuration Options

| Option | Default | Description |
|--------|---------|-------------|
| `min_connections` | 1 | Minimum pool size |
| `max_connections` | 10 | Maximum pool size |
| `acquire_timeout` | 30s | Max wait for available connection |
| `idle_timeout` | 10m | Close idle connections after |
| `max_lifetime` | 30m | Force reconnect after |
| `health_check_interval` | 30s | Check connection health |

## Basic Usage

### Python

```python
from chakra import Session, connect

# Using Session (recommended)
async with Session() as session:
    # Connection acquired from pool
    user = await session.get(User, 1)
# Connection returned to pool

# Direct pool access
pool = await connect("postgresql://localhost/db")

async with pool.acquire() as conn:
    result = await conn.execute("SELECT 1")
```

### Rust

```rust
use chakra::Pool;

// Create pool
let pool = chakra::connect("postgresql://localhost/db").await?;

// Acquire connection
let conn = pool.acquire().await?;

// Use connection
let users = User::query().all(&conn).await?;

// Connection returned when dropped
```

## Pool Metrics

Monitor pool health:

```python
from chakra import get_pool

pool = get_pool()
metrics = pool.metrics()

print(f"Active connections: {metrics.active}")
print(f"Idle connections: {metrics.idle}")
print(f"Waiting requests: {metrics.waiting}")
print(f"Total connections: {metrics.size}")
```

```rust
let metrics = pool.status();

println!("Active: {}", metrics.active);
println!("Idle: {}", metrics.idle);
println!("Waiting: {}", metrics.waiting);
println!("Size: {}", metrics.size);
```

## Connection Health Checks

Chakra validates connections before use:

```toml
[pool]
health_check_interval = "30s"  # Check every 30 seconds
```

```python
# Manual health check
is_healthy = await pool.check_health()

# Health check on acquire
async with pool.acquire(check_health=True) as conn:
    pass
```

## Multi-Database Support

Use multiple pools for different databases:

```python
from chakra import create_pool

# Create named pools
primary_pool = await create_pool(
    "postgresql://primary:5432/db",
    name="primary"
)

replica_pool = await create_pool(
    "postgresql://replica:5432/db",
    name="replica"
)

# Use specific pool
async with Session(pool=primary_pool) as session:
    # Writes go to primary
    session.add(user)
    await session.commit()

async with Session(pool=replica_pool) as session:
    # Reads from replica
    users = await User.objects.all()
```

## Connection Lifecycle

```
┌─────────────────────────────────────────────────────────────────┐
│                        Connection Pool                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐           │
│  │  Conn 1 │  │  Conn 2 │  │  Conn 3 │  │  Conn 4 │  ...      │
│  │  (idle) │  │ (active)│  │  (idle) │  │ (active)│           │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘           │
│       │            │            │            │                  │
│       ▼            ▼            ▼            ▼                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Connection Queue                      │   │
│  │  [Request 1] [Request 2] [Request 3] ...                │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

1. **Request arrives** — Application needs a connection
2. **Check idle pool** — If available, return immediately
3. **Create new** — If pool not at max, create new connection
4. **Wait in queue** — If at max, wait for available connection
5. **Timeout** — If wait exceeds `acquire_timeout`, error

## Best Practices

### Sizing the Pool

```python
# Rule of thumb
max_connections = (2 * cpu_cores) + effective_spindle_count

# For SSD with 4 cores
max_connections = (2 * 4) + 1 = 9  # Round to 10

# For cloud database (pooled already)
# Use smaller pool, as cloud has its own pooling
max_connections = cpu_cores = 4
```

### Connection Reuse

```python
# ❌ Bad: New session per query
async def get_user(id):
    async with Session() as session:
        return await session.get(User, id)

async def get_posts(user_id):
    async with Session() as session:  # Another session!
        return await Post.objects.filter(author_id=user_id).all()

# ✅ Good: Reuse session
async def get_user_with_posts(id, session):
    user = await session.get(User, id)
    posts = await Post.objects.filter(author_id=id).all()
    return user, posts
```

### Handling Exhaustion

```python
from chakra.exceptions import PoolExhausted

try:
    async with Session() as session:
        await session.get(User, 1)
except PoolExhausted:
    # Pool is at max and all connections busy
    # Options:
    # 1. Increase max_connections
    # 2. Reduce connection hold time
    # 3. Add retry logic
    logger.error("Database pool exhausted")
    raise ServiceUnavailable()
```

## Framework Integration

### FastAPI Lifespan

```python
from fastapi import FastAPI
from contextlib import asynccontextmanager
from chakra import create_pool, close_pool

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Startup
    app.state.pool = await create_pool(DATABASE_URL)
    yield
    # Shutdown
    await close_pool(app.state.pool)

app = FastAPI(lifespan=lifespan)
```

### Axum State

```rust
use axum::{Router, extract::State};
use chakra::Pool;

#[tokio::main]
async fn main() {
    let pool = chakra::connect("postgresql://localhost/db").await.unwrap();

    let app = Router::new()
        .route("/users", get(list_users))
        .with_state(pool);

    // Pool closed when app shuts down
}
```
