# Chakra ORM

> **Rust-Powered Database Velocity**

The first cross-language ORM that combines **Rust performance** with **Python elegance**.

[![Python](https://img.shields.io/badge/python-3.9+-blue.svg)](https://www.python.org/downloads/)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## Features

- **Blazing Fast** — Rust core handles query building, SQL generation, and connection pooling
- **Cross-Language** — First-class Python and Rust support with feature parity
- **Async Native** — Built for async from day one, with sync convenience wrappers
- **Type Safe** — Compile-time validation in Rust, runtime + type hints in Python
- **Auto Migrations** — Django-quality migrations without the framework lock-in
- **Database Agnostic** — PostgreSQL, MySQL, SQLite, Oracle with unified API

## Installation

### Python

```bash
pip install chakra-orm
```

### Rust

```toml
[dependencies]
chakra = "0.1"
```

## Quick Example

### Python

```python
from chakra import Model, Field, String, Integer, Session

class User(Model):
    __tablename__ = "users"

    id = Field(Integer, primary_key=True)
    name = Field(String(100))
    email = Field(String(255), unique=True)

async with Session() as session:
    # Create
    user = User(name="Alice", email="alice@example.com")
    session.add(user)
    await session.commit()

    # Query
    users = await User.objects.filter(name__startswith="A").all()
```

### Rust

```rust
use chakra::prelude::*;

#[derive(Model)]
#[chakra(table = "users")]
struct User {
    #[chakra(primary_key)]
    id: i64,
    name: String,
    #[chakra(unique)]
    email: String,
}

let pool = chakra::connect("postgresql://localhost/db").await?;

// Create
let mut user = User { id: 0, name: "Alice".into(), email: "alice@example.com".into() };
user.insert(&pool).await?;

// Query
let users = User::query()
    .filter(User::name().starts_with("A"))
    .all(&pool)
    .await?;
```

## Documentation

Full documentation is available at [https://chakra-orm.dev](https://chakra-orm.dev)

- [Getting Started](https://chakra-orm.dev/learn/getting-started/)
- [Models](https://chakra-orm.dev/learn/models/)
- [Queries](https://chakra-orm.dev/learn/queries/)
- [Migrations](https://chakra-orm.dev/learn/migrations/)
- [API Reference](https://chakra-orm.dev/api/)

## Building the Docs

```bash
# Install dependencies
pip install mkdocs-material mkdocs-minify-plugin mkdocs-git-revision-date-localized-plugin

# Serve locally
mkdocs serve

# Build
mkdocs build
```

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

## License

MIT License - see [LICENSE](LICENSE) for details.

---

**Chakra ORM** — Where Performance Meets Elegance
