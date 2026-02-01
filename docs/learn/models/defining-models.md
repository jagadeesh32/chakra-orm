---
title: Defining Models
description: Learn how to define database models in Chakra ORM
---

# Defining Models

Models are Python classes or Rust structs that represent database tables.

## Python Models

### Basic Model

```python
from chakra import Model, Field, String, Integer, Boolean, DateTime
from datetime import datetime

class User(Model):
    # Table name (required)
    __tablename__ = "users"

    # Primary key (required)
    id = Field(Integer, primary_key=True)

    # Regular fields
    username = Field(String(50))
    email = Field(String(255))
    is_active = Field(Boolean, default=True)
    created_at = Field(DateTime, default=datetime.utcnow)
```

### Field Options

```python
class User(Model):
    __tablename__ = "users"

    id = Field(Integer,
        primary_key=True,     # This is the primary key
    )

    username = Field(String(50),
        unique=True,          # UNIQUE constraint
        index=True,           # Create index
    )

    email = Field(String(255),
        unique=True,
        nullable=False,       # NOT NULL (default)
    )

    bio = Field(Text,
        nullable=True,        # Allow NULL
    )

    age = Field(Integer,
        nullable=True,
        default=None,
    )

    is_active = Field(Boolean,
        default=True,         # Default value
    )

    created_at = Field(DateTime,
        default=datetime.utcnow,  # Callable default
    )

    updated_at = Field(DateTime,
        default=datetime.utcnow,
        on_update=datetime.utcnow,  # Update on save
    )
```

### Model Meta Options

```python
from chakra import Model, Field, Index, UniqueConstraint, CheckConstraint

class User(Model):
    __tablename__ = "users"

    __table_args__ = (
        # Composite index
        Index("idx_user_email_tenant", "email", "tenant_id"),

        # Composite unique constraint
        UniqueConstraint("email", "tenant_id", name="uq_email_tenant"),

        # Check constraint
        CheckConstraint("age >= 0", name="ck_age_positive"),
    )

    id = Field(Integer, primary_key=True)
    email = Field(String(255))
    tenant_id = Field(Integer)
    age = Field(Integer)

    class Meta:
        # Default ordering
        ordering = ["-created_at"]

        # Verbose names
        verbose_name = "User"
        verbose_name_plural = "Users"

        # Abstract base (not a real table)
        abstract = False
```

---

## Rust Models

### Basic Model

```rust
use chakra::prelude::*;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Model)]
#[chakra(table = "users")]
pub struct User {
    #[chakra(primary_key, auto_increment)]
    pub id: i64,

    #[chakra(max_length = 50)]
    pub username: String,

    #[chakra(max_length = 255)]
    pub email: String,

    #[chakra(default = true)]
    pub is_active: bool,

    #[chakra(default = "now()")]
    pub created_at: DateTime<Utc>,
}
```

### Field Attributes

```rust
#[derive(Model)]
#[chakra(table = "users")]
pub struct User {
    // Primary key with auto-increment
    #[chakra(primary_key, auto_increment)]
    pub id: i64,

    // Unique with index
    #[chakra(max_length = 50, unique, index)]
    pub username: String,

    // Just unique constraint
    #[chakra(max_length = 255, unique)]
    pub email: String,

    // Nullable field (use Option<T>)
    #[chakra(nullable)]
    pub bio: Option<String>,

    // With default value
    #[chakra(default = true)]
    pub is_active: bool,

    // SQL default expression
    #[chakra(default = "now()")]
    pub created_at: DateTime<Utc>,

    // Foreign key
    #[chakra(foreign_key = "tenants.id")]
    pub tenant_id: i64,
}
```

### Model-Level Attributes

```rust
#[derive(Model)]
#[chakra(table = "users")]
#[chakra(indexes = [
    Index::new("idx_user_email").columns(&["email"]),
    Index::new("idx_user_tenant").columns(&["tenant_id", "created_at"]).descending(),
])]
#[chakra(constraints = [
    Unique::new("uq_email_tenant").columns(&["email", "tenant_id"]),
    Check::new("ck_age").expression("age >= 0"),
])]
pub struct User {
    // ...
}
```

---

## Composite Primary Keys

### Python

```python
class PostTag(Model):
    __tablename__ = "post_tags"

    post_id = Field(Integer, ForeignKey("posts.id"), primary_key=True)
    tag_id = Field(Integer, ForeignKey("tags.id"), primary_key=True)
    created_at = Field(DateTime, default=datetime.utcnow)
```

### Rust

```rust
#[derive(Model)]
#[chakra(table = "post_tags")]
pub struct PostTag {
    #[chakra(primary_key, foreign_key = "posts.id")]
    pub post_id: i64,

    #[chakra(primary_key, foreign_key = "tags.id")]
    pub tag_id: i64,

    #[chakra(default = "now()")]
    pub created_at: DateTime<Utc>,
}
```

---

## Abstract Base Models

### Python

```python
class TimestampMixin(Model):
    class Meta:
        abstract = True

    created_at = Field(DateTime, default=datetime.utcnow)
    updated_at = Field(DateTime, default=datetime.utcnow, on_update=datetime.utcnow)

class User(TimestampMixin):
    __tablename__ = "users"

    id = Field(Integer, primary_key=True)
    username = Field(String(50))
    # Inherits created_at and updated_at
```

### Rust

```rust
// Use a trait for shared behavior
pub trait Timestamped {
    fn created_at(&self) -> DateTime<Utc>;
    fn updated_at(&self) -> DateTime<Utc>;
}

#[derive(Model)]
#[chakra(table = "users")]
pub struct User {
    #[chakra(primary_key)]
    pub id: i64,
    pub username: String,
    #[chakra(default = "now()")]
    pub created_at: DateTime<Utc>,
    #[chakra(default = "now()")]
    pub updated_at: DateTime<Utc>,
}

impl Timestamped for User {
    fn created_at(&self) -> DateTime<Utc> { self.created_at }
    fn updated_at(&self) -> DateTime<Utc> { self.updated_at }
}
```

---

## Model Methods

### Python

```python
class User(Model):
    __tablename__ = "users"

    id = Field(Integer, primary_key=True)
    first_name = Field(String(50))
    last_name = Field(String(50))
    email = Field(String(255))

    @property
    def full_name(self) -> str:
        return f"{self.first_name} {self.last_name}"

    def __repr__(self) -> str:
        return f"<User {self.email}>"

    async def send_email(self, subject: str, body: str):
        # Custom method
        await send_email(self.email, subject, body)
```

### Rust

```rust
#[derive(Model)]
#[chakra(table = "users")]
pub struct User {
    #[chakra(primary_key)]
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

impl User {
    pub fn full_name(&self) -> String {
        format!("{} {}", self.first_name, self.last_name)
    }

    pub async fn send_email(&self, subject: &str, body: &str) -> Result<()> {
        send_email(&self.email, subject, body).await
    }
}
```
