---
title: Migrations
description: Manage database schema changes with Chakra migrations
---

# Migrations

Chakra ORM includes a powerful migration system that auto-detects model changes.

## In This Section

- [Creating Migrations](creating.md) — Generate migrations from model changes
- [Applying Migrations](applying.md) — Apply pending migrations
- [Rollback](rollback.md) — Undo migrations
- [Advanced](advanced.md) — Raw SQL, data migrations, and more

## Quick Start

```bash
# 1. Make changes to your models

# 2. Generate migration
chakra migrate make --name add_user_email

# 3. Review generated migration
cat migrations/0002_add_user_email.toml

# 4. Apply migration
chakra migrate apply
```

## How It Works

1. **Define models** in Python or Rust
2. **Run `chakra migrate make`** — Chakra compares your models to the last known schema
3. **Migration file generated** — TOML file with operations
4. **Run `chakra migrate apply`** — Executes SQL against your database
5. **History tracked** — Applied migrations recorded in `_chakra_migrations` table

## Migration File Example

```toml
# migrations/0002_add_user_email.toml

[migration]
id = "0002_add_user_email"
app = "default"
created_at = "2024-06-15T10:30:00Z"
dependencies = ["0001_initial"]
description = "Add email field to users"

[[operations]]
type = "AddField"
model = "User"
table = "users"
name = "email"
field_type = "String"
max_length = 255
unique = true
nullable = false
default = "'unknown@example.com'"
```

## Commands Overview

| Command | Description |
|---------|-------------|
| `chakra migrate make` | Create new migration |
| `chakra migrate apply` | Apply pending migrations |
| `chakra migrate rollback` | Rollback migrations |
| `chakra migrate status` | Show migration status |
| `chakra migrate sql <id>` | Show migration SQL |
