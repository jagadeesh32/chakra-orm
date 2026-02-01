---
title: CLI Commands
description: Complete reference for Chakra ORM CLI commands
---

# CLI Commands

The Chakra CLI provides commands for migrations, database management, and code generation.

## Installation

```bash
# Python (included with chakra-orm)
pip install chakra-orm

# Rust
cargo install chakra-cli
```

## Global Options

```bash
chakra [OPTIONS] <COMMAND>

Options:
  -c, --config <FILE>    Configuration file [default: chakra.toml]
  -v, --verbose          Increase verbosity (-v, -vv, -vvv)
  -q, --quiet            Suppress output
  --version              Print version
  -h, --help             Print help
```

## Commands

### `chakra init`

Initialize Chakra in a project.

```bash
chakra init [OPTIONS]

Options:
  --python          Initialize for Python
  --rust            Initialize for Rust
  --database <DB>   Database type [postgres, mysql, sqlite]
  --example         Create example models
```

**Example:**
```bash
chakra init --python --database postgres
```

---

### `chakra migrate`

Migration management commands.

#### `chakra migrate make`

Create a new migration.

```bash
chakra migrate make [OPTIONS]

Options:
  -n, --name <NAME>    Migration name
  --empty              Create empty migration
  --app <APP>          Application name
```

**Examples:**
```bash
# Auto-detect changes
chakra migrate make

# With custom name
chakra migrate make --name add_user_email

# Empty migration for raw SQL
chakra migrate make --empty --name create_custom_function
```

#### `chakra migrate apply`

Apply pending migrations.

```bash
chakra migrate apply [OPTIONS] [MIGRATION]

Options:
  --dry-run            Show SQL without executing
  --fake               Mark as applied without executing
  --database <URL>     Override database URL

Arguments:
  [MIGRATION]          Specific migration to apply
```

**Examples:**
```bash
# Apply all pending
chakra migrate apply

# Dry run
chakra migrate apply --dry-run

# Apply specific migration
chakra migrate apply 0003_add_email
```

#### `chakra migrate rollback`

Rollback migrations.

```bash
chakra migrate rollback [OPTIONS]

Options:
  --count <N>          Number of migrations to rollback [default: 1]
  --to <MIGRATION>     Rollback to specific migration
  --dry-run            Show SQL without executing
```

**Examples:**
```bash
# Rollback last migration
chakra migrate rollback

# Rollback last 3
chakra migrate rollback --count 3

# Rollback to specific
chakra migrate rollback --to 0001_initial
```

#### `chakra migrate status`

Show migration status.

```bash
chakra migrate status

# Output:
# Migration Status:
#   ✓ 0001_initial (applied 2024-06-15)
#   ✓ 0002_add_email (applied 2024-06-16)
#   ○ 0003_add_profile (pending)
```

#### `chakra migrate sql`

Show SQL for a migration.

```bash
chakra migrate sql <MIGRATION>

Options:
  --forward            Show forward SQL (default)
  --reverse            Show reverse SQL
```

---

### `chakra db`

Database management commands.

#### `chakra db check`

Check database connection.

```bash
chakra db check

# Output:
# Connecting to postgresql://localhost:5432/mydb...
# ✓ Connection successful
#   PostgreSQL 15.2
#   Schema: public
#   Tables: 12
```

#### `chakra db schema`

Show database schema.

```bash
chakra db schema [TABLE]

Options:
  --detailed           Show full details
```

#### `chakra db pull`

Generate models from existing database.

```bash
chakra db pull [OPTIONS]

Options:
  --output <DIR>       Output directory [default: models]
  --tables <TABLES>    Specific tables (comma-separated)
```

#### `chakra db reset`

Reset database (drop all, re-migrate).

```bash
chakra db reset

# ⚠ WARNING: This will DROP ALL TABLES
# Type 'RESET' to confirm: RESET
```

#### `chakra db create`

Create the database.

```bash
chakra db create
```

#### `chakra db drop`

Drop the database.

```bash
chakra db drop

# ⚠ WARNING: This will permanently delete the database
```

---

### `chakra generate`

Code generation commands.

#### `chakra generate model`

Generate model from table.

```bash
chakra generate model <TABLE>

Options:
  --output <FILE>      Output file
```

#### `chakra generate crud`

Generate CRUD handlers.

```bash
chakra generate crud <MODEL>

Options:
  --framework <FW>     Framework [fastapi, axum, actix]
  --output <FILE>      Output file
```

---

### `chakra shell`

Interactive database shell.

```bash
chakra shell

# Commands in shell:
# .tables          List tables
# .schema <table>  Show table schema
# .sql <query>     Execute raw SQL
# exit             Exit shell
```

---

### `chakra check`

Validate models and configuration.

```bash
chakra check

# Output:
# Checking models...
#   ✓ User (users)
#   ✓ Post (posts)
# Checking configuration...
#   ✓ chakra.toml valid
# Checking database connection...
#   ✓ Connected to PostgreSQL 15.2
#
# All checks passed!
```
