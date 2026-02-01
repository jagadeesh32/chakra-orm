---
title: Changelog
description: Complete version history for Chakra ORM
---

# Changelog

All notable changes to Chakra ORM are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/), and this project adheres to [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Added
- Oracle database support
- Query result caching
- Schema introspection from existing databases

### Changed
- Improved error messages with source locations

### Fixed
- Connection pool timeout handling edge cases

---

## [0.1.0] - 2024-06-15

### Added

#### Core
- **Rust Core Engine** — Query building, SQL generation, and result decoding in Rust
- **Python Bindings** — First-class Python support via PyO3
- **Async/Await** — Native async support for both Python and Rust
- **Sync API** — Convenience sync wrappers for non-async contexts

#### Models
- `Model` base class (Python) and `#[derive(Model)]` macro (Rust)
- All standard field types: `Integer`, `String`, `Text`, `Boolean`, `DateTime`, `Date`, `Time`, `Decimal`, `Float`, `UUID`, `JSON`, `Binary`
- Field options: `primary_key`, `unique`, `index`, `nullable`, `default`, `max_length`
- Relationship types: `OneToOne`, `OneToMany`, `ManyToMany`
- Model inheritance and abstract base classes

#### Queries
- Fluent query builder API
- Django-style lookups: `__exact`, `__gt`, `__gte`, `__lt`, `__lte`, `__in`, `__contains`, `__startswith`, `__endswith`, `__isnull`
- `Q` objects for complex queries (AND, OR, NOT)
- `F` expressions for field references
- Aggregations: `Count`, `Sum`, `Avg`, `Max`, `Min`
- `select_related` (JOIN) and `prefetch_related` (separate queries)
- `order_by`, `limit`, `offset`
- `values` and `values_list` for partial results
- `distinct` for unique results

#### Sessions
- Session with Unit of Work pattern
- Automatic dirty tracking
- Identity map for object caching
- Transaction support with `begin()`, `commit()`, `rollback()`
- Savepoints for nested transactions
- Isolation level configuration

#### Migrations
- Auto-detection of model changes
- TOML-based migration files
- Migration operations: `CreateModel`, `DeleteModel`, `AddField`, `RemoveField`, `AlterField`, `RenameField`, `CreateIndex`, `DropIndex`
- `RunSQL` for raw SQL migrations
- Migration dependencies
- Rollback support
- Migration history tracking

#### Connection Pool
- Async connection pool (deadpool-based)
- Configurable min/max connections
- Idle timeout and max lifetime
- Health checks

#### CLI
- `chakra init` — Project initialization
- `chakra migrate make` — Create migrations
- `chakra migrate apply` — Apply migrations
- `chakra migrate rollback` — Rollback migrations
- `chakra migrate status` — Show migration status
- `chakra db check` — Check database connection
- `chakra db schema` — Show database schema
- `chakra shell` — Interactive shell

#### Database Support
- **PostgreSQL** — Full support including JSONB, arrays, and advanced types
- **MySQL / MariaDB** — Full support
- **SQLite** — Full support including in-memory databases

### Security
- Parameterized queries to prevent SQL injection
- Secure default connection settings

---

## Version History

| Version | Date | Highlights |
|---------|------|------------|
| [0.1.0](#010---2024-06-15) | 2024-06-15 | Initial release |

---

## Upgrade Path

See the [Migration Guide](migration-guide.md) for detailed upgrade instructions between versions.
