---
title: Installation
description: Install Chakra ORM for Python or Rust
---

# Installation

## Python

### Requirements

- Python 3.9 or higher
- pip or poetry

### Install with pip

```bash
pip install chakra-orm
```

### Install with Poetry

```bash
poetry add chakra-orm
```

### Install with specific database drivers

```bash
# PostgreSQL (recommended)
pip install chakra-orm[postgres]

# MySQL
pip install chakra-orm[mysql]

# SQLite (included by default)
pip install chakra-orm

# All databases
pip install chakra-orm[all]
```

### Verify Installation

```python
import chakra
print(chakra.__version__)
# Output: 0.1.0
```

---

## Rust

### Requirements

- Rust 1.70 or higher
- Cargo

### Add to Cargo.toml

```toml
[dependencies]
chakra = "0.1"
```

### With specific features

```toml
[dependencies]
chakra = { version = "0.1", features = ["postgres", "derive", "runtime-tokio"] }
```

### Available Features

| Feature | Description |
|---------|-------------|
| `postgres` | PostgreSQL support |
| `mysql` | MySQL/MariaDB support |
| `sqlite` | SQLite support |
| `oracle` | Oracle support |
| `derive` | Derive macros for models |
| `runtime-tokio` | Tokio async runtime (default) |
| `runtime-async-std` | async-std runtime |
| `migrations` | Migration support |
| `cli` | Command-line tools |

### Verify Installation

```rust
use chakra::prelude::*;

fn main() {
    println!("Chakra version: {}", chakra::VERSION);
}
```

---

## CLI Tool

The Chakra CLI is included with the Python package:

```bash
# Verify CLI installation
chakra --version

# See available commands
chakra --help
```

For Rust, install the CLI separately:

```bash
cargo install chakra-cli
```

---

## Database Drivers

### PostgreSQL

=== "Python"

    ```bash
    pip install chakra-orm[postgres]
    # Installs: asyncpg
    ```

=== "Rust"

    ```toml
    chakra = { version = "0.1", features = ["postgres"] }
    # Uses: tokio-postgres
    ```

### MySQL

=== "Python"

    ```bash
    pip install chakra-orm[mysql]
    # Installs: aiomysql
    ```

=== "Rust"

    ```toml
    chakra = { version = "0.1", features = ["mysql"] }
    # Uses: mysql_async
    ```

### SQLite

=== "Python"

    ```bash
    pip install chakra-orm
    # SQLite support included by default
    ```

=== "Rust"

    ```toml
    chakra = { version = "0.1", features = ["sqlite"] }
    # Uses: sqlx-sqlite
    ```

---

## Project Setup

### Initialize a New Project

```bash
# Create project directory
mkdir my-app && cd my-app

# Initialize Chakra
chakra init
```

This creates:

```
my-app/
├── chakra.toml          # Configuration
├── migrations/          # Migration files
└── models/              # Example models (optional)
```

### Configuration File

```toml
# chakra.toml
[database]
url = "postgresql://user:pass@localhost:5432/mydb"

[pool]
min_connections = 5
max_connections = 20

[migrations]
directory = "migrations"
```

---

## Next Steps

- [Quick Start (Python)](quickstart-python.md) — Build your first app
- [Quick Start (Rust)](quickstart-rust.md) — Build your first Rust app
- [Configuration](configuration.md) — Advanced configuration options
