---
title: API Documentation
description: Complete API reference for Chakra ORM
---

# API Documentation

Complete API reference for both Python and Rust.

## Python API

The Python API is available via the `chakra` package.

```python
import chakra
from chakra import Model, Field, Session, QuerySet
from chakra import String, Integer, Boolean, DateTime
from chakra import ForeignKey, OneToMany, ManyToMany
from chakra import Q, F, Count, Sum, Avg
```

### Modules

| Module | Description |
|--------|-------------|
| [`chakra`](python/chakra.md) | Main module and configuration |
| [`chakra.models`](python/models.md) | Model base class |
| [`chakra.fields`](python/fields.md) | Field types |
| [`chakra.query`](python/query.md) | QuerySet and expressions |
| [`chakra.session`](python/session.md) | Session management |
| [`chakra.migrate`](python/migrate.md) | Migration API |
| [`chakra.exceptions`](python/exceptions.md) | Exception classes |

---

## Rust API

The Rust API is available via the `chakra` crate.

```rust
use chakra::prelude::*;
use chakra::{Pool, Session, Query};
use chakra::model::Model;
use chakra::query::{Filter, Order};
```

### Modules

| Module | Description |
|--------|-------------|
| [`chakra`](rust/chakra.md) | Main crate and prelude |
| [`chakra::model`](rust/model.md) | Model trait and types |
| [`chakra::query`](rust/query.md) | Query builder |
| [`chakra::session`](rust/session.md) | Session type |
| [`chakra::pool`](rust/pool.md) | Connection pool |
| [`chakra::migrate`](rust/migrate.md) | Migration engine |
| [`chakra::error`](rust/error.md) | Error types |

---

## Quick Reference

### Create

=== "Python"

    ```python
    user = User(name="Alice", email="alice@example.com")
    session.add(user)
    await session.commit()
    ```

=== "Rust"

    ```rust
    let mut user = User { name: "Alice".into(), .. };
    user.insert(&pool).await?;
    ```

### Read

=== "Python"

    ```python
    user = await User.objects.get(id=1)
    users = await User.objects.filter(is_active=True).all()
    ```

=== "Rust"

    ```rust
    let user = User::get(&pool, 1).await?;
    let users = User::query().filter(User::is_active().eq(true)).all(&pool).await?;
    ```

### Update

=== "Python"

    ```python
    user.name = "Alice Smith"
    await session.commit()
    ```

=== "Rust"

    ```rust
    user.name = "Alice Smith".into();
    user.update(&pool).await?;
    ```

### Delete

=== "Python"

    ```python
    session.delete(user)
    await session.commit()
    ```

=== "Rust"

    ```rust
    user.delete(&pool).await?;
    ```
