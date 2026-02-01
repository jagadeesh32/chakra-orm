---
title: Database Support
description: Supported databases and their specific features
tags:
  - postgresql
  - mysql
  - sqlite
  - oracle
---

# Database Support

Chakra ORM supports multiple databases through a unified API while preserving database-specific features.

## Supported Databases

| Database | Version | Status | Driver |
|----------|---------|--------|--------|
| PostgreSQL | 12+ | ✅ Full Support | tokio-postgres |
| MySQL | 8.0+ | ✅ Full Support | mysql_async |
| MariaDB | 10.5+ | ✅ Full Support | mysql_async |
| SQLite | 3.35+ | ✅ Full Support | sqlx-sqlite |
| Oracle | 19c+ | ✅ Full Support | oracle |

## Connection URLs

```python
# PostgreSQL
DATABASE_URL = "postgresql://user:pass@localhost:5432/dbname"

# MySQL
DATABASE_URL = "mysql://user:pass@localhost:3306/dbname"

# SQLite
DATABASE_URL = "sqlite:///path/to/database.db"
DATABASE_URL = "sqlite:///:memory:"  # In-memory

# Oracle
DATABASE_URL = "oracle://user:pass@localhost:1521/service_name"
```

## PostgreSQL

### Supported Features

- ✅ All standard SQL operations
- ✅ JSONB fields with indexing
- ✅ Array fields
- ✅ UUID fields
- ✅ Full-text search
- ✅ Range types
- ✅ LISTEN/NOTIFY
- ✅ COPY for bulk operations
- ✅ Materialized views
- ✅ Table partitioning

### PostgreSQL-Specific Types

```python
from chakra import Model, Field
from chakra.postgres import (
    JSONB, Array, UUID, TSVector, DateRange, Int4Range,
    Inet, Cidr, MacAddr, HStore
)

class Document(Model):
    __tablename__ = "documents"

    id = Field(UUID, primary_key=True, default="gen_random_uuid()")
    data = Field(JSONB, default={})
    tags = Field(Array(String(50)))
    search_vector = Field(TSVector)
    ip_address = Field(Inet)
```

### JSONB Queries

```python
# Filter by JSON field
docs = await Document.objects.filter(
    data__title="Hello",           # data->>'title' = 'Hello'
    data__author__name="Alice",    # data->'author'->>'name' = 'Alice'
    data__has_key="status",        # data ? 'status'
    data__contains={"type": "post"},  # data @> '{"type": "post"}'
).all()
```

### Array Operations

```python
# Filter by array
docs = await Document.objects.filter(
    tags__contains=["python"],      # 'python' = ANY(tags)
    tags__overlap=["rust", "go"],   # tags && ARRAY['rust', 'go']
    tags__len=3,                    # array_length(tags, 1) = 3
).all()
```

## MySQL / MariaDB

### Supported Features

- ✅ All standard SQL operations
- ✅ JSON fields
- ✅ Full-text search (InnoDB)
- ✅ Spatial types
- ✅ Transactions (InnoDB)
- ✅ Character sets / collations

### MySQL-Specific Types

```python
from chakra.mysql import (
    JSON, Enum, Set, Year, Geometry, Point
)

class Product(Model):
    __tablename__ = "products"

    id = Field(Integer, primary_key=True)
    attributes = Field(JSON)
    status = Field(Enum("draft", "published", "archived"))
    flags = Field(Set("featured", "sale", "new"))
    location = Field(Point)
```

### MySQL Configuration

```toml
[database]
url = "mysql://user:pass@localhost:3306/db"

[database.options]
charset = "utf8mb4"
collation = "utf8mb4_unicode_ci"
```

## SQLite

### Supported Features

- ✅ All standard SQL operations
- ✅ JSON functions (3.38+)
- ✅ Full-text search (FTS5)
- ✅ Window functions
- ✅ Common table expressions
- ✅ In-memory databases

### SQLite Configuration

```toml
[database]
url = "sqlite:///app.db"

[database.options]
journal_mode = "WAL"        # Write-ahead logging
synchronous = "NORMAL"      # Balance speed/safety
foreign_keys = true         # Enable FK constraints
busy_timeout = 5000         # 5 second busy timeout
```

### In-Memory Database

```python
# Perfect for testing
from chakra import create_pool

pool = await create_pool("sqlite:///:memory:")

# Or named in-memory (shared between connections)
pool = await create_pool("sqlite:///file:memdb?mode=memory&cache=shared")
```

## Oracle

### Supported Features

- ✅ All standard SQL operations
- ✅ PL/SQL stored procedures
- ✅ Sequences
- ✅ Table partitioning
- ✅ Materialized views
- ✅ LOB types (CLOB, BLOB)

### Oracle-Specific Configuration

```toml
[database]
url = "oracle://user:pass@localhost:1521/ORCL"

[database.options]
encoding = "UTF-8"
nencoding = "UTF-8"
```

### Oracle Sequences

```python
class Order(Model):
    __tablename__ = "orders"

    id = Field(Integer, primary_key=True)

    class Meta:
        # Use Oracle sequence
        sequence = "order_seq"
```

## Dialect Differences

Chakra abstracts most differences, but some behaviors vary:

| Feature | PostgreSQL | MySQL | SQLite | Oracle |
|---------|------------|-------|--------|--------|
| Auto-increment | SERIAL | AUTO_INCREMENT | AUTOINCREMENT | SEQUENCE |
| Boolean | BOOLEAN | TINYINT(1) | INTEGER | NUMBER(1) |
| UUID | UUID | CHAR(36) | TEXT | RAW(16) |
| JSON | JSONB | JSON | TEXT+JSON | CLOB |
| Upsert | ON CONFLICT | ON DUPLICATE KEY | ON CONFLICT | MERGE |
| Limit | LIMIT | LIMIT | LIMIT | FETCH FIRST |
| String concat | \|\| | CONCAT() | \|\| | \|\| |

## Feature Detection

Check database capabilities at runtime:

```python
from chakra import get_pool

pool = get_pool()
dialect = pool.dialect()

if dialect.supports_returning:
    # PostgreSQL, SQLite 3.35+
    user = await User.objects.create(name="Alice")  # Returns created row

if dialect.supports_json:
    # PostgreSQL, MySQL 5.7+, SQLite 3.38+
    docs = await Doc.objects.filter(data__key="value").all()

if dialect.supports_arrays:
    # PostgreSQL only
    items = await Item.objects.filter(tags__contains=["featured"]).all()
```

## Multi-Database Queries

```python
from chakra import Session, create_pool

# Create pools for different databases
pg_pool = await create_pool("postgresql://localhost/main")
sqlite_pool = await create_pool("sqlite:///cache.db")

# Use appropriate pool for each operation
async with Session(pool=pg_pool) as session:
    users = await User.objects.all()

async with Session(pool=sqlite_pool) as session:
    cache = await CacheEntry.objects.filter(key="users").first()
```

## Testing with Different Databases

```python
import pytest
from chakra import create_pool

@pytest.fixture(params=[
    "postgresql://localhost/test",
    "mysql://localhost/test",
    "sqlite:///:memory:",
])
async def pool(request):
    pool = await create_pool(request.param)
    await chakra.migrate.apply(pool)
    yield pool
    await pool.close()

async def test_crud(pool):
    # Test runs against all databases
    user = User(name="Test")
    async with Session(pool=pool) as session:
        session.add(user)
        await session.commit()
        assert user.id is not None
```
