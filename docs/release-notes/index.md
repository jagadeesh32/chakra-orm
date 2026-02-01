---
title: Release Notes
description: Chakra ORM release notes and changelog
---

# Release Notes

Stay up to date with Chakra ORM releases.

## Latest Release

### v0.1.0 (2024-06-15) — Initial Release

The first public release of Chakra ORM!

**Highlights:**

- Rust core with Python bindings via PyO3
- PostgreSQL, MySQL, SQLite support
- Full async/await support
- Django-style auto migrations
- Comprehensive query API
- Session with Unit of Work pattern
- CLI for migrations and database management

[Full Changelog](changelog.md#v010-2024-06-15) · [Migration Guide](migration-guide.md)

---

## Quick Links

- **[Full Changelog](changelog.md)** — Detailed version history
- **[Migration Guide](migration-guide.md)** — Upgrading between versions
- **[Roadmap](roadmap.md)** — Upcoming features

---

## Version Support

| Version | Python | Rust | Status |
|---------|--------|------|--------|
| 0.1.x | 3.9+ | 1.70+ | **Current** |

---

## Upgrade Notice

When upgrading, always:

1. Read the [Migration Guide](migration-guide.md) for your version
2. Backup your database before running migrations
3. Test in a staging environment first
