---
title: Sessions
description: Manage database sessions and transactions
---

# Sessions

Sessions manage database connections, transactions, and object lifecycle.

## In This Section

- [Unit of Work](unit-of-work.md) — How sessions track and persist changes
- [Transactions](transactions.md) — Transaction management and isolation
- [Identity Map](identity-map.md) — Object caching and identity

## Overview

The Session is the gateway to the database. It:

1. **Manages connections** — Acquires and releases database connections
2. **Tracks changes** — Knows which objects are new, modified, or deleted
3. **Handles transactions** — Begins, commits, and rolls back transactions
4. **Caches objects** — Maintains an identity map of loaded objects

=== "Python"

    ```python
    from chakra import Session

    async with Session() as session:
        # Load object
        user = await session.get(User, 1)

        # Modify
        user.name = "Updated"

        # Create new
        post = Post(title="Hello", author_id=user.id)
        session.add(post)

        # Commit all changes
        await session.commit()
    ```

=== "Rust"

    ```rust
    let session = Session::new(&pool);

    let mut user = session.get::<User>(1).await?.unwrap();
    user.name = "Updated".into();

    let post = Post { title: "Hello".into(), author_id: user.id, .. };
    session.add(&post);

    session.commit().await?;
    ```
