---
title: Async Native
description: First-class async/await support in Chakra ORM
tags:
  - async
  - concurrency
  - performance
---

# Async Native

Chakra ORM is built async-first. The entire core is asynchronous, with sync APIs provided as convenient wrappers.

## Design Philosophy

> "Async is the truth. Sync is a convenience wrapper."

```
┌─────────────────────────────────────────┐
│         Your Application Code           │
│   async def / def (your choice)         │
├─────────────────────────────────────────┤
│         Chakra API Layer                │
│   await session.get() / session.get_sync()  │
├─────────────────────────────────────────┤
│         Rust Async Core                 │
│   Always async (Tokio runtime)          │
├─────────────────────────────────────────┤
│         Database Driver                 │
│   Async I/O (tokio-postgres, etc.)      │
└─────────────────────────────────────────┘
```

## Python Async API

### Basic Usage

```python
import asyncio
from chakra import Session, Model, Field, String, Integer

class User(Model):
    __tablename__ = "users"
    id = Field(Integer, primary_key=True)
    name = Field(String(100))

async def main():
    async with Session() as session:
        # All database operations are async
        user = User(name="Alice")
        session.add(user)
        await session.commit()

        # Queries are async
        users = await User.objects.filter(name__startswith="A").all()

        for user in users:
            print(user.name)

asyncio.run(main())
```

### Async Context Managers

```python
# Session
async with Session() as session:
    # Auto-commits on success, rolls back on exception
    pass

# Transaction
async with session.begin():
    # Explicit transaction block
    pass

# Connection
async with chakra.connect(url) as conn:
    # Direct connection (bypasses pool)
    pass
```

### Async Iteration

```python
# Stream large result sets
async for user in User.objects.filter(is_active=True).stream():
    await process_user(user)

# With batching
async for batch in User.objects.all().stream(batch_size=1000):
    await process_batch(batch)
```

### Concurrent Queries

```python
import asyncio

async def get_dashboard_data(user_id: int):
    # Run queries concurrently
    user, posts, notifications = await asyncio.gather(
        User.objects.get(id=user_id),
        Post.objects.filter(author_id=user_id).limit(10).all(),
        Notification.objects.filter(user_id=user_id, read=False).all(),
    )

    return {
        "user": user,
        "recent_posts": posts,
        "notifications": notifications,
    }
```

## Python Sync API

For sync contexts (scripts, Django, legacy code):

```python
from chakra import Session

def sync_example():
    with Session() as session:
        # Use _sync suffix for sync operations
        user = session.get_sync(User, 1)
        user.name = "Updated"
        session.commit_sync()

        # QuerySet sync methods
        users = User.objects.filter(is_active=True).all_sync()
        count = User.objects.count_sync()
```

### Mixing Async and Sync

```python
# In an async context, prefer async
async def async_view():
    user = await User.objects.get(id=1)  # ✅ Async

# In a sync context (Django, scripts)
def sync_view():
    user = User.objects.get_sync(id=1)  # ✅ Sync wrapper

# ❌ Don't call sync from async (blocks event loop)
async def bad_example():
    user = User.objects.get_sync(id=1)  # ⚠️ Warning in dev mode
```

## Rust Async API

### With Tokio

```rust
use chakra::prelude::*;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    let pool = chakra::connect("postgresql://localhost/db").await?;

    // All operations are async
    let users = User::query()
        .filter(User::is_active().eq(true))
        .all(&pool)
        .await?;

    for user in users {
        println!("{}", user.name);
    }

    Ok(())
}
```

### Concurrent Operations

```rust
use futures::future::try_join_all;

async fn get_users_with_posts(ids: Vec<i64>, pool: &Pool) -> Result<Vec<UserWithPosts>> {
    let futures: Vec<_> = ids.iter().map(|id| async {
        let user = User::get(pool, *id).await?;
        let posts = Post::query()
            .filter(Post::author_id().eq(*id))
            .all(pool)
            .await?;

        Ok(UserWithPosts { user, posts })
    }).collect();

    try_join_all(futures).await
}
```

### Streaming Results

```rust
use futures::StreamExt;

async fn process_all_users(pool: &Pool) -> Result<()> {
    let mut stream = User::query()
        .filter(User::is_active().eq(true))
        .stream(pool)
        .await?;

    while let Some(result) = stream.next().await {
        let user = result?;
        process_user(&user).await?;
    }

    Ok(())
}
```

## Framework Integration

### FastAPI (Async)

```python
from fastapi import FastAPI, Depends
from chakra import Session

app = FastAPI()

async def get_session():
    async with Session() as session:
        yield session

@app.get("/users/{user_id}")
async def get_user(user_id: int, session: Session = Depends(get_session)):
    user = await session.get(User, user_id)
    if not user:
        raise HTTPException(404)
    return user
```

### Django (Sync with Async Views)

```python
# Sync view
def user_list(request):
    users = User.objects.all_sync()
    return render(request, "users.html", {"users": users})

# Async view (Django 4.1+)
async def user_list_async(request):
    users = await User.objects.all()
    return render(request, "users.html", {"users": users})
```

### Axum (Rust Async)

```rust
use axum::{extract::State, routing::get, Router, Json};

async fn list_users(State(pool): State<Pool>) -> Json<Vec<User>> {
    let users = User::query()
        .filter(User::is_active().eq(true))
        .all(&pool)
        .await
        .unwrap();

    Json(users)
}

#[tokio::main]
async fn main() {
    let pool = chakra::connect("postgresql://localhost/db").await.unwrap();

    let app = Router::new()
        .route("/users", get(list_users))
        .with_state(pool);

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## Connection Pool & Async

The connection pool is designed for async:

```toml
# chakra.toml
[pool]
min_connections = 5      # Keep 5 connections ready
max_connections = 20     # Scale up to 20 under load
acquire_timeout = "30s"  # Wait up to 30s for a connection
idle_timeout = "10m"     # Close idle connections after 10m
```

```python
# Connection acquisition is async and non-blocking
async with Session() as session:
    # Acquires connection from pool (non-blocking)
    user = await session.get(User, 1)
# Connection returned to pool
```

## Performance Considerations

### GIL Release

Python's GIL is released during all I/O operations:

```python
# GIL released here - other Python threads can run
users = await User.objects.all()
```

### Batching

For many small queries, batch them:

```python
# ❌ Inefficient: 100 round trips
for id in user_ids:
    user = await User.objects.get(id=id)

# ✅ Efficient: 1 round trip
users = await User.objects.filter(id__in=user_ids).all()
```

### Connection Reuse

Sessions reuse connections within their scope:

```python
async with Session() as session:
    # All queries use the same connection
    user = await session.get(User, 1)
    posts = await Post.objects.filter(author_id=1).all()
    comments = await Comment.objects.filter(post_id__in=[p.id for p in posts]).all()
```
