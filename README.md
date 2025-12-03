# Rust Axum Clean Architecture Boilerplate

A robust, production-ready boilerplate for building REST APIs in Rust using **Axum**, **SQLx**, and **Tokio**. This project follows **Clean Architecture** principles and implements best practices for validation, error handling, and JSON:API responses.

## 🎯 Principles

This boilerplate is built on the following core principles:

-   **Clean Architecture**: Separation of concerns is paramount. The dependency rule is strictly enforced: `Domain` <- `Application` <- `Infrastructure` <- `Presentation`.
-   **KISS (Keep It Simple, Stupid)**: The structure is modular but avoids unnecessary abstraction overhead. We use simple dependency injection via constructors.
-   **DRY (Don't Repeat Yourself)**: Common logic like validation, error mapping, and response formatting is centralized in the `shared` module.
-   **TDD (Test Driven Development)**: The architecture supports easy unit testing by decoupling business logic from the database and HTTP layer.

## 📂 Project Structure

The source code is organized into four main layers plus a shared module:

```
src/
├── domain/             # 🧠 The Core
│   ├── users.rs        # Entities (Data structures)
│   └── mod.rs          # Repository Traits (Interfaces)
│
├── application/        # 💼 Business Logic
│   ├── users/          # Feature-specific use cases
│   │   ├── create.rs   # "Create User" logic (Command/Handler)
│   │   └── mod.rs
│   └── mod.rs
│
├── infrastructure/     # 🏗️ External World
│   ├── db.rs           # Database connection (SQLx)
│   ├── repositories/   # Implementation of Domain Repository Traits
│   │   ├── users.rs    # Postgres implementation
│   │   └── mod.rs
│   └── mod.rs
│
├── presentation/       # 🌐 HTTP Layer
│   ├── handlers/       # Axum Controllers/Handlers
│   │   ├── users.rs    # User endpoints
│   │   └── mod.rs
│   ├── router.rs       # Route definitions
│   └── mod.rs
│
└── shared/             # 🛠️ Utilities
    ├── error.rs        # Centralized AppError & HTTP mapping
    ├── response.rs     # JSON:API Response Wrapper
    ├── validation.rs   # Custom Request Extractor for Validation
    └── mod.rs
```

## 🚀 Getting Started

### Prerequisites
-   Rust (latest stable)
-   PostgreSQL
-   `sqlx-cli` (`cargo install sqlx-cli`)

### Installation

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/damnyan/caxur.git
    cd caxur
    ```

2.  **Setup Environment**:
    Copy the example environment file (or create one):
    ```bash
    # .env
    DATABASE_URL=postgres://user:password@localhost:5432/dbname
    RUST_LOG=caxur=debug,tower_http=debug
    ```

3.  **Setup Database**:
    ```bash
    # Create database
    sqlx database create

    # Run migrations
    sqlx migrate run
    ```

4.  **Run the Server**:
    ```bash
    cargo run
    ```
    The server will start at `http://127.0.0.1:3000`.

## 🛠️ How to Use (Where to put things)

When adding a new feature (e.g., "Products"), follow this flow:

### 1. Domain Layer (`src/domain/products.rs`)
Define **WHAT** your data is and **HOW** you access it (interface).
-   Create the `Product` struct (Entity).
-   Define the `ProductRepository` trait.

### 2. Infrastructure Layer (`src/infrastructure/repositories/products.rs`)
Implement **HOW** data is actually stored.
-   Create `PostgresProductRepository`.
-   Implement `ProductRepository` for it using SQLx.
-   *Tip: Write your SQL queries here.*

### 3. Application Layer (`src/application/products/create.rs`)
Define **WHAT** the system does (Business Logic).
-   Create a Request DTO (e.g., `CreateProductRequest`) with validation rules.
-   Create a Use Case struct (e.g., `CreateProductUseCase`).
-   Inject the repository trait into the Use Case.
-   Implement the `execute` method containing the logic.

### 4. Presentation Layer (`src/presentation/handlers/products.rs`)
Connect the **HTTP** world to your Application.
-   Create an async function (handler).
-   Use `ValidatedJson<CreateProductRequest>` to automatically validate input.
-   Call the Use Case.
-   Return `ApiResponse::new(result)`.

### 5. Router (`src/presentation/router.rs`)
-   Register your new handler in the `app` function.

## ✨ Key Features

### Validation
We use the `validator` crate. Request structs in the **Application** layer should derive `Validate`.
The `ValidatedJson` extractor in `shared/validation.rs` automatically runs these rules and returns a `422 Unprocessable Entity` if validation fails.

```rust
#[derive(Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
}
```

### Error Handling
All errors are converted to `AppError` (in `shared/error.rs`). This enum implements `IntoResponse`, ensuring that any error returned from a handler is automatically formatted as a proper JSON error response with the correct HTTP status code.

### JSON:API Response
Wrap your successful responses in `ApiResponse::new(data)`. This ensures a consistent response structure:
```json
{
  "data": { ... },
  "meta": { ... } // Optional
}
```

## 🧪 Testing & Coverage

This project maintains **98.97% code coverage** with comprehensive unit and integration tests.

### Test Structure

- **Unit Tests** (78 tests): Located in `#[cfg(test)]` modules within each file
  - Application layer: Uses `MockRepository` for isolated testing
  - Domain layer: Pure logic tests (password hashing, validation)
  - Shared utilities: Error handling, response formatting, validation

- **Integration Tests** (35 tests): Located in `tests/` directory
  - Uses real PostgreSQL database (`TEST_DATABASE_URL`)
  - Tests full HTTP request/response cycle
  - Covers authentication, authorization, and edge cases

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_create_user

# Run integration tests only
cargo test --test users_test

# Run with test database
TEST_DATABASE_URL="postgres://postgres:postgres@localhost:5432/caxur_test" cargo test
```

### Coverage Analysis

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Lcov Html

# View HTML report
open tarpaulin-report.html
```

**Current Coverage**: 98.97% (578/584 lines)
- ✅ 100% - Application layer (auth, users)
- ✅ 100% - Domain layer (auth, password, users)
- ✅ 100% - Infrastructure layer (auth, db, repositories)
- ✅ 100% - Presentation layer (handlers, routes, router, openapi)
- ✅ 100% - Shared layer (error, response, validation, pagination)
- ✅ 77% - Main entry point (all testable logic covered)

### Writing Tests

**Unit Test Example** (Application Layer):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::repositories::mock::MockUserRepository;

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
    }
}
```

**Integration Test Example**:
```rust
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
    common::cleanup_test_db(&pool).await;
}
```

## 🔐 Authentication

This boilerplate includes JWT-based authentication with refresh tokens.

### Setup JWT Keys

```bash
./scripts/generate_keys.sh
```

This generates ES256 (ECDSA P-256) key pair in `keys/` directory.

### Environment Variables

```bash
JWT_PRIVATE_KEY_PATH=keys/private_key.pem
JWT_PUBLIC_KEY_PATH=keys/public_key.pem
JWT_ACCESS_TOKEN_EXPIRY=900        # 15 minutes
JWT_REFRESH_TOKEN_EXPIRY=604800    # 7 days
```

### Authentication Flow

1. **Login**: `POST /api/v1/auth/login`
   - Returns access token + refresh token
   
2. **Protected Routes**: Include `Authorization: Bearer <access_token>` header

3. **Refresh**: `POST /api/v1/auth/refresh`
   - Use refresh token to get new access token

### Protected Handler Example

```rust
pub async fn update_user(
    auth: AuthUser,  // Validates JWT automatically
    Path(id): Path<Uuid>,
    ValidatedJson(req): ValidatedJson<UpdateUserRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Ensure user can only update their own account
    let auth_user_id = auth.claims.user_id()?;
    if auth_user_id != id {
        return Err(AppError::Forbidden("Cannot modify other users".to_string()));
    }
    // ... rest of logic
}
```

## 📊 API Documentation

Interactive Swagger UI available at: `http://localhost:3000/swagger-ui`

All endpoints are documented using `utoipa` macros:

```rust
#[utoipa::path(
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully"),
        (status = 422, description = "Validation error")
    ),
    tag = "users"
)]
pub async fn create_user(...) { }
```

## 🐳 Docker Support

```bash
# Start PostgreSQL and application
docker-compose up

# Run in background
docker-compose up -d

# Stop services
docker-compose down
```

## 📝 Code Quality

### Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

### Pre-commit Checklist
- [ ] All tests pass: `cargo test`
- [ ] Coverage ≥ 98%: `cargo tarpaulin`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Code formatted: `cargo fmt`
- [ ] OpenAPI docs updated
- [ ] Migration files added (if schema changed)

## 🤝 Contributing

1. Follow Clean Architecture principles
2. Maintain 98%+ test coverage
3. Use trait-based dependency injection
4. Write comprehensive tests (unit + integration)
5. Document all public APIs with `utoipa`
6. Follow JSON:API specification for responses

## 📄 License

MIT License - See LICENSE file for details

## 🙏 Acknowledgments

Built with:
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [Tokio](https://tokio.rs/) - Async runtime
- [utoipa](https://github.com/juhaku/utoipa) - OpenAPI documentation
- [validator](https://github.com/Keats/validator) - Request validation
