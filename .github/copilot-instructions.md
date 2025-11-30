# Caxur: Rust Axum Clean Architecture Project

## Core Principles

This project enforces strict Clean Architecture with the dependency rule: `Domain ← Application ← Infrastructure ← Presentation`. Imports flow inward only.

- **Trait-based DI**: All repositories are injected as `Arc<dyn Trait>`, never concrete types
- **Idiomatic Rust**: Use module names for context (`routes::auth::routes()` not `routes::auth::auth_routes()`)
- **JSON:API Compliance**: All responses use `JsonApiResponse<JsonApiResource<T>>` structure
- **Central Error Handling**: All errors convert to `AppError` enum, which implements `IntoResponse`
- **Testability**: Mock repositories use `Arc<Mutex<Vec<T>>>` pattern in `infrastructure/repositories/mock.rs`

## Architecture Layers

### Domain (`src/domain/`)

**What**: Core entities and repository trait definitions (interfaces only)

Entities must derive:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, ToSchema)]
```

Timestamps use `time::OffsetDateTime`:

```rust
#[serde(with = "time::serde::iso8601")]
#[schema(value_type = String, example = "2025-11-29T10:00:00Z")]
pub created_at: OffsetDateTime,
```

Repository traits:

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, new: NewUser) -> Result<User, anyhow::Error>;
    // ... other CRUD operations
}
```

**Examples**: `src/domain/users.rs` (User + UserRepository), `src/domain/auth.rs` (Claims + AuthService)

### Application (`src/application/`)

**What**: Business logic isolated from HTTP/DB concerns

Request DTOs pattern:

```rust
#[derive(Deserialize, Validate, ToSchema)]
pub struct CreateUserRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[validate(length(min = 6, message = "Password must be at least 6 characters"))]
    pub password: String,
}
```

Use case pattern (constructor injection):

```rust
pub struct CreateUserUseCase {
    repo: Arc<dyn UserRepository>,  // Trait, not concrete type!
}

impl CreateUserUseCase {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub async fn execute(&self, req: CreateUserRequest) -> Result<User, AppError> {
        // Business logic here
    }
}
```

**Key Files**: `src/application/users/create.rs`, `src/application/auth/login.rs`

### Infrastructure (`src/infrastructure/`)

**What**: Database implementations and external service integrations

Repository implementation pattern:

```rust
pub struct PostgresUserRepository {
    pool: DbPool,
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create(&self, new: NewUser) -> Result<User, anyhow::Error> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, password_hash)
             VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(&new.username)
        .bind(&new.email)
        .bind(&new.password_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
}
```

**Mock repositories** for testing (`src/infrastructure/repositories/mock.rs`):

```rust
#[derive(Clone, Default)]
pub struct MockUserRepository {
    users: Arc<Mutex<Vec<User>>>,
}
```

### Presentation (`src/presentation/`)

**What**: HTTP handlers, routes, and API documentation

**Handlers** (`handlers/`) instantiate dependencies:

```rust
pub async fn create_user(
    State(pool): State<DbPool>,
    ValidatedJson(req): ValidatedJson<CreateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let repo = Arc::new(PostgresUserRepository::new(pool));
    let use_case = CreateUserUseCase::new(repo);
    let user = use_case.execute(req).await?;

    let resource = JsonApiResource::new("users", user.id.to_string(), UserResource::from(user));
    Ok((StatusCode::CREATED, Json(JsonApiResponse::new(resource))))
}
```

**Routes** (`routes/`) export `routes() -> Router<DbPool>`:

```rust
pub fn routes() -> Router<DbPool> {
    Router::new()
        .route("/", post(users::create_user).get(users::list_users))
        .route("/{id}", get(users::get_user).put(users::update_user).delete(users::delete_user))
}
```

**Router** (`router.rs`) composes all routes:

```rust
pub fn app(pool: DbPool) -> Router {
    Router::new()
        .nest("/api/v1/users", routes::users::routes())
        .nest("/api/v1/auth", routes::auth::routes())
        .layer(TraceLayer::new_for_http())
        .with_state(pool)
}
```

### Shared (`src/shared/`)

**What**: Cross-cutting utilities

- `error.rs`: `AppError` enum with `IntoResponse` trait - converts all errors to JSON:API format
- `validation.rs`: `ValidatedJson<T>` extractor - auto-validates using `validator` crate
- `response.rs`: `JsonApiResponse`, `JsonApiResource`, `JsonApiMeta`, `JsonApiLinks` - consistent response structure

## Critical Implementation Patterns

### JSON:API Response Structure

All successful responses use:

```rust
JsonApiResponse::new(JsonApiResource::new("users", id, attributes))
```

With pagination:

```rust
JsonApiResponse::new(resources)
    .with_meta(JsonApiMeta::new().with_total(100).with_page(1).with_per_page(10))
    .with_links(JsonApiLinks::new().with_self(url).with_next(next_url))
```

### Error Handling Flow

1. Use `anyhow::Error` in domain/application layers
2. Convert to `AppError` at presentation boundary
3. `AppError::IntoResponse` generates JSON:API error response

Database unique violations auto-detected:

```rust
// In error.rs IntoResponse impl
if db_err.is_unique_violation() {
    // Returns 422 with descriptive message
}
```

### Authentication Pattern

JWT tokens use ES256 (ECDSA P-256). Generate keys:

```bash
./scripts/generate_keys.sh
```

Handler pattern for authenticated routes:

```rust
pub async fn update_user(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    auth: AuthUser,  // Custom extractor validates JWT
    ValidatedJson(req): ValidatedJson<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    let auth_user_id = auth.claims.user_id()?;
    if auth_user_id != id {
        return Err(AppError::Forbidden("...".to_string()));
    }
    // ...
}
```

Token generation helper (`src/application/auth/token_utils.rs`):

```rust
generate_and_store_tokens(user_id, user_type, &auth_service, &refresh_token_repo,
                          access_expiry, refresh_expiry).await?
```

### Resource Transformation

Convert domain entities to API resources:

```rust
#[derive(Serialize, ToSchema)]
pub struct UserResource {
    pub id: String,
    pub username: String,
    #[serde(with = "time::serde::iso8601")]
    #[schema(value_type = String)]
    pub created_at: OffsetDateTime,
}

impl From<User> for UserResource {
    fn from(user: User) -> Self {
        Self { id: user.id.to_string(), username: user.username, created_at: user.created_at }
    }
}
```

## Development Workflow

### First-Time Setup

```bash
# Install SQLx CLI if not present
cargo install sqlx-cli --no-default-features --features postgres

# Create database and run migrations
sqlx database create
sqlx migrate run

# Generate JWT keys (required for auth endpoints)
./scripts/generate_keys.sh

# Configure .env
cat > .env << EOF
DATABASE_URL=postgres://user:password@localhost:5432/caxur
RUST_LOG=caxur=debug,tower_http=debug
JWT_PRIVATE_KEY_PATH=keys/private_key.pem
JWT_PUBLIC_KEY_PATH=keys/public_key.pem
JWT_ACCESS_TOKEN_EXPIRY=900
JWT_REFRESH_TOKEN_EXPIRY=604800
EOF

# Run server (migrations run automatically)
cargo run
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test --package caxur --bin caxur -- application::users::create::tests --nocapture

# Tests use MockRepository - no database required
```

### Adding a New Feature

1. **Domain** (`src/domain/feature.rs`): Define entity + repository trait
2. **Infrastructure** (`src/infrastructure/repositories/feature.rs`): Implement repository + mock
3. **Application** (`src/application/feature/action.rs`): Create request DTO + use case with `execute()`
4. **Presentation Handlers** (`src/presentation/handlers/feature.rs`): HTTP handlers with utoipa docs
5. **Presentation Routes** (`src/presentation/routes/feature.rs`): Export `routes() -> Router<DbPool>`
6. **Router** (`src/presentation/router.rs`): Add `.nest("/api/v1/feature", routes::feature::routes())`
7. **OpenAPI** (`src/presentation/openapi.rs`): Register schemas in `ApiDoc` derive macro

### Database Migrations

```bash
# Create migration
sqlx migrate add create_products_table

# Edit migrations/TIMESTAMP_create_products_table.sql
# Then run
sqlx migrate run
```

### OpenAPI/Swagger

Access at: `http://localhost:3000/swagger-ui`

Document handlers with `#[utoipa::path(...)]` macro - see `src/presentation/handlers/users.rs`

## Key Dependencies

- **Axum 0.8**: HTTP framework - use `State<DbPool>` extractor
- **SQLx 0.8**: Async Postgres driver - use `#[derive(FromRow)]` on entities
- **Validator 0.20**: Request validation - `#[validate(...)]` on DTO fields
- **Time 0.3**: Date/time handling - always use `OffsetDateTime` with iso8601 serde
- **jsonwebtoken**: JWT auth - ES256 algorithm, load keys from PEM files
- **utoipa**: OpenAPI docs - `#[derive(ToSchema)]` on all API types

## Common Patterns Reference

**Pagination query params**:

```rust
#[derive(Deserialize, IntoParams)]
pub struct ListRequest {
    #[serde(default = "default_page")]
    pub page: PaginationParams,
}

#[derive(Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page_number")]
    pub number: i64,
    #[serde(default = "default_page_size")]
    pub size: i64,
}
```

**Custom async validation**:

```rust
impl CreateUserRequest {
    pub async fn validate_unique_email(&self, repo: &Arc<dyn UserRepository>) -> Result<(), AppError> {
        if let Some(_) = repo.find_by_email(&self.email).await? {
            return Err(AppError::ValidationError("Email already exists".to_string()));
        }
        Ok(())
    }
}
// Call in use case before main logic
```

**Password hashing** (see `src/domain/password.rs`):

```rust
let hash = PasswordService::hash_password(password)?;
let is_valid = PasswordService::verify_password(password, &hash)?;
```
