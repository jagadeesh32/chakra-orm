---
title: Quick Start (Rust)
description: Build your first Rust application with Chakra ORM
---

# Quick Start (Rust)

Build a complete CRUD application in Rust with Chakra ORM.

## Prerequisites

- Rust 1.70+
- PostgreSQL (or SQLite)
- Tokio runtime

## 1. Create Project

```bash
cargo new blog-app
cd blog-app
```

## 2. Add Dependencies

Edit `Cargo.toml`:

```toml
[package]
name = "blog-app"
version = "0.1.0"
edition = "2021"

[dependencies]
chakra = { version = "0.1", features = ["postgres", "derive", "runtime-tokio", "migrations"] }
tokio = { version = "1", features = ["full"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
```

## 3. Define Models

Create `src/models.rs`:

```rust
use chakra::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Model, Serialize, Deserialize)]
#[chakra(table = "users")]
pub struct User {
    #[chakra(primary_key, auto_increment)]
    pub id: i64,

    #[chakra(max_length = 50, unique, index)]
    pub username: String,

    #[chakra(max_length = 255, unique)]
    pub email: String,

    #[chakra(default = true)]
    pub is_active: bool,

    #[chakra(default = "now()")]
    pub created_at: DateTime<Utc>,

    #[chakra(relation = OneToMany, model = "Post", foreign_key = "author_id")]
    #[serde(skip)]
    pub posts: Related<Vec<Post>>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            id: 0,
            username: String::new(),
            email: String::new(),
            is_active: true,
            created_at: Utc::now(),
            posts: Related::new(),
        }
    }
}

#[derive(Debug, Clone, Model, Serialize, Deserialize)]
#[chakra(table = "posts")]
pub struct Post {
    #[chakra(primary_key, auto_increment)]
    pub id: i64,

    #[chakra(max_length = 200)]
    pub title: String,

    pub content: String,

    #[chakra(default = false)]
    pub published: bool,

    #[chakra(default = "now()")]
    pub created_at: DateTime<Utc>,

    #[chakra(foreign_key = "users.id")]
    pub author_id: i64,

    #[chakra(relation = ManyToOne, model = "User")]
    #[serde(skip)]
    pub author: Related<User>,
}

impl Default for Post {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::new(),
            content: String::new(),
            published: false,
            created_at: Utc::now(),
            author_id: 0,
            author: Related::new(),
        }
    }
}
```

## 4. Create Configuration

Create `chakra.toml`:

```toml
[database]
url = "postgresql://postgres:password@localhost:5432/blog"

[pool]
min_connections = 2
max_connections = 10
```

## 5. Create and Apply Migrations

```bash
# Generate migration
chakra migrate make --name initial

# Apply
chakra migrate apply
```

## 6. Main Application

Edit `src/main.rs`:

```rust
mod models;

use chakra::prelude::*;
use models::{User, Post};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to database
    let pool = chakra::connect("postgresql://postgres:password@localhost:5432/blog").await?;

    println!("Connected to database!");

    // ========================================
    // CREATE
    // ========================================
    println!("\n=== Creating users ===");

    let mut alice = User {
        username: "alice".into(),
        email: "alice@example.com".into(),
        ..Default::default()
    };

    let mut bob = User {
        username: "bob".into(),
        email: "bob@example.com".into(),
        ..Default::default()
    };

    // Insert users
    alice.insert(&pool).await?;
    bob.insert(&pool).await?;

    println!("Created alice (id={})", alice.id);
    println!("Created bob (id={})", bob.id);

    // Create posts
    let mut post1 = Post {
        title: "Hello World".into(),
        content: "My first post!".into(),
        author_id: alice.id,
        published: true,
        ..Default::default()
    };

    let mut post2 = Post {
        title: "Learning Chakra ORM".into(),
        content: "Chakra is amazing...".into(),
        author_id: alice.id,
        ..Default::default()
    };

    post1.insert(&pool).await?;
    post2.insert(&pool).await?;

    println!("Created {} posts", 2);

    // ========================================
    // READ
    // ========================================
    println!("\n=== Querying ===");

    // Get by ID
    let user = User::get(&pool, alice.id).await?;
    if let Some(user) = user {
        println!("Found user: {}", user.username);
    }

    // Filter query
    let active_users = User::query()
        .filter(User::is_active().eq(true))
        .all(&pool)
        .await?;
    println!("Active users: {}", active_users.len());

    // Complex query
    let users = User::query()
        .filter(User::username().starts_with("a"))
        .filter(User::is_active().eq(true))
        .order_by(User::created_at().desc())
        .all(&pool)
        .await?;

    for user in &users {
        println!("  - {}", user.username);
    }

    // Query with prefetch
    let users_with_posts = User::query()
        .prefetch_related(User::posts())
        .all(&pool)
        .await?;

    for user in &users_with_posts {
        let post_count = user.posts.get()?.len();
        println!("{} has {} posts", user.username, post_count);
    }

    // ========================================
    // UPDATE
    // ========================================
    println!("\n=== Updating ===");

    alice.email = "alice.smith@example.com".into();
    alice.update(&pool).await?;
    println!("Updated alice's email");

    // Bulk update
    let count = Post::query()
        .filter(Post::author_id().eq(alice.id))
        .update(Post::published().set(true))
        .execute(&pool)
        .await?;
    println!("Published {} posts", count);

    // ========================================
    // DELETE
    // ========================================
    println!("\n=== Deleting ===");

    bob.delete(&pool).await?;
    println!("Deleted bob");

    let remaining = User::query().count(&pool).await?;
    println!("Remaining users: {}", remaining);

    Ok(())
}
```

## 7. Run It

```bash
cargo run
```

Output:

```
Connected to database!

=== Creating users ===
Created alice (id=1)
Created bob (id=2)
Created 2 posts

=== Querying ===
Found user: alice
Active users: 2
  - alice
alice has 2 posts
bob has 0 posts

=== Updating ===
Updated alice's email
Published 2 posts

=== Deleting ===
Deleted bob
Remaining users: 1
```

## Full Example: Axum Web Server

```rust
// main.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chakra::prelude::*;
use serde::{Deserialize, Serialize};

mod models;
use models::{User, Post};

type Pool = chakra::Pool;

#[derive(Deserialize)]
struct CreateUser {
    username: String,
    email: String,
}

#[derive(Serialize)]
struct UserResponse {
    id: i64,
    username: String,
    email: String,
}

async fn create_user(
    State(pool): State<Pool>,
    Json(payload): Json<CreateUser>,
) -> Result<Json<UserResponse>, StatusCode> {
    let mut user = User {
        username: payload.username,
        email: payload.email,
        ..Default::default()
    };

    user.insert(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
    }))
}

async fn list_users(State(pool): State<Pool>) -> Result<Json<Vec<UserResponse>>, StatusCode> {
    let users = User::query()
        .all(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response: Vec<UserResponse> = users
        .into_iter()
        .map(|u| UserResponse {
            id: u.id,
            username: u.username,
            email: u.email,
        })
        .collect();

    Ok(Json(response))
}

async fn get_user(
    State(pool): State<Pool>,
    Path(id): Path<i64>,
) -> Result<Json<UserResponse>, StatusCode> {
    let user = User::get(&pool, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(UserResponse {
        id: user.id,
        username: user.username,
        email: user.email,
    }))
}

#[tokio::main]
async fn main() {
    let pool = chakra::connect("postgresql://localhost/blog")
        .await
        .expect("Failed to connect to database");

    let app = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user))
        .with_state(pool);

    println!("Server running on http://localhost:3000");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

## Next Steps

- [Rust Model Macros](../../reference/rust/macros.md) — Deep dive into derive macros
- [Query Builder](../../reference/rust/query.md) — Complete query API
- [Axum Integration](../integrations/axum.md) — Full Axum guide
- [Actix Integration](../integrations/actix.md) — Actix Web guide
