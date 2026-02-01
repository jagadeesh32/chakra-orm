---
title: Auto Migrations
description: Django-quality migrations without the framework lock-in
tags:
  - migrations
  - schema
  - database
---

# Auto Migrations

Chakra ORM provides Django-quality migrations with automatic change detection, but without requiring Django or any specific framework.

## Key Features

- **Auto-detection** — Chakra detects model changes automatically
- **Framework-agnostic** — Works with any Python or Rust application
- **Reversible** — Most operations can be rolled back
- **Version-controlled** — TOML files that belong in git
- **Raw SQL support** — Escape hatch for complex operations

## Quick Start

```bash
# 1. Define your models
# 2. Generate migration
$ chakra migrate make

Detecting changes...
  + CreateModel: User (users)
  + CreateModel: Post (posts)

? Migration name: initial
✓ Created migrations/0001_initial.toml

# 3. Apply migration
$ chakra migrate apply

Applying migrations...
  → 0001_initial... done (23ms)

✓ Applied 1 migration
```

## Migration Workflow

```
┌─────────────────┐
│  Define Models  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ chakra migrate  │
│      make       │──────────────────┐
└────────┬────────┘                  │
         │                           │
         ▼                           ▼
┌─────────────────┐      ┌───────────────────────┐
│ Migration File  │      │ Schema Snapshot       │
│  (.toml)        │      │ (.chakra/snapshot)    │
└────────┬────────┘      └───────────────────────┘
         │
         ▼
┌─────────────────┐
│ chakra migrate  │
│     apply       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Database     │
│    Updated      │
└─────────────────┘
```

## Migration File Format

Migrations use TOML for readability and portability:

```toml
# migrations/0001_initial.toml

[migration]
id = "0001_initial"
app = "default"
created_at = "2024-06-15T10:30:00Z"
dependencies = []
description = "Create initial user and post tables"

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

[[operations.fields]]
name = "email"
type = "String"
max_length = 255

[[operations.fields]]
name = "is_active"
type = "Boolean"
default = true

[[operations.fields]]
name = "created_at"
type = "DateTime"
default = "now()"
```

## Supported Operations

### Model Operations

| Operation | Description | Reversible |
|-----------|-------------|:----------:|
| `CreateModel` | Create a new table | ✅ |
| `DeleteModel` | Drop a table | ⚠️ |
| `RenameModel` | Rename a table | ✅ |

### Field Operations

| Operation | Description | Reversible |
|-----------|-------------|:----------:|
| `AddField` | Add a column | ✅ |
| `RemoveField` | Drop a column | ⚠️ |
| `AlterField` | Modify column type/constraints | ⚠️ |
| `RenameField` | Rename a column | ✅ |

### Index Operations

| Operation | Description | Reversible |
|-----------|-------------|:----------:|
| `CreateIndex` | Create an index | ✅ |
| `DropIndex` | Drop an index | ✅ |
| `RenameIndex` | Rename an index | ✅ |

### Constraint Operations

| Operation | Description | Reversible |
|-----------|-------------|:----------:|
| `AddConstraint` | Add CHECK/UNIQUE constraint | ✅ |
| `RemoveConstraint` | Remove constraint | ✅ |

### Raw SQL

| Operation | Description | Reversible |
|-----------|-------------|:----------:|
| `RunSQL` | Execute raw SQL | Depends |

## Change Detection

Chakra compares your current models against a schema snapshot:

```python
# Before: User model
class User(Model):
    id = Field(Integer, primary_key=True)
    name = Field(String(100))

# After: Add email field
class User(Model):
    id = Field(Integer, primary_key=True)
    name = Field(String(100))
    email = Field(String(255), nullable=True)  # NEW
```

Running `chakra migrate make`:

```toml
# Generated migration
[[operations]]
type = "AddField"
model = "User"
table = "users"
name = "email"
field_type = "String"
max_length = 255
nullable = true
```

## CLI Commands

### Generate Migration

```bash
# Auto-detect changes
$ chakra migrate make

# With custom name
$ chakra migrate make --name add_user_email

# Empty migration (for manual SQL)
$ chakra migrate make --empty --name custom_function
```

### Check Status

```bash
$ chakra migrate status

Migration Status:
  ✓ 0001_initial (applied 2024-06-15)
  ✓ 0002_add_profile (applied 2024-06-16)
  ○ 0003_add_settings (pending)
```

### Apply Migrations

```bash
# Apply all pending
$ chakra migrate apply

# Apply specific migration
$ chakra migrate apply 0003_add_settings

# Dry run (show SQL without executing)
$ chakra migrate apply --dry-run
```

### Rollback

```bash
# Rollback last migration
$ chakra migrate rollback

# Rollback specific count
$ chakra migrate rollback --count 2

# Rollback to specific migration
$ chakra migrate rollback --to 0001_initial
```

### Show SQL

```bash
$ chakra migrate sql 0002_add_profile

-- Forward
ALTER TABLE users ADD COLUMN bio TEXT;

-- Reverse
ALTER TABLE users DROP COLUMN bio;
```

## Raw SQL Migrations

For complex operations, use raw SQL:

```toml
# migrations/0005_create_view.toml

[migration]
id = "0005_create_view"
app = "analytics"
dependencies = ["0004_add_metrics"]

[[operations]]
type = "RunSQL"
reversible = true

forward = """
CREATE MATERIALIZED VIEW user_stats AS
SELECT
    user_id,
    COUNT(*) as post_count,
    SUM(view_count) as total_views
FROM posts
GROUP BY user_id;

CREATE UNIQUE INDEX idx_user_stats_user ON user_stats(user_id);
"""

reverse = """
DROP MATERIALIZED VIEW IF EXISTS user_stats;
"""
```

## Data Migrations

For data transformations:

=== "Python"

    ```python
    # migrations/0006_populate_slugs.py
    from chakra.migrate import Migration, RunPython

    def populate_slugs(session):
        posts = session.execute("SELECT id, title FROM posts WHERE slug IS NULL")
        for post in posts:
            slug = slugify(post.title)
            session.execute(
                "UPDATE posts SET slug = %s WHERE id = %s",
                [slug, post.id]
            )

    class Migration(Migration):
        dependencies = ["0005_add_slug_field"]
        operations = [
            RunPython(populate_slugs),
        ]
    ```

=== "TOML + SQL"

    ```toml
    [[operations]]
    type = "RunSQL"
    forward = """
    UPDATE posts
    SET slug = LOWER(REPLACE(title, ' ', '-'))
    WHERE slug IS NULL;
    """
    reverse = """
    UPDATE posts SET slug = NULL;
    """
    ```

## Migration Dependencies

Migrations can depend on others:

```toml
[migration]
id = "0003_add_profile"
dependencies = [
    "users/0002_add_bio",      # Same app
    "auth/0001_initial",        # Different app
]
```

## History Table

Chakra tracks applied migrations in the database:

```sql
CREATE TABLE _chakra_migrations (
    id VARCHAR(255) PRIMARY KEY,
    app VARCHAR(100) NOT NULL,
    applied_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    checksum VARCHAR(64),
    execution_time_ms INTEGER
);
```

## Best Practices

!!! tip "Migration Best Practices"

    1. **Commit migrations** — Always version control migration files
    2. **One change per migration** — Keep migrations focused
    3. **Test rollbacks** — Verify rollback works before deploying
    4. **Review SQL** — Use `--dry-run` to review generated SQL
    5. **Backup first** — Always backup before applying to production
    6. **Avoid data loss** — Be careful with `RemoveField` and `DeleteModel`

## Comparison with Other Tools

| Feature | Chakra | Django | Alembic | Prisma |
|---------|:------:|:------:|:-------:|:------:|
| Auto-detection | ✅ | ✅ | ❌ | ✅ |
| Framework-agnostic | ✅ | ❌ | ✅ | ✅ |
| Reversible ops | ✅ | ✅ | ✅ | ❌ |
| Raw SQL | ✅ | ✅ | ✅ | ✅ |
| Data migrations | ✅ | ✅ | ✅ | ❌ |
| Multi-database | ✅ | ✅ | ✅ | ✅ |
| File format | TOML | Python | Python | Prisma |
