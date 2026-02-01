---
title: Usage
description: Practical examples and patterns for Chakra ORM
---

# Usage

Real-world examples, design patterns, and best practices.

## Examples

Complete example applications demonstrating Chakra ORM in action.

<div class="grid cards" markdown>

-   **[Blog Application](examples/blog.md)**

    A full-featured blog with users, posts, comments, and tags.

-   **[E-Commerce](examples/ecommerce.md)**

    Products, orders, customers, and inventory management.

-   **[Multi-tenant SaaS](examples/multitenant.md)**

    Tenant isolation, shared schema, and per-tenant customization.

-   **[Real-time App](examples/realtime.md)**

    WebSocket integration with live updates.

</div>

## Design Patterns

Common patterns for structuring your database layer.

<div class="grid cards" markdown>

-   **[Repository Pattern](patterns/repository.md)**

    Abstract database operations behind a clean interface.

-   **[Active Record](patterns/active-record.md)**

    Models with built-in persistence methods.

-   **[CQRS](patterns/cqrs.md)**

    Separate read and write models.

-   **[Soft Deletes](patterns/soft-deletes.md)**

    Mark records as deleted without removing data.

-   **[Audit Logging](patterns/audit.md)**

    Track all changes to your data.

</div>

## Cookbook

Solutions to common problems.

<div class="grid cards" markdown>

-   **[Bulk Operations](cookbook/bulk-operations.md)**

    Efficient bulk insert, update, and delete.

-   **[Complex Queries](cookbook/complex-queries.md)**

    CTEs, subqueries, and window functions.

-   **[Performance Tips](cookbook/performance.md)**

    Optimize your database operations.

-   **[Testing](cookbook/testing.md)**

    Test your database code effectively.

</div>

---

## Quick Recipes

### Pagination

```python
from chakra import Paginator

paginator = Paginator(User.objects.all(), page_size=20)

page1 = await paginator.page(1)
print(f"Page 1 of {paginator.num_pages}")
for user in page1:
    print(user.username)
```

### Soft Deletes

```python
class SoftDeleteMixin(Model):
    class Meta:
        abstract = True

    deleted_at = Field(DateTime, nullable=True)

    async def soft_delete(self):
        self.deleted_at = datetime.utcnow()
        await self.save()

    @classmethod
    def active(cls):
        return cls.objects.filter(deleted_at__isnull=True)

class User(SoftDeleteMixin):
    __tablename__ = "users"
    # ...

# Usage
await user.soft_delete()
active_users = await User.active().all()
```

### Audit Trail

```python
class AuditMixin(Model):
    class Meta:
        abstract = True

    created_at = Field(DateTime, default=datetime.utcnow)
    created_by = Field(Integer, nullable=True)
    updated_at = Field(DateTime, default=datetime.utcnow, on_update=datetime.utcnow)
    updated_by = Field(Integer, nullable=True)
```

### Full-Text Search (PostgreSQL)

```python
from chakra.postgres import TSVector, to_tsvector, plainto_tsquery

class Article(Model):
    title = Field(String(200))
    content = Field(Text)
    search_vector = Field(TSVector)

# Search
articles = await Article.objects.filter(
    search_vector__matches=plainto_tsquery("python orm")
).order_by("-rank").all()
```

### Optimistic Locking

```python
class VersionedModel(Model):
    class Meta:
        abstract = True

    version = Field(Integer, default=1)

    async def save_versioned(self, session):
        old_version = self.version
        self.version += 1

        result = await type(self).objects.filter(
            id=self.id,
            version=old_version
        ).update(
            **self.to_dict(),
            version=self.version
        )

        if result == 0:
            raise OptimisticLockError("Record was modified by another process")
```
