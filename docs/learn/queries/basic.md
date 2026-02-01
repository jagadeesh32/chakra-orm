---
title: Basic Queries
description: Perform CRUD operations with Chakra ORM
---

# Basic Queries

Learn the fundamental query operations: Create, Read, Update, Delete.

## Create

### Single Record

=== "Python"

    ```python
    from chakra import Session
    from models import User

    async with Session() as session:
        # Create instance
        user = User(
            username="alice",
            email="alice@example.com",
            is_active=True
        )

        # Add to session
        session.add(user)

        # Commit to database
        await session.commit()

        # ID is now available
        print(f"Created user with ID: {user.id}")
    ```

=== "Rust"

    ```rust
    let mut user = User {
        id: 0,
        username: "alice".into(),
        email: "alice@example.com".into(),
        is_active: true,
        ..Default::default()
    };

    user.insert(&pool).await?;

    println!("Created user with ID: {}", user.id);
    ```

### Multiple Records

=== "Python"

    ```python
    users = [
        User(username=f"user_{i}", email=f"user_{i}@example.com")
        for i in range(100)
    ]

    session.add_all(users)
    await session.commit()

    # Or bulk create (more efficient)
    await User.objects.bulk_create(users)
    ```

=== "Rust"

    ```rust
    let users: Vec<User> = (0..100)
        .map(|i| User {
            username: format!("user_{}", i),
            email: format!("user_{}@example.com", i),
            ..Default::default()
        })
        .collect();

    User::bulk_insert(&pool, &users).await?;
    ```

## Read

### Get by Primary Key

=== "Python"

    ```python
    # Returns None if not found
    user = await session.get(User, 1)

    # Raises NotFoundError if not found
    user = await User.objects.get(id=1)

    # Get or create
    user, created = await User.objects.get_or_create(
        username="alice",
        defaults={"email": "alice@example.com"}
    )
    ```

=== "Rust"

    ```rust
    // Returns Option<User>
    let user = User::get(&pool, 1).await?;

    // Returns User or error
    let user = User::get_or_404(&pool, 1).await?;
    ```

### Get All

=== "Python"

    ```python
    # All records
    users = await User.objects.all()

    # With filter
    active_users = await User.objects.filter(is_active=True).all()
    ```

=== "Rust"

    ```rust
    let users = User::query().all(&pool).await?;

    let active_users = User::query()
        .filter(User::is_active().eq(true))
        .all(&pool)
        .await?;
    ```

### Get First/Last

=== "Python"

    ```python
    # First record
    first_user = await User.objects.order_by("created_at").first()

    # Last record
    last_user = await User.objects.order_by("created_at").last()
    ```

=== "Rust"

    ```rust
    let first = User::query()
        .order_by(User::created_at().asc())
        .first(&pool)
        .await?;

    let last = User::query()
        .order_by(User::created_at().desc())
        .first(&pool)
        .await?;
    ```

### Count and Exists

=== "Python"

    ```python
    # Count
    count = await User.objects.filter(is_active=True).count()

    # Exists
    exists = await User.objects.filter(email="alice@example.com").exists()
    ```

=== "Rust"

    ```rust
    let count = User::query()
        .filter(User::is_active().eq(true))
        .count(&pool)
        .await?;

    let exists = User::query()
        .filter(User::email().eq("alice@example.com"))
        .exists(&pool)
        .await?;
    ```

## Update

### Single Record

=== "Python"

    ```python
    async with Session() as session:
        # Get the record
        user = await session.get(User, 1)

        # Modify
        user.email = "newemail@example.com"
        user.is_active = False

        # Commit changes
        await session.commit()
    ```

=== "Rust"

    ```rust
    let mut user = User::get(&pool, 1).await?.unwrap();

    user.email = "newemail@example.com".into();
    user.is_active = false;

    user.update(&pool).await?;
    ```

### Bulk Update

=== "Python"

    ```python
    # Update all matching records
    count = await User.objects.filter(
        is_active=False
    ).update(is_active=True)

    print(f"Updated {count} users")

    # Update with expressions
    from chakra import F
    await Post.objects.filter(id=1).update(
        view_count=F("view_count") + 1
    )
    ```

=== "Rust"

    ```rust
    let count = User::query()
        .filter(User::is_active().eq(false))
        .update(User::is_active().set(true))
        .execute(&pool)
        .await?;

    // With expressions
    Post::query()
        .filter(Post::id().eq(1))
        .update(Post::view_count().set(F::col(Post::view_count()) + 1))
        .execute(&pool)
        .await?;
    ```

### Update or Create (Upsert)

=== "Python"

    ```python
    user, created = await User.objects.update_or_create(
        email="alice@example.com",  # Lookup fields
        defaults={                   # Fields to update/create
            "username": "alice",
            "is_active": True,
        }
    )
    ```

=== "Rust"

    ```rust
    let user = User::upsert(&pool)
        .on_conflict(&["email"])
        .update(&["username", "is_active"])
        .values(User {
            email: "alice@example.com".into(),
            username: "alice".into(),
            is_active: true,
            ..Default::default()
        })
        .execute()
        .await?;
    ```

## Delete

### Single Record

=== "Python"

    ```python
    async with Session() as session:
        user = await session.get(User, 1)
        session.delete(user)
        await session.commit()

    # Or directly
    await User.objects.filter(id=1).delete()
    ```

=== "Rust"

    ```rust
    let user = User::get(&pool, 1).await?.unwrap();
    user.delete(&pool).await?;

    // Or directly
    User::query()
        .filter(User::id().eq(1))
        .delete()
        .execute(&pool)
        .await?;
    ```

### Bulk Delete

=== "Python"

    ```python
    # Delete all matching
    count = await User.objects.filter(
        is_active=False,
        created_at__lt=datetime(2020, 1, 1)
    ).delete()

    print(f"Deleted {count} users")
    ```

=== "Rust"

    ```rust
    let count = User::query()
        .filter(User::is_active().eq(false))
        .filter(User::created_at().lt(Utc.ymd(2020, 1, 1).and_hms(0, 0, 0)))
        .delete()
        .execute(&pool)
        .await?;
    ```
