---
title: Learn Chakra ORM
description: Comprehensive tutorials and guides for Chakra ORM
---

# Learn Chakra ORM

Welcome to the Chakra ORM learning path. Whether you're new to ORMs or migrating from another tool, these guides will help you become productive quickly.

## Learning Path

<div class="grid cards" markdown>

-   :material-rocket-launch:{ .lg } **Getting Started**

    ---

    Install Chakra and build your first application.

    [:octicons-arrow-right-24: Start here](getting-started/installation.md)

-   :material-cube-outline:{ .lg } **Models**

    ---

    Define your data models and relationships.

    [:octicons-arrow-right-24: Learn models](models/index.md)

-   :material-magnify:{ .lg } **Queries**

    ---

    Master the query API for filtering, aggregating, and joining.

    [:octicons-arrow-right-24: Learn queries](queries/index.md)

-   :material-database-sync:{ .lg } **Sessions**

    ---

    Understand the Unit of Work pattern and transactions.

    [:octicons-arrow-right-24: Learn sessions](sessions/index.md)

-   :material-arrow-up-bold:{ .lg } **Migrations**

    ---

    Manage database schema changes safely.

    [:octicons-arrow-right-24: Learn migrations](migrations/index.md)

-   :material-puzzle:{ .lg } **Integrations**

    ---

    Connect Chakra to your favorite framework.

    [:octicons-arrow-right-24: See integrations](integrations/index.md)

</div>

---

## Quick Links

| I want to... | Go to... |
|--------------|----------|
| Install Chakra | [Installation Guide](getting-started/installation.md) |
| See a quick example | [Quick Start (Python)](getting-started/quickstart-python.md) |
| Define a model | [Defining Models](models/defining-models.md) |
| Query the database | [Basic Queries](queries/basic.md) |
| Use transactions | [Transactions](sessions/transactions.md) |
| Create a migration | [Creating Migrations](migrations/creating.md) |
| Use with FastAPI | [FastAPI Integration](integrations/fastapi.md) |
| Use with Axum | [Axum Integration](integrations/axum.md) |

---

## By Experience Level

### New to ORMs?

Start with the basics:

1. [Installation](getting-started/installation.md)
2. [Quick Start](getting-started/quickstart-python.md)
3. [Defining Models](models/defining-models.md)
4. [Basic Queries](queries/basic.md)

### Coming from Django ORM?

You'll feel right at home:

- Similar model definition syntax
- Familiar QuerySet API with `filter()`, `exclude()`, etc.
- Same migration workflow with `make` and `apply`
- [Django Integration Guide](integrations/django.md)

### Coming from SQLAlchemy?

Key differences:

- Simpler model definition (no declarative base needed)
- Session is lighter weight
- Migrations are built-in (no Alembic needed)
- [Comparison Guide](../reference/sqlalchemy-comparison.md)

### Rust Developer?

Start here:

1. [Quick Start (Rust)](getting-started/quickstart-rust.md)
2. [Rust Model Macros](../reference/rust/macros.md)
3. [Axum Integration](integrations/axum.md)

---

## Code Examples

### Python Example

```python
from chakra import Model, Field, String, Integer, Session

class User(Model):
    __tablename__ = "users"

    id = Field(Integer, primary_key=True)
    name = Field(String(100))
    email = Field(String(255), unique=True)

async def main():
    async with Session() as session:
        # Create
        user = User(name="Alice", email="alice@example.com")
        session.add(user)
        await session.commit()

        # Query
        users = await User.objects.filter(name__startswith="A").all()

        # Update
        user.name = "Alice Smith"
        await session.commit()

        # Delete
        session.delete(user)
        await session.commit()
```

### Rust Example

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

#[tokio::main]
async fn main() -> Result<()> {
    let pool = chakra::connect("postgresql://localhost/db").await?;

    // Create
    let mut user = User {
        id: 0,
        name: "Alice".into(),
        email: "alice@example.com".into(),
    };
    user.insert(&pool).await?;

    // Query
    let users = User::query()
        .filter(User::name().starts_with("A"))
        .all(&pool)
        .await?;

    // Update
    user.name = "Alice Smith".into();
    user.update(&pool).await?;

    // Delete
    user.delete(&pool).await?;

    Ok(())
}
```

---

## Need Help?

- **Discord**: Join our [community server](https://discord.gg/chakra-orm)
- **GitHub Issues**: [Report bugs or request features](https://github.com/chakra-orm/chakra-orm/issues)
- **Stack Overflow**: Tag your questions with `chakra-orm`
