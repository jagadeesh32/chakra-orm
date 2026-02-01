---
title: Framework Integrations
description: Use Chakra ORM with your favorite framework
---

# Framework Integrations

Chakra ORM works with any Python or Rust web framework.

## Python Frameworks

<div class="grid cards" markdown>

-   **[FastAPI](fastapi.md)**

    The most popular async Python framework.

-   **[Django](django.md)**

    Use Chakra alongside or instead of Django ORM.

-   **[Flask](flask.md)**

    Lightweight and flexible.

-   **Starlette**

    FastAPI's foundation, works identically.

-   **Sanic**

    High-performance async framework.

</div>

## Rust Frameworks

<div class="grid cards" markdown>

-   **[Axum](axum.md)**

    Modern, ergonomic, built on Tokio.

-   **[Actix Web](actix.md)**

    High-performance actor framework.

-   **Rocket**

    Type-safe with powerful macros.

-   **Warp**

    Composable filter-based routing.

</div>

## Key Patterns

### Dependency Injection

All frameworks benefit from injecting sessions:

=== "FastAPI"

    ```python
    async def get_session():
        async with Session() as session:
            yield session

    @app.get("/users/{id}")
    async def get_user(id: int, session: Session = Depends(get_session)):
        return await session.get(User, id)
    ```

=== "Axum"

    ```rust
    async fn get_user(
        State(pool): State<Pool>,
        Path(id): Path<i64>,
    ) -> Result<Json<User>, StatusCode> {
        let user = User::get(&pool, id).await?;
        Ok(Json(user))
    }
    ```

### Lifecycle Management

Properly manage pool lifecycle:

```python
from contextlib import asynccontextmanager

@asynccontextmanager
async def lifespan(app: FastAPI):
    # Startup: create pool
    app.state.pool = await chakra.create_pool(DATABASE_URL)
    yield
    # Shutdown: close pool
    await app.state.pool.close()

app = FastAPI(lifespan=lifespan)
```
