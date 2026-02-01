---
title: Model Options
description: Configure model behavior with Meta options
---

# Model Options

Configure table-level settings, indexes, and constraints using model options.

## Table Options

### Python

```python
class User(Model):
    __tablename__ = "users"           # Table name (required)
    __schema__ = "public"              # Schema name
    __table_args__ = (                 # Additional table options
        Index("idx_email", "email"),
        UniqueConstraint("email", "tenant_id"),
        {"comment": "User accounts table"},
    )

    class Meta:
        ordering = ["-created_at"]     # Default ordering
        verbose_name = "User"          # Human-readable name
        verbose_name_plural = "Users"
        abstract = False               # True for base classes
```

### Rust

```rust
#[derive(Model)]
#[chakra(table = "users")]
#[chakra(schema = "public")]
#[chakra(comment = "User accounts table")]
#[chakra(indexes = [
    Index::new("idx_email").columns(&["email"]),
])]
pub struct User {
    // fields...
}
```

## Indexes

```python
from chakra import Index

class Article(Model):
    __tablename__ = "articles"
    __table_args__ = (
        # Simple index
        Index("idx_title", "title"),

        # Composite index
        Index("idx_author_date", "author_id", "published_at"),

        # Unique index
        Index("idx_slug", "slug", unique=True),

        # Partial index (PostgreSQL)
        Index("idx_published", "published_at", where="published = true"),

        # Descending index
        Index("idx_created", "created_at", desc=True),
    )
```

## Constraints

```python
from chakra import UniqueConstraint, CheckConstraint, ForeignKeyConstraint

class Order(Model):
    __tablename__ = "orders"
    __table_args__ = (
        # Unique constraint
        UniqueConstraint("order_number", "tenant_id", name="uq_order_tenant"),

        # Check constraint
        CheckConstraint("total >= 0", name="ck_total_positive"),
        CheckConstraint("quantity > 0", name="ck_quantity_positive"),

        # Composite foreign key
        ForeignKeyConstraint(
            ["billing_address_id", "billing_address_type"],
            ["addresses.id", "addresses.type"],
            name="fk_billing_address"
        ),
    )
```

## Abstract Models

```python
class TimestampMixin(Model):
    """Base class with timestamp fields."""

    class Meta:
        abstract = True  # Won't create a table

    created_at = Field(DateTime, default=datetime.utcnow)
    updated_at = Field(DateTime, default=datetime.utcnow, on_update=datetime.utcnow)


class SoftDeleteMixin(Model):
    """Base class with soft delete support."""

    class Meta:
        abstract = True

    deleted_at = Field(DateTime, nullable=True)
    is_deleted = Field(Boolean, default=False)


class User(TimestampMixin, SoftDeleteMixin):
    __tablename__ = "users"

    id = Field(Integer, primary_key=True)
    username = Field(String(50))
    # Inherits: created_at, updated_at, deleted_at, is_deleted
```
