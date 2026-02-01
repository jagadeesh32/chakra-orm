---
title: Transactions
description: Transaction management in Chakra ORM
---

# Transactions

Chakra ORM provides robust transaction support for data integrity.

## Basic Transactions

=== "Python"

    ```python
    from chakra import Session

    async with Session() as session:
        async with session.begin():
            # All operations in this block are in a transaction
            user = User(username="alice", email="alice@example.com")
            session.add(user)

            post = Post(title="Hello", author_id=user.id)
            session.add(post)

            # Commits automatically on success
        # Rolls back automatically on exception
    ```

=== "Rust"

    ```rust
    let session = Session::new(&pool);

    session.transaction(|tx| async move {
        let user = User { username: "alice".into(), .. };
        tx.add(&user);

        let post = Post { title: "Hello".into(), author_id: user.id, .. };
        tx.add(&post);

        Ok(())
    }).await?;
    ```

## Explicit Commit/Rollback

```python
async with Session() as session:
    try:
        user = User(username="alice")
        session.add(user)

        # Explicit commit
        await session.commit()

    except Exception:
        # Explicit rollback
        await session.rollback()
        raise
```

## Nested Transactions (Savepoints)

```python
async with Session() as session:
    async with session.begin():
        user = User(username="outer")
        session.add(user)

        try:
            async with session.begin_nested():  # Savepoint
                invalid = User(username="inner")
                session.add(invalid)
                raise ValueError("Rollback inner only")

        except ValueError:
            pass  # Savepoint rolled back

        # Outer transaction continues
        await session.commit()  # outer user is committed
```

## Isolation Levels

```python
from chakra import IsolationLevel

async with Session() as session:
    async with session.begin(isolation_level=IsolationLevel.SERIALIZABLE):
        # Highest isolation level
        pass
```

| Level | Description |
|-------|-------------|
| `READ_UNCOMMITTED` | Lowest isolation |
| `READ_COMMITTED` | Default for most databases |
| `REPEATABLE_READ` | Consistent reads within transaction |
| `SERIALIZABLE` | Highest isolation, full ACID |

## Transaction Patterns

### Transfer Pattern

```python
async def transfer(from_id: int, to_id: int, amount: Decimal):
    async with Session() as session:
        async with session.begin():
            from_account = await session.get(Account, from_id, for_update=True)
            to_account = await session.get(Account, to_id, for_update=True)

            if from_account.balance < amount:
                raise InsufficientFunds()

            from_account.balance -= amount
            to_account.balance += amount
            # Commits on exit
```

### Retry on Conflict

```python
from chakra.exceptions import TransactionConflict

async def update_with_retry(user_id: int, max_retries: int = 3):
    for attempt in range(max_retries):
        try:
            async with Session() as session:
                async with session.begin(isolation=IsolationLevel.SERIALIZABLE):
                    user = await session.get(User, user_id)
                    user.login_count += 1
                    return user

        except TransactionConflict:
            if attempt == max_retries - 1:
                raise
            await asyncio.sleep(0.1 * (2 ** attempt))
```
