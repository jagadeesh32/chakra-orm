---
title: Features
description: Explore the powerful features of Chakra ORM
---

# Features

Chakra ORM is built from the ground up to solve the pain points of existing ORMs while introducing capabilities never before seen in the database tooling space.

## Core Features

<div class="grid cards" markdown>

-   :material-lightning-bolt:{ .lg } **[Performance](performance.md)**

    Rust core delivers exceptional speed with minimal overhead.

-   :material-language-python:{ .lg } **[Cross-Language](cross-language.md)**

    First-class Python and Rust support with shared models.

-   :material-shield-check:{ .lg } **[Type Safety](type-safety.md)**

    Compile-time checks in Rust, runtime validation in Python.

-   :material-sync:{ .lg } **[Async Native](async-native.md)**

    Built for async from day one, with sync convenience.

-   :material-database-arrow-up:{ .lg } **[Auto Migrations](migrations.md)**

    Django-quality migrations without Django.

-   :material-pipe:{ .lg } **[Connection Pooling](pooling.md)**

    Efficient connection management for high concurrency.

-   :material-database-multiple:{ .lg } **[Database Support](databases.md)**

    PostgreSQL, MySQL, SQLite, Oracle — unified API.

</div>

---

## Feature Comparison

| Feature | Chakra ORM | Django ORM | SQLAlchemy | Prisma | Diesel |
|---------|:----------:|:----------:|:----------:|:------:|:------:|
| **Languages** | Python + Rust | Python | Python | TypeScript | Rust |
| **Async Support** | Native | Limited | 2.0+ | Yes | No |
| **Sync Support** | Yes | Yes | Yes | Yes | Yes |
| **Auto Migrations** | Yes | Yes | No (Alembic) | Yes | No |
| **Type Safety** | Both | Runtime | Runtime | Build | Compile |
| **Connection Pool** | Built-in | Limited | Yes | Yes | External |
| **Query Builder** | Fluent | Fluent | Flexible | Generated | DSL |
| **Raw SQL** | Yes | Yes | Yes | Yes | Yes |
| **Relationships** | Full | Full | Full | Full | Full |
| **Streaming** | Yes | No | Yes | Yes | No |
| **Framework Lock-in** | None | Django | None | None | None |

---

## What Makes Chakra Different

### 1. True Cross-Language Support

No other ORM works natively in both Python and Rust. Chakra lets you:

- Define models once, use everywhere
- Share database code between microservices
- Migrate Python services to Rust incrementally
- Use the same mental model across your stack

### 2. Rust Core, Python Convenience

```
┌─────────────────────────────────────────┐
│           Python User Code              │  ← Simple, Pythonic API
├─────────────────────────────────────────┤
│           PyO3 Bindings                 │  ← Zero-copy where possible
├─────────────────────────────────────────┤
│           Rust Core Engine              │  ← All heavy lifting
├─────────────────────────────────────────┤
│           Database                      │
└─────────────────────────────────────────┘
```

The Rust core handles:
- Query building and optimization
- SQL generation (dialect-specific)
- Connection pooling
- Result decoding
- Type conversion

Python just orchestrates — no CPU-bound work in Python.

### 3. Django Migrations, Zero Lock-in

Chakra's migration system:

- Auto-detects model changes (like Django)
- Works with any Python/Rust framework
- Uses portable TOML format
- Supports raw SQL escape hatches
- Includes reversibility tracking

### 4. Performance Without Compromise

Benchmarks show Chakra is:

- **2-5x faster** than SQLAlchemy for complex queries
- **Comparable to raw SQL** with < 50μs overhead
- **10x faster** at bulk operations via zero-copy Arrow

---

## Design Philosophy

!!! quote "Guiding Principles"

    1. **Performance First** — Never sacrifice speed for convenience
    2. **Zero Compromise DX** — Intuitive APIs that feel native
    3. **Framework Agnostic** — Work with any stack
    4. **Async Native** — Built for modern concurrent applications
    5. **Type Safe** — Catch errors before production
