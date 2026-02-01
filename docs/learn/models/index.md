---
title: Models
description: Learn how to define and work with Chakra ORM models
---

# Models

Models are the foundation of Chakra ORM. They define the structure of your database tables and the relationships between them.

## In This Section

- [Defining Models](defining-models.md) — Create your first model
- [Field Types](field-types.md) — Available field types and options
- [Relationships](relationships.md) — One-to-one, one-to-many, many-to-many
- [Model Options](options.md) — Table options, indexes, constraints

## Quick Overview

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

        posts = OneToMany("Post", back_populates="author")
    ```

=== "Rust"

    ```rust
    use chakra::prelude::*;
    use chrono::{DateTime, Utc};

    #[derive(Model)]
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

        #[chakra(relation = OneToMany, foreign_key = "author_id")]
        pub posts: Related<Vec<Post>>,
    }
    ```

## Key Concepts

### Tables

Each model maps to a database table:

```python
class User(Model):
    __tablename__ = "users"  # Explicit table name
```

### Fields

Fields define columns with types and constraints:

```python
username = Field(String(50), unique=True, index=True)
```

### Relationships

Connect models together:

```python
posts = OneToMany("Post", back_populates="author")
```

### Primary Keys

Every model needs a primary key:

```python
id = Field(Integer, primary_key=True)
# or
id = Field(BigInteger, primary_key=True, auto_increment=True)
```
