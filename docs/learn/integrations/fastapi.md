---
title: FastAPI Integration
description: Use Chakra ORM with FastAPI
---

# FastAPI Integration

FastAPI and Chakra ORM are a perfect match — both are async-native and type-safe.

## Setup

### Installation

```bash
pip install chakra-orm[postgres] fastapi uvicorn
```

### Project Structure

```
my_app/
├── app/
│   ├── __init__.py
│   ├── main.py
│   ├── models.py
│   ├── schemas.py
│   ├── database.py
│   └── routers/
│       └── users.py
├── migrations/
├── chakra.toml
└── requirements.txt
```

## Database Configuration

```python
# app/database.py
from chakra import Session, create_pool
from contextlib import asynccontextmanager

DATABASE_URL = "postgresql://user:pass@localhost:5432/mydb"

pool = None

async def init_db():
    global pool
    pool = await create_pool(DATABASE_URL)

async def close_db():
    global pool
    if pool:
        await pool.close()

async def get_session():
    async with Session(pool=pool) as session:
        yield session
```

## Models

```python
# app/models.py
from chakra import Model, Field, String, Integer, DateTime, Boolean
from chakra import ForeignKey, OneToMany
from datetime import datetime

class User(Model):
    __tablename__ = "users"

    id = Field(Integer, primary_key=True)
    username = Field(String(50), unique=True)
    email = Field(String(255), unique=True)
    hashed_password = Field(String(255))
    is_active = Field(Boolean, default=True)
    created_at = Field(DateTime, default=datetime.utcnow)

    posts = OneToMany("Post", back_populates="author")

class Post(Model):
    __tablename__ = "posts"

    id = Field(Integer, primary_key=True)
    title = Field(String(200))
    content = Field(String)
    published = Field(Boolean, default=False)
    author_id = Field(Integer, ForeignKey("users.id"))

    author = ForeignKey(User, back_populates="posts")
```

## Pydantic Schemas

```python
# app/schemas.py
from pydantic import BaseModel, EmailStr
from datetime import datetime

class UserCreate(BaseModel):
    username: str
    email: EmailStr
    password: str

class UserResponse(BaseModel):
    id: int
    username: str
    email: str
    is_active: bool
    created_at: datetime

    class Config:
        from_attributes = True

class PostCreate(BaseModel):
    title: str
    content: str

class PostResponse(BaseModel):
    id: int
    title: str
    content: str
    published: bool
    author_id: int

    class Config:
        from_attributes = True
```

## Main Application

```python
# app/main.py
from fastapi import FastAPI
from contextlib import asynccontextmanager
from .database import init_db, close_db
from .routers import users, posts

@asynccontextmanager
async def lifespan(app: FastAPI):
    await init_db()
    yield
    await close_db()

app = FastAPI(
    title="My API",
    lifespan=lifespan
)

app.include_router(users.router, prefix="/users", tags=["users"])
app.include_router(posts.router, prefix="/posts", tags=["posts"])

@app.get("/health")
async def health():
    return {"status": "ok"}
```

## Routers

```python
# app/routers/users.py
from fastapi import APIRouter, Depends, HTTPException, status
from chakra import Session
from ..database import get_session
from ..models import User
from ..schemas import UserCreate, UserResponse
from passlib.hash import bcrypt

router = APIRouter()

@router.post("/", response_model=UserResponse, status_code=status.HTTP_201_CREATED)
async def create_user(user: UserCreate, session: Session = Depends(get_session)):
    # Check if user exists
    existing = await User.objects.filter(email=user.email).first()
    if existing:
        raise HTTPException(
            status_code=status.HTTP_400_BAD_REQUEST,
            detail="Email already registered"
        )

    db_user = User(
        username=user.username,
        email=user.email,
        hashed_password=bcrypt.hash(user.password)
    )
    session.add(db_user)
    await session.commit()

    return db_user

@router.get("/", response_model=list[UserResponse])
async def list_users(
    skip: int = 0,
    limit: int = 100,
    session: Session = Depends(get_session)
):
    users = await User.objects.offset(skip).limit(limit).all()
    return users

@router.get("/{user_id}", response_model=UserResponse)
async def get_user(user_id: int, session: Session = Depends(get_session)):
    user = await session.get(User, user_id)
    if not user:
        raise HTTPException(
            status_code=status.HTTP_404_NOT_FOUND,
            detail="User not found"
        )
    return user

@router.put("/{user_id}", response_model=UserResponse)
async def update_user(
    user_id: int,
    user_update: UserCreate,
    session: Session = Depends(get_session)
):
    user = await session.get(User, user_id)
    if not user:
        raise HTTPException(status_code=404, detail="User not found")

    user.username = user_update.username
    user.email = user_update.email
    await session.commit()

    return user

@router.delete("/{user_id}", status_code=status.HTTP_204_NO_CONTENT)
async def delete_user(user_id: int, session: Session = Depends(get_session)):
    user = await session.get(User, user_id)
    if not user:
        raise HTTPException(status_code=404, detail="User not found")

    session.delete(user)
    await session.commit()
```

## Running the Application

```bash
# Run migrations
chakra migrate apply

# Start server
uvicorn app.main:app --reload
```

## Testing

```python
# tests/test_users.py
import pytest
from httpx import AsyncClient
from app.main import app
from app.database import init_db, close_db

@pytest.fixture
async def client():
    await init_db()
    async with AsyncClient(app=app, base_url="http://test") as client:
        yield client
    await close_db()

@pytest.mark.asyncio
async def test_create_user(client):
    response = await client.post("/users/", json={
        "username": "testuser",
        "email": "test@example.com",
        "password": "secret"
    })
    assert response.status_code == 201
    data = response.json()
    assert data["username"] == "testuser"
    assert data["email"] == "test@example.com"
```
