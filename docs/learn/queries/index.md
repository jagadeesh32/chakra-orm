---
title: Queries
description: Master the Chakra ORM query API
---

# Queries

Chakra ORM provides a powerful, intuitive query API for both Python and Rust.

## In This Section

- [Basic Queries](basic.md) — CRUD operations and simple queries
- [Filtering](filtering.md) — Filter expressions and lookups
- [Ordering & Pagination](ordering.md) — Sort and paginate results
- [Aggregation](aggregation.md) — COUNT, SUM, AVG, and more
- [Joins & Relationships](joins.md) — Query across relationships
- [Raw SQL](raw-sql.md) — Execute raw SQL when needed

## Quick Reference

=== "Python"

    ```python
    # All records
    users = await User.objects.all()

    # Filter
    users = await User.objects.filter(is_active=True).all()

    # Get single record
    user = await User.objects.get(id=1)

    # First/Last
    user = await User.objects.order_by("created_at").first()

    # Count
    count = await User.objects.filter(is_active=True).count()

    # Exists
    exists = await User.objects.filter(email="test@example.com").exists()

    # Complex filter
    from chakra import Q
    users = await User.objects.filter(
        Q(age__gte=18) & Q(is_active=True)
    ).all()

    # Order and limit
    users = await User.objects.order_by("-created_at").limit(10).all()

    # With relationships
    users = await User.objects.prefetch_related("posts").all()
    ```

=== "Rust"

    ```rust
    // All records
    let users = User::query().all(&pool).await?;

    // Filter
    let users = User::query()
        .filter(User::is_active().eq(true))
        .all(&pool)
        .await?;

    // Get single record
    let user = User::get(&pool, 1).await?;

    // First
    let user = User::query()
        .order_by(User::created_at().asc())
        .first(&pool)
        .await?;

    // Count
    let count = User::query()
        .filter(User::is_active().eq(true))
        .count(&pool)
        .await?;

    // Complex filter
    let users = User::query()
        .filter(
            User::age().gte(18).and(User::is_active().eq(true))
        )
        .all(&pool)
        .await?;

    // Order and limit
    let users = User::query()
        .order_by(User::created_at().desc())
        .limit(10)
        .all(&pool)
        .await?;
    ```

## Query Methods Overview

| Method | Description | Returns |
|--------|-------------|---------|
| `all()` | Get all matching records | `List[Model]` |
| `first()` | Get first record | `Optional[Model]` |
| `last()` | Get last record | `Optional[Model]` |
| `get(**kwargs)` | Get exactly one record | `Model` |
| `get_or_none(**kwargs)` | Get one or None | `Optional[Model]` |
| `count()` | Count matching records | `int` |
| `exists()` | Check if any match | `bool` |
| `filter(**kwargs)` | Filter records | `QuerySet` |
| `exclude(**kwargs)` | Exclude records | `QuerySet` |
| `order_by(*fields)` | Order results | `QuerySet` |
| `limit(n)` | Limit results | `QuerySet` |
| `offset(n)` | Skip results | `QuerySet` |
| `select_related(*rels)` | Eager load (JOIN) | `QuerySet` |
| `prefetch_related(*rels)` | Eager load (queries) | `QuerySet` |
| `only(*fields)` | Select specific fields | `QuerySet` |
| `defer(*fields)` | Exclude specific fields | `QuerySet` |
| `distinct()` | Unique results | `QuerySet` |
| `values(*fields)` | Return dicts | `QuerySet` |
| `values_list(*fields)` | Return tuples | `QuerySet` |
