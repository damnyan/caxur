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

## Testing Patterns

### Coverage Requirement
**Maintain 98%+ code coverage**. Current: 98.97% (578/584 lines)

### Test Structure
- **Unit Tests**: `#[cfg(test)] mod tests` within each file
- **Integration Tests**: `tests/` directory with real PostgreSQL

### Unit Test Patterns

**Application Layer** (use MockRepository):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::mock::MockUserRepository;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_create_user_success() {
        let repo = Arc::new(MockUserRepository::default());
        let use_case = CreateUserUseCase::new(repo);
        
        let request = CreateUserRequest {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
        };
        
        let result = use_case.execute(request).await;
        assert!(result.is_ok());
        
        let user = result.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[tokio::test]
    async fn test_create_user_validation_error() {
        let repo = Arc::new(MockUserRepository::default());
        let use_case = CreateUserUseCase::new(repo);
        
        let request = CreateUserRequest {
            username: "".to_string(),  // Invalid
            email: "invalid-email".to_string(),  // Invalid
            password: "123".to_string(),  // Too short
        };
        
        let result = use_case.execute(request).await;
        assert!(result.is_err());
    }
}
```

**Domain Layer** (pure logic):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = "password123";
        let hash = PasswordService::hash_password(password).unwrap();
        
        assert!(PasswordService::verify_password(password, &hash).unwrap());
        assert!(!PasswordService::verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn test_password_hash_error() {
        let result = PasswordService::hash_password("");
        assert!(result.is_err());
    }
}
```

**Shared Layer** (error handling, validation):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_error_to_status_code() {
        assert_eq!(AppError::NotFound("User not found".to_string()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(AppError::ValidationError("Invalid email".to_string()).status_code(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(AppError::Unauthorized("Invalid token".to_string()).status_code(), StatusCode::UNAUTHORIZED);
    }
}
```

### Integration Test Patterns

**Setup** (`tests/common/mod.rs`):
```rust
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
use uuid::Uuid;

#[allow(dead_code)]
pub async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/caxur_test".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!().run(&pool).await.expect("Failed to run migrations");
    pool
}

#[allow(dead_code)]
pub async fn cleanup_test_db(pool: &PgPool) {
    sqlx::query("TRUNCATE users, refresh_tokens CASCADE")
        .execute(pool)
        .await
        .expect("Failed to cleanup test database");
}

#[allow(dead_code)]
pub fn generate_test_token(user_id: Uuid) -> String {
    use caxur::domain::auth::AuthService;
    use caxur::infrastructure::auth::JwtAuthService;

    let auth_service = JwtAuthService::new(
        "keys/private_key.pem",
        "keys/public_key.pem",
        900,
        604800,
    ).expect("Failed to create auth service");

    auth_service.generate_access_token(user_id, "user".to_string())
        .expect("Failed to generate test token")
}
```

**Handler Tests** (`tests/users_test.rs`):
```rust
use axum::{body::Body, http::{Request, StatusCode}};
use serde_json::json;
use tower::ServiceExt;

mod common;

#[tokio::test]
async fn test_create_user() {
    let pool = common::setup_test_db().await;
    common::cleanup_test_db(&pool).await;
    
    let app = caxur::presentation::router::app(pool.clone());
    
    let request_body = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });
    
    let response = app.oneshot(
        Request::builder()
            .uri("/api/v1/users")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(request_body.to_string()))
            .unwrap(),
    ).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::CREATED);
    
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(json["data"]["type"], "users");
    assert_eq!(json["data"]["attributes"]["username"], "testuser");
    assert_eq!(json["data"]["attributes"]["email"], "test@example.com");
    
    common::cleanup_test_db(&pool).await;
}

#[tokio::test]
async fn test_update_user_forbidden() {
    let pool = common::setup_test_db().await;
    common::cleanup_test_db(&pool).await;
    
    // Create two users
    let app = caxur::presentation::router::app(pool.clone());
    
    // Create user 1
    let user1_response = app.clone().oneshot(
        Request::builder()
            .uri("/api/v1/users")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(json!({"username": "user1", "email": "user1@example.com", "password": "password123"}).to_string()))
            .unwrap(),
    ).await.unwrap();
    
    let user1_body = axum::body::to_bytes(user1_response.into_body(), usize::MAX).await.unwrap();
    let user1_json: serde_json::Value = serde_json::from_slice(&user1_body).unwrap();
    let user1_id = user1_json["data"]["id"].as_str().unwrap();
    
    // Create user 2 and get their token
    let user2_response = app.clone().oneshot(
        Request::builder()
            .uri("/api/v1/users")
            .method("POST")
            .header("content-type", "application/json")
            .body(Body::from(json!({"username": "user2", "email": "user2@example.com", "password": "password123"}).to_string()))
            .unwrap(),
    ).await.unwrap();
    
    let user2_body = axum::body::to_bytes(user2_response.into_body(), usize::MAX).await.unwrap();
    let user2_json: serde_json::Value = serde_json::from_slice(&user2_body).unwrap();
    let user2_id_uuid = uuid::Uuid::parse_str(user2_json["data"]["id"].as_str().unwrap()).unwrap();
    let user2_token = common::generate_test_token(user2_id_uuid);
    
    // Try to update user1 with user2's token (should be forbidden)
    let update_response = app.oneshot(
        Request::builder()
            .uri(&format!("/api/v1/users/{}", user1_id))
            .method("PUT")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", user2_token))
            .body(Body::from(json!({"username": "hacked"}).to_string()))
            .unwrap(),
    ).await.unwrap();
    
    assert_eq!(update_response.status(), StatusCode::FORBIDDEN);
    
    common::cleanup_test_db(&pool).await;
}
```

### Test Coverage Rules

1. **100% coverage required** for:
   - Domain layer (business logic)
   - Application layer (use cases)
   - Shared utilities
   - Infrastructure repositories (via integration tests)

2. **Acceptable uncovered**:
   - `src/main.rs` entry point (signal handling)
   - Boilerplate that delegates to tested functions

3. **Run coverage**:
   ```bash
   cargo tarpaulin --out Lcov Html
   ```

4. **Coverage targets**:
   - Overall: ≥ 98%
   - Per module: ≥ 95% (except main.rs)

### Test Naming Conventions

- Unit tests: `test_<function>_<scenario>` (e.g., `test_create_user_success`, `test_create_user_validation_error`)
- Integration tests: `test_<endpoint>_<scenario>` (e.g., `test_create_user`, `test_update_user_forbidden`)

### Test Organization

```
tests/
├── common/
│   └── mod.rs          # Shared test utilities
├── users_test.rs       # User endpoint tests
├── auth_test.rs        # Auth endpoint tests
├── db_test.rs          # Database connection tests
└── refresh_tokens_test.rs  # Refresh token repository tests
```

**Password hashing** (see `src/domain/password.rs`):

```rust
let hash = PasswordService::hash_password(password)?;
let is_valid = PasswordService::verify_password(password, &hash)?;
```
