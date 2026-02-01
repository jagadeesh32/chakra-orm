---
title: Roadmap
description: Upcoming features and future plans for Chakra ORM
---

# Roadmap

Our development roadmap for Chakra ORM.

!!! note "Disclaimer"
    This roadmap is subject to change based on community feedback and priorities.

---

## Current Focus: v0.2.0

**Target: Q3 2024**

### Planned Features

- [ ] **Oracle Database Support** — Full Oracle 19c+ support
- [ ] **Query Caching** — Cache query results with configurable TTL
- [ ] **Schema Introspection** — Generate models from existing database
- [ ] **Improved Error Messages** — Source locations and fix suggestions
- [ ] **VSCode Extension** — Model validation and query preview

---

## Future Releases

### v0.3.0 — Performance & Scale

- [ ] **Read Replicas** — Automatic routing of read queries
- [ ] **Sharding Support** — Horizontal database partitioning
- [ ] **Streaming Results** — Memory-efficient large result sets
- [ ] **Batch Processing** — Efficient batch insert/update APIs
- [ ] **Connection Pool Metrics** — Prometheus/OpenTelemetry integration

### v0.4.0 — Developer Experience

- [ ] **Prisma-style Studio** — Web-based database browser
- [ ] **Migration Squashing** — Combine migrations
- [ ] **Database Seeding** — Fixture loading framework
- [ ] **Factory Pattern** — Test data generation

### v0.5.0 — Enterprise Features

- [ ] **Multi-tenancy** — Built-in tenant isolation
- [ ] **Audit Logging** — Automatic change tracking
- [ ] **Row-Level Security** — PostgreSQL RLS integration
- [ ] **Encryption at Rest** — Column-level encryption

### v1.0.0 — Stable Release

- [ ] API stability guarantee
- [ ] Comprehensive documentation
- [ ] Performance benchmarks
- [ ] Security audit

---

## Community Requested

Features requested by the community (vote on GitHub):

| Feature | Votes | Status |
|---------|-------|--------|
| GraphQL Integration | 45 | Planned (v0.4) |
| MongoDB Support | 32 | Under consideration |
| TimescaleDB Support | 28 | Under consideration |
| GIS/PostGIS Support | 24 | Planned (v0.3) |
| Database Views | 21 | Planned (v0.2) |

---

## Contributing

Want to help shape the roadmap?

1. **Vote** on existing feature requests on [GitHub Issues](https://github.com/chakra-orm/chakra-orm/issues)
2. **Propose** new features via [GitHub Discussions](https://github.com/chakra-orm/chakra-orm/discussions)
3. **Contribute** code via [Pull Requests](https://github.com/chakra-orm/chakra-orm/pulls)

---

## Release Schedule

We aim for quarterly releases:

| Quarter | Version | Focus |
|---------|---------|-------|
| Q2 2024 | 0.1.0 | Initial release |
| Q3 2024 | 0.2.0 | Oracle, caching, introspection |
| Q4 2024 | 0.3.0 | Performance & scale |
| Q1 2025 | 0.4.0 | Developer experience |
| Q2 2025 | 0.5.0 | Enterprise features |
| Q3 2025 | 1.0.0 | Stable release |
