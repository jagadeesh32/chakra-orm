---
title: Filtering
description: Filter queries using expressions and lookups
---

# Filtering

Chakra ORM provides a powerful filtering system with Django-style lookups.

## Basic Filtering

=== "Python"

    ```python
    # Simple equality
    users = await User.objects.filter(username="alice").all()

    # Multiple conditions (AND)
    users = await User.objects.filter(
        is_active=True,
        age=25
    ).all()
    ```

=== "Rust"

    ```rust
    let users = User::query()
        .filter(User::username().eq("alice"))
        .all(&pool)
        .await?;

    let users = User::query()
        .filter(User::is_active().eq(true))
        .filter(User::age().eq(25))
        .all(&pool)
        .await?;
    ```

## Lookup Expressions

### Comparison

| Lookup | SQL | Example |
|--------|-----|---------|
| `exact` / `eq` | `=` | `username__exact="alice"` |
| `ne` | `!=` | `status__ne="deleted"` |
| `gt` | `>` | `age__gt=18` |
| `gte` | `>=` | `age__gte=21` |
| `lt` | `<` | `age__lt=65` |
| `lte` | `<=` | `age__lte=60` |

```python
users = await User.objects.filter(
    age__gte=18,
    age__lte=65
).all()
```

### String Matching

| Lookup | SQL | Example |
|--------|-----|---------|
| `contains` | `LIKE '%x%'` | `name__contains="john"` |
| `icontains` | `ILIKE '%x%'` | `name__icontains="JOHN"` |
| `startswith` | `LIKE 'x%'` | `email__startswith="admin"` |
| `istartswith` | `ILIKE 'x%'` | `email__istartswith="ADMIN"` |
| `endswith` | `LIKE '%x'` | `email__endswith=".com"` |
| `iendswith` | `ILIKE '%x'` | `email__iendswith=".COM"` |
| `regex` | `~` | `name__regex=r"^[A-Z]"` |
| `iregex` | `~*` | `name__iregex=r"^[a-z]"` |

```python
users = await User.objects.filter(
    email__endswith="@gmail.com"
).all()
```

### Null Checking

| Lookup | SQL | Example |
|--------|-----|---------|
| `isnull` | `IS NULL` / `IS NOT NULL` | `bio__isnull=True` |

```python
# Has no bio
users = await User.objects.filter(bio__isnull=True).all()

# Has bio
users = await User.objects.filter(bio__isnull=False).all()
```

### List Membership

| Lookup | SQL | Example |
|--------|-----|---------|
| `in` | `IN (...)` | `status__in=["active", "pending"]` |

```python
users = await User.objects.filter(
    id__in=[1, 2, 3, 4, 5]
).all()
```

### Range

| Lookup | SQL | Example |
|--------|-----|---------|
| `range` | `BETWEEN` | `age__range=(18, 65)` |

```python
users = await User.objects.filter(
    created_at__range=(start_date, end_date)
).all()
```

### Date/Time

| Lookup | SQL | Example |
|--------|-----|---------|
| `year` | `EXTRACT(YEAR ...)` | `created_at__year=2024` |
| `month` | `EXTRACT(MONTH ...)` | `created_at__month=6` |
| `day` | `EXTRACT(DAY ...)` | `created_at__day=15` |
| `date` | `DATE(...)` | `created_at__date="2024-06-15"` |

```python
# Posts from June 2024
posts = await Post.objects.filter(
    created_at__year=2024,
    created_at__month=6
).all()
```

## Complex Queries with Q Objects

### OR Conditions

```python
from chakra import Q

# Users who are either admin OR active
users = await User.objects.filter(
    Q(is_admin=True) | Q(is_active=True)
).all()
```

### AND Conditions

```python
# Explicit AND (same as multiple filter kwargs)
users = await User.objects.filter(
    Q(is_active=True) & Q(age__gte=18)
).all()
```

### NOT Conditions

```python
# Users who are NOT deleted
users = await User.objects.filter(
    ~Q(status="deleted")
).all()
```

### Combined Expressions

```python
# Complex logic
users = await User.objects.filter(
    Q(is_admin=True) | (Q(is_active=True) & Q(age__gte=21))
).all()

# With exclude
users = await User.objects.filter(
    Q(is_active=True)
).exclude(
    Q(status="banned") | Q(email__endswith="@spam.com")
).all()
```

## Rust Filter Expressions

```rust
// Basic
let users = User::query()
    .filter(User::is_active().eq(true))
    .all(&pool)
    .await?;

// String operations
let users = User::query()
    .filter(User::email().ends_with("@gmail.com"))
    .all(&pool)
    .await?;

// OR conditions
let users = User::query()
    .filter(
        User::is_admin().eq(true)
            .or(User::is_active().eq(true))
    )
    .all(&pool)
    .await?;

// Complex
let users = User::query()
    .filter(
        User::is_admin().eq(true)
            .or(
                User::is_active().eq(true)
                    .and(User::age().gte(21))
            )
    )
    .all(&pool)
    .await?;
```

## Exclude

Opposite of filter — exclude matching records.

```python
# All users except deleted ones
users = await User.objects.exclude(status="deleted").all()

# Combine with filter
users = await User.objects.filter(
    is_active=True
).exclude(
    email__endswith="@spam.com"
).all()
```

## Filter Across Relationships

```python
# Posts by user named "alice"
posts = await Post.objects.filter(
    author__username="alice"
).all()

# Posts by active users
posts = await Post.objects.filter(
    author__is_active=True
).all()

# Users with published posts
users = await User.objects.filter(
    posts__published=True
).distinct().all()

# Deep traversal
comments = await Comment.objects.filter(
    post__author__username="alice"
).all()
```
