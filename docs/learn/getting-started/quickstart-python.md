---
title: Quick Start (Python)
description: Build your first Python application with Chakra ORM
---

# Quick Start (Python)

Build a complete CRUD application in under 5 minutes.

## Prerequisites

- Python 3.9+
- PostgreSQL (or SQLite for quick testing)
- `pip install chakra-orm[postgres]`

## 1. Initialize Project

```bash
mkdir blog-app && cd blog-app
chakra init
```

## 2. Configure Database

Edit `chakra.toml`:

```toml
[database]
url = "postgresql://postgres:password@localhost:5432/blog"
# Or for SQLite:
# url = "sqlite:///blog.db"

[pool]
min_connections = 2
max_connections = 10
```

## 3. Define Models

Create `models.py`:

```python
from chakra import Model, Field, String, Integer, Text, DateTime, Boolean
from chakra import ForeignKey, OneToMany
from datetime import datetime

class User(Model):
    __tablename__ = "users"

    id = Field(Integer, primary_key=True)
    username = Field(String(50), unique=True, index=True)
    email = Field(String(255), unique=True)
    is_active = Field(Boolean, default=True)
    created_at = Field(DateTime, default=datetime.utcnow)

    # Relationship
    posts = OneToMany("Post", back_populates="author")

    def __repr__(self):
        return f"<User {self.username}>"


class Post(Model):
    __tablename__ = "posts"

    id = Field(Integer, primary_key=True)
    title = Field(String(200))
    content = Field(Text)
    published = Field(Boolean, default=False)
    created_at = Field(DateTime, default=datetime.utcnow)

    # Foreign key
    author_id = Field(Integer, ForeignKey("users.id"))
    author = ForeignKey(User, back_populates="posts")

    def __repr__(self):
        return f"<Post {self.title}>"
```

## 4. Create Migration

```bash
chakra migrate make --name initial

# Output:
# Detecting changes...
#   + CreateModel: User (users)
#   + CreateModel: Post (posts)
# ✓ Created migrations/0001_initial.toml
```

## 5. Apply Migration

```bash
chakra migrate apply

# Output:
# Applying migrations...
#   → 0001_initial... done (45ms)
# ✓ Applied 1 migration
```

## 6. Use in Your Application

Create `app.py`:

```python
import asyncio
from chakra import Session
from models import User, Post

async def main():
    async with Session() as session:
        # ========================================
        # CREATE
        # ========================================
        print("Creating users...")

        alice = User(username="alice", email="alice@example.com")
        bob = User(username="bob", email="bob@example.com")

        session.add(alice)
        session.add(bob)
        await session.commit()

        print(f"Created: {alice} (id={alice.id})")
        print(f"Created: {bob} (id={bob.id})")

        # Create posts
        post1 = Post(
            title="Hello World",
            content="My first post!",
            author_id=alice.id,
            published=True
        )
        post2 = Post(
            title="Learning Chakra ORM",
            content="Chakra is amazing...",
            author_id=alice.id
        )

        session.add(post1)
        session.add(post2)
        await session.commit()

        # ========================================
        # READ
        # ========================================
        print("\nQuerying users...")

        # Get by ID
        user = await session.get(User, alice.id)
        print(f"Found: {user}")

        # Filter
        active_users = await User.objects.filter(is_active=True).all()
        print(f"Active users: {len(active_users)}")

        # Complex query
        users = await User.objects.filter(
            username__startswith="a",
            is_active=True
        ).order_by("-created_at").all()

        for user in users:
            print(f"  - {user.username}")

        # Query with relationships
        users_with_posts = await User.objects.prefetch_related("posts").all()
        for user in users_with_posts:
            print(f"{user.username} has {len(user.posts)} posts")

        # ========================================
        # UPDATE
        # ========================================
        print("\nUpdating...")

        alice.email = "alice.smith@example.com"
        await session.commit()
        print(f"Updated email: {alice.email}")

        # Bulk update
        count = await Post.objects.filter(author_id=alice.id).update(published=True)
        print(f"Published {count} posts")

        # ========================================
        # DELETE
        # ========================================
        print("\nDeleting...")

        # Delete bob
        session.delete(bob)
        await session.commit()
        print("Deleted bob")

        # Verify
        remaining = await User.objects.count()
        print(f"Remaining users: {remaining}")


if __name__ == "__main__":
    asyncio.run(main())
```

## 7. Run It

```bash
python app.py
```

Output:

```
Creating users...
Created: <User alice> (id=1)
Created: <User bob> (id=2)

Querying users...
Found: <User alice>
Active users: 2
  - alice

alice has 2 posts
bob has 0 posts

Updating...
Updated email: alice.smith@example.com
Published 2 posts

Deleting...
Deleted bob
Remaining users: 1
```

## Full Example: FastAPI Blog

```python
# main.py
from fastapi import FastAPI, Depends, HTTPException
from chakra import Session
from models import User, Post
from pydantic import BaseModel

app = FastAPI()

# Dependency
async def get_session():
    async with Session() as session:
        yield session

# Schemas
class UserCreate(BaseModel):
    username: str
    email: str

class PostCreate(BaseModel):
    title: str
    content: str
    author_id: int

# Routes
@app.post("/users")
async def create_user(user: UserCreate, session: Session = Depends(get_session)):
    db_user = User(**user.dict())
    session.add(db_user)
    await session.commit()
    return {"id": db_user.id, "username": db_user.username}

@app.get("/users")
async def list_users(session: Session = Depends(get_session)):
    users = await User.objects.all()
    return [{"id": u.id, "username": u.username} for u in users]

@app.get("/users/{user_id}")
async def get_user(user_id: int, session: Session = Depends(get_session)):
    user = await session.get(User, user_id)
    if not user:
        raise HTTPException(404, "User not found")
    return {"id": user.id, "username": user.username, "email": user.email}

@app.post("/posts")
async def create_post(post: PostCreate, session: Session = Depends(get_session)):
    db_post = Post(**post.dict())
    session.add(db_post)
    await session.commit()
    return {"id": db_post.id, "title": db_post.title}

@app.get("/posts")
async def list_posts(published: bool = None, session: Session = Depends(get_session)):
    query = Post.objects
    if published is not None:
        query = query.filter(published=published)
    posts = await query.order_by("-created_at").all()
    return [{"id": p.id, "title": p.title, "published": p.published} for p in posts]
```

Run with:

```bash
uvicorn main:app --reload
```

## Next Steps

- [Defining Models](../models/defining-models.md) — Learn more about model options
- [Querying](../queries/basic.md) — Master the query API
- [Sessions](../sessions/index.md) — Understand transactions and unit of work
- [FastAPI Integration](../integrations/fastapi.md) — Complete FastAPI guide
