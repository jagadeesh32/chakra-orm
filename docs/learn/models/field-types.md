---
title: Field Types
description: Complete reference for all Chakra ORM field types
---

# Field Types

Chakra ORM provides a comprehensive set of field types that map to database column types.

## Numeric Types

| Python Type | Rust Type | SQL Type | Description |
|-------------|-----------|----------|-------------|
| `Integer` | `i32` | INTEGER | 32-bit integer |
| `BigInteger` | `i64` | BIGINT | 64-bit integer |
| `SmallInteger` | `i16` | SMALLINT | 16-bit integer |
| `Float` | `f64` | DOUBLE | Double precision float |
| `Decimal(p, s)` | `Decimal` | DECIMAL(p,s) | Fixed precision decimal |

### Examples

=== "Python"

    ```python
    from chakra import Field, Integer, BigInteger, Float, Decimal

    class Product(Model):
        id = Field(BigInteger, primary_key=True)
        quantity = Field(Integer, default=0)
        price = Field(Decimal(10, 2))
        rating = Field(Float, nullable=True)
    ```

=== "Rust"

    ```rust
    use rust_decimal::Decimal;

    #[derive(Model)]
    pub struct Product {
        #[chakra(primary_key)]
        pub id: i64,
        #[chakra(default = 0)]
        pub quantity: i32,
        #[chakra(precision = 10, scale = 2)]
        pub price: Decimal,
        pub rating: Option<f64>,
    }
    ```

## String Types

| Python Type | Rust Type | SQL Type | Description |
|-------------|-----------|----------|-------------|
| `String(n)` | `String` | VARCHAR(n) | Variable length string |
| `Text` | `String` | TEXT | Unlimited text |
| `Char(n)` | `String` | CHAR(n) | Fixed length string |

### Examples

=== "Python"

    ```python
    from chakra import Field, String, Text, Char

    class Article(Model):
        title = Field(String(200))
        slug = Field(String(200), unique=True)
        content = Field(Text)
        country_code = Field(Char(2))
    ```

=== "Rust"

    ```rust
    #[derive(Model)]
    pub struct Article {
        #[chakra(max_length = 200)]
        pub title: String,
        #[chakra(max_length = 200, unique)]
        pub slug: String,
        pub content: String,
        #[chakra(fixed_length = 2)]
        pub country_code: String,
    }
    ```

## Date/Time Types

| Python Type | Rust Type | SQL Type | Description |
|-------------|-----------|----------|-------------|
| `DateTime` | `DateTime<Utc>` | TIMESTAMP | Date and time |
| `Date` | `NaiveDate` | DATE | Date only |
| `Time` | `NaiveTime` | TIME | Time only |
| `Interval` | `Duration` | INTERVAL | Time duration |

### Examples

=== "Python"

    ```python
    from chakra import Field, DateTime, Date, Time
    from datetime import datetime, date, time

    class Event(Model):
        name = Field(String(100))
        start_date = Field(Date)
        start_time = Field(Time)
        created_at = Field(DateTime, default=datetime.utcnow)
    ```

=== "Rust"

    ```rust
    use chrono::{DateTime, Utc, NaiveDate, NaiveTime};

    #[derive(Model)]
    pub struct Event {
        pub name: String,
        pub start_date: NaiveDate,
        pub start_time: NaiveTime,
        #[chakra(default = "now()")]
        pub created_at: DateTime<Utc>,
    }
    ```

## Boolean Type

| Python Type | Rust Type | SQL Type |
|-------------|-----------|----------|
| `Boolean` | `bool` | BOOLEAN |

```python
is_active = Field(Boolean, default=True)
is_verified = Field(Boolean, default=False)
```

## Binary Types

| Python Type | Rust Type | SQL Type | Description |
|-------------|-----------|----------|-------------|
| `Binary` | `Vec<u8>` | BYTEA | Binary data |
| `LargeBinary` | `Vec<u8>` | BLOB | Large binary |

```python
from chakra import Field, Binary

class File(Model):
    name = Field(String(255))
    content = Field(Binary)
    checksum = Field(Binary(32))  # Fixed size
```

## UUID Type

| Python Type | Rust Type | SQL Type |
|-------------|-----------|----------|
| `UUID` | `Uuid` | UUID |

=== "Python"

    ```python
    from chakra import Field, UUID
    import uuid

    class Document(Model):
        id = Field(UUID, primary_key=True, default=uuid.uuid4)
    ```

=== "Rust"

    ```rust
    use uuid::Uuid;

    #[derive(Model)]
    pub struct Document {
        #[chakra(primary_key, default = "gen_random_uuid()")]
        pub id: Uuid,
    }
    ```

## JSON Type

| Python Type | Rust Type | SQL Type |
|-------------|-----------|----------|
| `JSON` | `serde_json::Value` | JSONB |

=== "Python"

    ```python
    from chakra import Field, JSON

    class User(Model):
        settings = Field(JSON, default={})
        metadata = Field(JSON, nullable=True)
    ```

=== "Rust"

    ```rust
    use serde_json::Value;

    #[derive(Model)]
    pub struct User {
        #[chakra(json, default = "{}")]
        pub settings: Value,
        #[chakra(json)]
        pub metadata: Option<Value>,
    }
    ```

## Array Types (PostgreSQL)

| Python Type | Rust Type | SQL Type |
|-------------|-----------|----------|
| `Array(T)` | `Vec<T>` | T[] |

```python
from chakra import Field, Array, String, Integer

class Article(Model):
    tags = Field(Array(String(50)), default=[])
    scores = Field(Array(Integer))
```

## Enum Types

=== "Python"

    ```python
    from chakra import Field, Enum
    from enum import Enum as PyEnum

    class Status(PyEnum):
        DRAFT = "draft"
        PUBLISHED = "published"
        ARCHIVED = "archived"

    class Article(Model):
        status = Field(Enum(Status), default=Status.DRAFT)
    ```

=== "Rust"

    ```rust
    #[derive(Debug, Clone, ChakraEnum)]
    pub enum Status {
        Draft,
        Published,
        Archived,
    }

    #[derive(Model)]
    pub struct Article {
        #[chakra(default = "draft")]
        pub status: Status,
    }
    ```

## Field Options Reference

| Option | Description | Example |
|--------|-------------|---------|
| `primary_key` | Mark as primary key | `primary_key=True` |
| `auto_increment` | Auto-increment (integers) | `auto_increment=True` |
| `unique` | UNIQUE constraint | `unique=True` |
| `index` | Create index | `index=True` |
| `nullable` | Allow NULL | `nullable=True` |
| `default` | Default value | `default=0` |
| `on_update` | Value on update | `on_update=datetime.utcnow` |
| `max_length` | String max length | `String(100)` |
| `min_length` | String min length | `min_length=3` |
| `min_value` | Numeric minimum | `min_value=0` |
| `max_value` | Numeric maximum | `max_value=100` |

## Type Mapping by Database

| Chakra | PostgreSQL | MySQL | SQLite | Oracle |
|--------|------------|-------|--------|--------|
| Integer | INTEGER | INT | INTEGER | NUMBER(10) |
| BigInteger | BIGINT | BIGINT | INTEGER | NUMBER(19) |
| Float | DOUBLE PRECISION | DOUBLE | REAL | BINARY_DOUBLE |
| Decimal | NUMERIC | DECIMAL | TEXT | NUMBER |
| String | VARCHAR | VARCHAR | TEXT | VARCHAR2 |
| Text | TEXT | LONGTEXT | TEXT | CLOB |
| Boolean | BOOLEAN | TINYINT(1) | INTEGER | NUMBER(1) |
| DateTime | TIMESTAMPTZ | DATETIME | TEXT | TIMESTAMP |
| Date | DATE | DATE | TEXT | DATE |
| UUID | UUID | CHAR(36) | TEXT | RAW(16) |
| JSON | JSONB | JSON | TEXT | CLOB |
| Binary | BYTEA | LONGBLOB | BLOB | BLOB |
