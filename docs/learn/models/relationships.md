---
title: Relationships
description: Define relationships between models in Chakra ORM
---

# Relationships

Chakra ORM supports all standard relationship types: one-to-one, one-to-many, and many-to-many.

## One-to-One

A user has one profile, and a profile belongs to one user.

=== "Python"

    ```python
    from chakra import Model, Field, Integer, String, Text
    from chakra import ForeignKey, OneToOne

    class User(Model):
        __tablename__ = "users"

        id = Field(Integer, primary_key=True)
        username = Field(String(50))

        # Relationship (reverse side)
        profile = OneToOne("Profile", back_populates="user")

    class Profile(Model):
        __tablename__ = "profiles"

        id = Field(Integer, primary_key=True)
        bio = Field(Text)

        # Foreign key (owning side)
        user_id = Field(Integer, ForeignKey("users.id"), unique=True)
        user = ForeignKey(User, back_populates="profile")
    ```

=== "Rust"

    ```rust
    #[derive(Model)]
    #[chakra(table = "users")]
    pub struct User {
        #[chakra(primary_key)]
        pub id: i64,
        pub username: String,

        #[chakra(relation = OneToOne, model = "Profile", foreign_key = "user_id")]
        pub profile: Related<Option<Profile>>,
    }

    #[derive(Model)]
    #[chakra(table = "profiles")]
    pub struct Profile {
        #[chakra(primary_key)]
        pub id: i64,
        pub bio: String,

        #[chakra(foreign_key = "users.id", unique)]
        pub user_id: i64,

        #[chakra(relation = ManyToOne, model = "User")]
        pub user: Related<User>,
    }
    ```

### Usage

```python
# Create with relationship
user = User(username="alice")
profile = Profile(bio="Hello!", user=user)
session.add_all([user, profile])
await session.commit()

# Query with eager loading
user = await User.objects.select_related("profile").get(id=1)
print(user.profile.bio)  # No additional query

# Access relationship (lazy load if not prefetched)
user = await User.objects.get(id=1)
profile = await user.profile  # Triggers query
```

## One-to-Many

A user has many posts, each post belongs to one user.

=== "Python"

    ```python
    class User(Model):
        __tablename__ = "users"

        id = Field(Integer, primary_key=True)
        username = Field(String(50))

        # One-to-many (reverse side)
        posts = OneToMany("Post", back_populates="author")

    class Post(Model):
        __tablename__ = "posts"

        id = Field(Integer, primary_key=True)
        title = Field(String(200))
        content = Field(Text)

        # Foreign key (owning side)
        author_id = Field(Integer, ForeignKey("users.id"))
        author = ForeignKey(User, back_populates="posts")
    ```

=== "Rust"

    ```rust
    #[derive(Model)]
    #[chakra(table = "users")]
    pub struct User {
        #[chakra(primary_key)]
        pub id: i64,
        pub username: String,

        #[chakra(relation = OneToMany, model = "Post", foreign_key = "author_id")]
        pub posts: Related<Vec<Post>>,
    }

    #[derive(Model)]
    #[chakra(table = "posts")]
    pub struct Post {
        #[chakra(primary_key)]
        pub id: i64,
        pub title: String,
        pub content: String,

        #[chakra(foreign_key = "users.id")]
        pub author_id: i64,

        #[chakra(relation = ManyToOne, model = "User")]
        pub author: Related<User>,
    }
    ```

### Usage

```python
# Create posts for user
user = await User.objects.get(id=1)
post = Post(title="Hello", content="...", author_id=user.id)
session.add(post)
await session.commit()

# Query with prefetch (separate queries, efficient for *-to-many)
users = await User.objects.prefetch_related("posts").all()
for user in users:
    print(f"{user.username} has {len(user.posts)} posts")

# Filter by relationship
posts = await Post.objects.filter(author__username="alice").all()
```

## Many-to-Many

Posts have many tags, tags have many posts.

=== "Python"

    ```python
    class Post(Model):
        __tablename__ = "posts"

        id = Field(Integer, primary_key=True)
        title = Field(String(200))

        # Many-to-many
        tags = ManyToMany("Tag", through="post_tags", back_populates="posts")

    class Tag(Model):
        __tablename__ = "tags"

        id = Field(Integer, primary_key=True)
        name = Field(String(50), unique=True)

        # Many-to-many (reverse)
        posts = ManyToMany(Post, through="post_tags", back_populates="tags")

    # Junction table (auto-created if not defined)
    class PostTag(Model):
        __tablename__ = "post_tags"

        post_id = Field(Integer, ForeignKey("posts.id"), primary_key=True)
        tag_id = Field(Integer, ForeignKey("tags.id"), primary_key=True)
        created_at = Field(DateTime, default=datetime.utcnow)
    ```

=== "Rust"

    ```rust
    #[derive(Model)]
    #[chakra(table = "posts")]
    pub struct Post {
        #[chakra(primary_key)]
        pub id: i64,
        pub title: String,

        #[chakra(relation = ManyToMany, model = "Tag", through = "post_tags")]
        pub tags: Related<Vec<Tag>>,
    }

    #[derive(Model)]
    #[chakra(table = "tags")]
    pub struct Tag {
        #[chakra(primary_key)]
        pub id: i64,
        pub name: String,

        #[chakra(relation = ManyToMany, model = "Post", through = "post_tags")]
        pub posts: Related<Vec<Post>>,
    }

    #[derive(Model)]
    #[chakra(table = "post_tags")]
    pub struct PostTag {
        #[chakra(primary_key, foreign_key = "posts.id")]
        pub post_id: i64,
        #[chakra(primary_key, foreign_key = "tags.id")]
        pub tag_id: i64,
    }
    ```

### Usage

```python
# Add tags to post
post = await Post.objects.get(id=1)
tag = await Tag.objects.get(name="python")

await post.tags.add(tag)

# Or set multiple
python = await Tag.objects.get(name="python")
rust = await Tag.objects.get(name="rust")
await post.tags.set([python, rust])

# Remove
await post.tags.remove(rust)

# Clear all
await post.tags.clear()

# Query
posts = await Post.objects.prefetch_related("tags").all()
for post in posts:
    tag_names = [tag.name for tag in post.tags]
    print(f"{post.title}: {tag_names}")

# Filter by many-to-many
posts = await Post.objects.filter(tags__name="python").all()
```

## Self-Referential Relationships

For hierarchical data like categories or threaded comments.

```python
class Comment(Model):
    __tablename__ = "comments"

    id = Field(Integer, primary_key=True)
    content = Field(Text)

    # Self-reference
    parent_id = Field(Integer, ForeignKey("comments.id"), nullable=True)
    parent = ForeignKey("Comment", back_populates="replies")
    replies = OneToMany("Comment", back_populates="parent")

# Usage
root = Comment(content="Root comment")
reply1 = Comment(content="Reply 1", parent=root)
reply2 = Comment(content="Reply 2", parent=root)

session.add_all([root, reply1, reply2])
await session.commit()

# Get with replies
comment = await Comment.objects.prefetch_related("replies").get(id=1)
for reply in comment.replies:
    print(reply.content)
```

## Loading Strategies

### select_related (JOIN)

Use for **to-one** relationships. Single query with JOIN.

```python
# Single query: SELECT users.*, profiles.* FROM users JOIN profiles ...
user = await User.objects.select_related("profile").get(id=1)
print(user.profile.bio)  # Already loaded
```

### prefetch_related (Separate Queries)

Use for **to-many** relationships. Multiple efficient queries.

```python
# Query 1: SELECT * FROM users
# Query 2: SELECT * FROM posts WHERE author_id IN (1, 2, 3, ...)
users = await User.objects.prefetch_related("posts").all()
for user in users:
    print(len(user.posts))  # Already loaded
```

### Nested Prefetch

```python
# Prefetch posts and each post's tags
users = await User.objects.prefetch_related(
    "posts",
    "posts__tags",  # Nested
).all()
```

### Filtered Prefetch

```python
from chakra import Prefetch

# Only prefetch published posts
users = await User.objects.prefetch_related(
    Prefetch(
        "posts",
        queryset=Post.objects.filter(published=True).order_by("-created_at")
    )
).all()
```

## Cascade Options

```python
# Foreign key with cascade
author_id = Field(
    Integer,
    ForeignKey("users.id", on_delete="CASCADE", on_update="CASCADE")
)
```

| Option | Description |
|--------|-------------|
| `CASCADE` | Delete/update related records |
| `SET NULL` | Set foreign key to NULL |
| `SET DEFAULT` | Set to default value |
| `RESTRICT` | Prevent delete/update if related |
| `NO ACTION` | Database default behavior |
