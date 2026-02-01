---
title: Chakra ORM
description: Ultra-Fast Cross-Language ORM Framework — Rust Core + Python Bindings
hide:
  - navigation
  - toc
---

<style>
.md-content__button {
  display: none;
}
</style>

<div class="hero" markdown>

# **Chakra ORM**

## Rust-Powered Database Velocity

The first cross-language ORM that combines **Rust performance** with **Python elegance**.

[Get Started](learn/getting-started/installation.md){ .md-button .md-button--primary }
[View on GitHub](https://github.com/chakra-orm/chakra-orm){ .md-button }

</div>

---

<div class="grid cards" markdown>

-   :material-lightning-bolt:{ .lg .middle } **Blazing Fast**

    ---

    Rust core handles query building, SQL generation, connection pooling, and result decoding. Python overhead is minimal.

    [:octicons-arrow-right-24: Performance](features/performance.md)

-   :fontawesome-brands-python:{ .lg .middle } **Python & Rust**

    ---

    First-class support for both languages. Same models, same API patterns, complete feature parity.

    [:octicons-arrow-right-24: Cross-Language](features/cross-language.md)

-   :material-database-sync:{ .lg .middle } **Auto Migrations**

    ---

    Django-quality migration system that auto-detects model changes. Framework-agnostic.

    [:octicons-arrow-right-24: Migrations](features/migrations.md)

-   :material-shield-check:{ .lg .middle } **Type Safe**

    ---

    Compile-time validation in Rust, comprehensive type hints in Python. Catch errors before runtime.

    [:octicons-arrow-right-24: Type Safety](features/type-safety.md)

</div>

---

## Quick Example

=== "Python"

    ```python
    from chakra import Model, Field, String, Integer, Session

    class User(Model):
        __tablename__ = "users"

        id = Field(Integer, primary_key=True)
        name = Field(String(100))
        email = Field(String(255), unique=True)

    # Async queries
    async with Session() as session:
        # Create
        user = User(name="Alice", email="alice@example.com")
        session.add(user)
        await session.commit()

        # Query
        users = await User.objects.filter(
            name__startswith="A"
        ).order_by("-created_at").all()

        # Update
        user.name = "Alice Smith"
        await session.commit()
    ```

=== "Rust"

    ```rust
    use chakra::prelude::*;

    #[derive(Debug, Clone, Model)]
    #[chakra(table = "users")]
    pub struct User {
        #[chakra(primary_key, auto_increment)]
        pub id: i64,
        #[chakra(max_length = 100)]
        pub name: String,
        #[chakra(max_length = 255, unique)]
        pub email: String,
    }

    // Async queries
    let session = Session::new(&pool);

    // Create
    let mut user = User {
        id: 0,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    };
    session.add(&mut user);
    session.commit().await?;

    // Query
    let users = User::query()
        .filter(User::name().starts_with("A"))
        .order_by(User::created_at().desc())
        .all(&pool)
        .await?;
    ```

---

## Why Chakra ORM?

<div class="grid" markdown>

| Feature | Django ORM | SQLAlchemy | Prisma | Diesel | **Chakra** |
|---------|:----------:|:----------:|:------:|:------:|:----------:|
| Python Support | :white_check_mark: | :white_check_mark: | :x: | :x: | :white_check_mark: |
| Rust Support | :x: | :x: | :x: | :white_check_mark: | :white_check_mark: |
| Async Native | :material-minus: | :white_check_mark: | :white_check_mark: | :x: | :white_check_mark: |
| Auto Migrations | :white_check_mark: | :x: | :white_check_mark: | :x: | :white_check_mark: |
| Type Safety | Runtime | Runtime | Build | Compile | **Both** |
| Performance | Medium | Medium | High | Very High | **Very High** |
| Framework Agnostic | :x: | :white_check_mark: | :white_check_mark: | :white_check_mark: | :white_check_mark: |

</div>

---

## Installation

=== "Python"

    ```bash
    pip install chakra-orm
    ```

=== "Rust"

    ```toml
    # Cargo.toml
    [dependencies]
    chakra = "0.1"
    ```

=== "Both"

    ```bash
    # Python bindings include Rust core
    pip install chakra-orm

    # For Rust projects, add to Cargo.toml
    chakra = "0.1"
    ```

---

## Supported Databases

<div class="grid cards" markdown>

-   :simple-postgresql: **PostgreSQL**

    Full support including JSON, arrays, and advanced features.

-   :simple-mysql: **MySQL / MariaDB**

    Complete support for MySQL 8+ and MariaDB 10+.

-   :simple-sqlite: **SQLite**

    Perfect for development, testing, and embedded applications.

-   :simple-oracle: **Oracle**

    Enterprise-grade Oracle Database support.

</div>

---

## Framework Integrations

Works seamlessly with your favorite frameworks:

**Python:** FastAPI, Django, Flask, Starlette, Sanic, any ASGI/WSGI

**Rust:** Axum, Actix Web, Rocket, Warp, Tide

[:octicons-arrow-right-24: See Integration Guides](learn/integrations/index.md)

---

## Community

<div class="grid cards" markdown>

-   :fontawesome-brands-github: **GitHub**

    Star us, report issues, contribute code.

    [:octicons-arrow-right-24: chakra-orm/chakra-orm](https://github.com/chakra-orm/chakra-orm)

-   :fontawesome-brands-discord: **Discord**

    Join our community for help and discussions.

    [:octicons-arrow-right-24: Join Discord](https://discord.gg/chakra-orm)

-   :material-twitter: **Twitter**

    Follow for updates and announcements.

    [:octicons-arrow-right-24: @chakra_orm](https://twitter.com/chakra_orm)

</div>

---

<div class="footer-cta" markdown>

## Ready to get started?

[Read the Documentation](learn/getting-started/installation.md){ .md-button .md-button--primary }
[View Examples](usage/examples/index.md){ .md-button }

</div>
