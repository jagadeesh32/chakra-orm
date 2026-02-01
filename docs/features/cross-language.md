---
title: Cross-Language Support
description: Use Chakra ORM seamlessly in both Python and Rust
tags:
  - python
  - rust
  - polyglot
---

# Cross-Language Support

Chakra ORM is the first ORM designed from the ground up for both Python and Rust. Same concepts, same patterns, same power — in your language of choice.

## Why Cross-Language?

Modern applications often span multiple languages:

- **Microservices** in different languages sharing a database
- **Performance-critical paths** rewritten in Rust
- **Teams** with different language preferences
- **Gradual migrations** from Python to Rust

Chakra lets you use a consistent database layer across all these scenarios.

## Shared Concepts

| Concept | Python | Rust |
|---------|--------|------|
| Model Definition | Class with Fields | Struct with derive macro |
| Query Builder | Method chaining | Method chaining |
| Session/Unit of Work | Context manager | RAII guard |
| Relationships | Descriptors | Related<T> wrapper |
| Migrations | Same TOML files | Same TOML files |

## Side-by-Side Examples

### Model Definition

=== "Python"

    ```python
    from chakra import Model, Field, String, Integer, DateTime
    from chakra import ForeignKey, OneToMany
    from datetime import datetime

    class User(Model):
        __tablename__ = "users"

        id = Field(Integer, primary_key=True)
        username = Field(String(50), unique=True)
        email = Field(String(255))
        created_at = Field(DateTime, default=datetime.utcnow)

        # Relationships
        posts = OneToMany("Post", back_populates="author")
    ```

=== "Rust"

    ```rust
    use chakra::prelude::*;
    use chrono::{DateTime, Utc};

    #[derive(Debug, Clone, Model)]
    #[chakra(table = "users")]
    pub struct User {
        #[chakra(primary_key, auto_increment)]
        pub id: i64,

        #[chakra(max_length = 50, unique)]
        pub username: String,

        #[chakra(max_length = 255)]
        pub email: String,

        #[chakra(default = "now()")]
        pub created_at: DateTime<Utc>,

        // Relationships
        #[chakra(relation = OneToMany, foreign_key = "author_id")]
        pub posts: Related<Vec<Post>>,
    }
    ```

### Basic Queries

=== "Python"

    ```python
    # Get by ID
    user = await User.objects.get(id=1)

    # Filter
    users = await User.objects.filter(
        username__startswith="a",
        is_active=True
    ).all()

    # Order and limit
    users = await User.objects.order_by("-created_at").limit(10).all()

    # Count
    count = await User.objects.filter(is_active=True).count()
    ```

=== "Rust"

    ```rust
    // Get by ID
    let user = User::get(&pool, 1).await?;

    // Filter
    let users = User::query()
        .filter(User::username().starts_with("a"))
        .filter(User::is_active().eq(true))
        .all(&pool)
        .await?;

    // Order and limit
    let users = User::query()
        .order_by(User::created_at().desc())
        .limit(10)
        .all(&pool)
        .await?;

    // Count
    let count = User::query()
        .filter(User::is_active().eq(true))
        .count(&pool)
        .await?;
    ```

### Creating Records

=== "Python"

    ```python
    async with Session() as session:
        user = User(
            username="alice",
            email="alice@example.com"
        )
        session.add(user)
        await session.commit()

        print(f"Created user with ID: {user.id}")
    ```

=== "Rust"

    ```rust
    let session = Session::new(&pool);

    let mut user = User {
        id: 0,
        username: "alice".into(),
        email: "alice@example.com".into(),
        ..Default::default()
    };

    session.add(&mut user);
    session.commit().await?;

    println!("Created user with ID: {}", user.id);
    ```

### Relationships

=== "Python"

    ```python
    # Eager loading with JOIN
    users = await User.objects.select_related("profile").all()

    # Eager loading with separate queries
    users = await User.objects.prefetch_related("posts").all()

    for user in users:
        for post in user.posts:
            print(f"{user.username}: {post.title}")
    ```

=== "Rust"

    ```rust
    // Eager loading with JOIN
    let users = User::query()
        .select_related(User::profile())
        .all(&pool)
        .await?;

    // Eager loading with separate queries
    let users = User::query()
        .prefetch_related(User::posts())
        .all(&pool)
        .await?;

    for user in &users {
        for post in user.posts.get()? {
            println!("{}: {}", user.username, post.title);
        }
    }
    ```

### Transactions

=== "Python"

    ```python
    async with Session() as session:
        async with session.begin():
            user1 = await session.get(User, 1)
            user2 = await session.get(User, 2)

            user1.balance -= 100
            user2.balance += 100

            # Auto-commits on exit, rolls back on exception
    ```

=== "Rust"

    ```rust
    let session = Session::new(&pool);

    session.transaction(|tx| async move {
        let mut user1 = tx.get::<User>(1).await?.unwrap();
        let mut user2 = tx.get::<User>(2).await?.unwrap();

        user1.balance -= 100;
        user2.balance += 100;

        Ok(())
    }).await?;
    ```

## Migration Sharing

Migrations are language-agnostic TOML files:

```toml
# migrations/users/0001_initial.toml
# Used by BOTH Python and Rust applications

[migration]
id = "0001_initial"
app = "users"

[[operations]]
type = "CreateModel"
name = "User"
table = "users"

[[operations.fields]]
name = "id"
type = "BigInteger"
primary_key = true
auto_increment = true

[[operations.fields]]
name = "username"
type = "String"
max_length = 50
unique = true
```

Both Python and Rust applications can:

- Read the same migration files
- Apply them with `chakra migrate apply`
- Generate new ones with `chakra migrate make`

## Polyglot Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Shared Database                       │
└─────────────────────────────────────────────────────────┘
         ▲                    ▲                    ▲
         │                    │                    │
┌────────┴────────┐  ┌────────┴────────┐  ┌────────┴────────┐
│  Python Service │  │   Rust Service  │  │  Python Worker  │
│                 │  │                 │  │                 │
│  from chakra    │  │  use chakra::   │  │  from chakra    │
│  import Model   │  │  prelude::*;    │  │  import Session │
└─────────────────┘  └─────────────────┘  └─────────────────┘
         │                    │                    │
         └────────────────────┼────────────────────┘
                              │
                    ┌─────────┴─────────┐
                    │ Shared Migrations │
                    │  (TOML files)     │
                    └───────────────────┘
```

## Type Mapping

Types map consistently between languages:

| Python Type | Rust Type | SQL Type |
|-------------|-----------|----------|
| `int` | `i32` / `i64` | INTEGER / BIGINT |
| `str` | `String` | VARCHAR / TEXT |
| `bool` | `bool` | BOOLEAN |
| `float` | `f64` | DOUBLE |
| `Decimal` | `rust_decimal::Decimal` | DECIMAL |
| `datetime` | `chrono::DateTime<Utc>` | TIMESTAMP |
| `date` | `chrono::NaiveDate` | DATE |
| `bytes` | `Vec<u8>` | BYTEA |
| `dict` (JSON) | `serde_json::Value` | JSONB |
| `list` (Array) | `Vec<T>` | ARRAY |
| `UUID` | `uuid::Uuid` | UUID |

## When to Use Each Language

| Use Python When | Use Rust When |
|-----------------|---------------|
| Rapid prototyping | Performance-critical paths |
| Data science / ML pipelines | High-throughput services |
| Web applications (FastAPI, Django) | Systems programming |
| Scripting and automation | When you need compile-time safety |
| Team is Python-focused | Latency-sensitive operations |
