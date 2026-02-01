---
title: Type Safety
description: Compile-time and runtime type safety in Chakra ORM
tags:
  - types
  - safety
  - validation
---

# Type Safety

Chakra ORM provides comprehensive type safety through compile-time checks in Rust and rich type hints in Python.

## Rust: Compile-Time Safety

### Model Field Validation

```rust
#[derive(Model)]
#[chakra(table = "users")]
pub struct User {
    #[chakra(primary_key)]
    pub id: i64,

    #[chakra(max_length = 50)]
    pub username: String,  // Enforced at compile time

    #[chakra(nullable)]
    pub bio: Option<String>,  // Must be Option<T> for nullable
}

// Compile error: non-nullable field must not be Option
#[chakra(max_length = 50)]
pub email: Option<String>,  // ❌ Error without nullable attribute
```

### Query Type Checking

```rust
// ✅ Correct: comparing String field with &str
let users = User::query()
    .filter(User::username().eq("alice"))
    .all(&pool)
    .await?;

// ❌ Compile error: type mismatch
let users = User::query()
    .filter(User::username().eq(42))  // Cannot compare String with i32
    .all(&pool)
    .await?;

// ❌ Compile error: field doesn't exist
let users = User::query()
    .filter(User::nonexistent().eq("value"))
    .all(&pool)
    .await?;
```

### Result Type Safety

```rust
// Return type is statically known
let user: Option<User> = User::get(&pool, 1).await?;

let users: Vec<User> = User::query().all(&pool).await?;

let count: i64 = User::query().count(&pool).await?;

// Aggregate results are typed
let stats: AggregateResult<(i64, f64)> = User::query()
    .aggregate((Count::all(), Avg::new(User::age())))
    .one(&pool)
    .await?;
```

## Python: Type Hints & Runtime Validation

### Model Type Hints

```python
from chakra import Model, Field, String, Integer, DateTime
from typing import Optional
from datetime import datetime

class User(Model):
    __tablename__ = "users"

    id: int = Field(Integer, primary_key=True)
    username: str = Field(String(50))
    email: str = Field(String(255))
    bio: Optional[str] = Field(String, nullable=True)
    created_at: datetime = Field(DateTime, default=datetime.utcnow)
```

### IDE Support

With type hints, your IDE provides:

- **Auto-completion** for field names
- **Type checking** with mypy/pyright
- **Inline documentation**
- **Refactoring support**

```python
# IDE knows user.username is str
user = await User.objects.get(id=1)
print(user.username.upper())  # ✅ IDE autocomplete works

# mypy/pyright catches errors
user.username = 123  # ❌ Type error: expected str, got int
```

### QuerySet Type Hints

```python
from chakra import QuerySet

# QuerySet methods return typed results
users: list[User] = await User.objects.all()
user: User = await User.objects.get(id=1)
user: User | None = await User.objects.filter(id=1).first()
count: int = await User.objects.count()
exists: bool = await User.objects.filter(id=1).exists()
```

### Runtime Validation

Chakra validates data at runtime:

```python
# Field validation on assignment
user = User()
user.username = "a" * 100  # ❌ ValidationError: max_length is 50

# Type coercion with validation
user.id = "123"  # Coerced to int: 123
user.id = "abc"  # ❌ ValidationError: cannot convert to int

# Constraint validation on save
user = User(email="duplicate@example.com")
await session.add(user)
await session.commit()  # ❌ UniqueViolation if email exists
```

## Field Validation Options

### String Fields

```python
# Python
username = Field(String(50),
    min_length=3,
    max_length=50,
    pattern=r"^[a-z0-9_]+$"
)
```

```rust
// Rust
#[chakra(min_length = 3, max_length = 50, pattern = r"^[a-z0-9_]+$")]
pub username: String,
```

### Numeric Fields

```python
# Python
age = Field(Integer,
    min_value=0,
    max_value=150
)

price = Field(Decimal(10, 2),
    min_value=0
)
```

```rust
// Rust
#[chakra(min_value = 0, max_value = 150)]
pub age: i32,

#[chakra(precision = 10, scale = 2, min_value = 0)]
pub price: Decimal,
```

### Custom Validators

```python
# Python
from chakra import Field, String, validator

class User(Model):
    email = Field(String(255))

    @validator("email")
    def validate_email(cls, value: str) -> str:
        if "@" not in value:
            raise ValueError("Invalid email address")
        return value.lower()
```

```rust
// Rust
impl User {
    fn validate_email(email: &str) -> Result<String, ValidationError> {
        if !email.contains('@') {
            return Err(ValidationError::new("Invalid email address"));
        }
        Ok(email.to_lowercase())
    }
}
```

## Error Messages

Type errors include helpful context:

```
chakra.ValidationError: Field 'username' validation failed

  Value: "ab"
  Constraint: min_length = 3

  The value must be at least 3 characters long.
  Got: 2 characters

  Location:
    File "app/models.py", line 15
    username = Field(String(50), min_length=3)
```

## Database Constraint Mapping

Field constraints map to database constraints:

| Chakra Option | Database Constraint |
|---------------|---------------------|
| `primary_key=True` | PRIMARY KEY |
| `unique=True` | UNIQUE |
| `nullable=False` | NOT NULL |
| `foreign_key=...` | FOREIGN KEY REFERENCES |
| `check=...` | CHECK constraint |
| `default=...` | DEFAULT value |

```python
class User(Model):
    __tablename__ = "users"
    __table_args__ = (
        CheckConstraint("age >= 0", name="ck_age_positive"),
        UniqueConstraint("email", "tenant_id", name="uq_email_tenant"),
    )

    id = Field(Integer, primary_key=True)
    email = Field(String(255), unique=True)  # Creates UNIQUE constraint
    age = Field(Integer, check="age >= 0")   # Creates CHECK constraint
```

## Integration with Type Checkers

### mypy Configuration

```ini
# mypy.ini
[mypy]
plugins = chakra.mypy_plugin

[mypy-chakra.*]
ignore_missing_imports = False
```

### pyright Configuration

```json
// pyrightconfig.json
{
  "typeCheckingMode": "strict",
  "reportMissingTypeStubs": false
}
```

### Example Type Check Output

```bash
$ mypy app/
app/views.py:25: error: Argument "id" to "get" has incompatible type "str"; expected "int"
app/views.py:30: error: "User" has no attribute "nonexistent"
Found 2 errors in 1 file
```
