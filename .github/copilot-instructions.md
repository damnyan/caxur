# Caxur: Rust Axum Clean Architecture Project

## Core Principles

This project is built on the following core principles:

- **Clean Architecture**: Separation of concerns is paramount. The dependency rule is strictly enforced: `Domain ← Application ← Infrastructure ← Presentation`.
- **Idiomatic Rust**: Follow Rust best practices and conventions throughout the codebase. Use module names for context (`routes::auth::routes()` instead of `routes::auth::auth_routes()`), prefer `Result` types, leverage the type system, and write expressive, safe code.
- **KISS (Keep It Simple, Stupid)**: The structure is modular but avoids unnecessary abstraction overhead. We use simple dependency injection via constructors.
- **DRY (Don't Repeat Yourself)**: Common logic like validation, error mapping, and response formatting is centralized in the `shared` module.
- **TDD (Test Driven Development)**: The architecture supports easy unit testing by decoupling business logic from the database and HTTP layer.

## Architecture Overview

This is a **strict Clean Architecture** Rust/Axum REST API following the dependency rule: `Domain ← Application ← Infrastructure ← Presentation`. Each layer has distinct responsibilities and imports flow inward only.

### Layer Responsibilities

**Domain (`src/domain/`)**: Core entities and repository trait definitions

- Define structs (entities) with `#[derive(FromRow)]` for SQLx
- Define repository traits with `#[async_trait]` - these are **interfaces only**
- Use `time::OffsetDateTime` for timestamps with `#[serde(with = "time::serde::iso8601")]`
- Example: `src/domain/users.rs` defines `User`, `NewUser`, and `UserRepository` trait

**Application (`src/application/`)**: Business logic and use cases

- Create request DTOs with `#[derive(Deserialize, Validate)]`
- Use `validator` attributes: `#[validate(email)]`, `#[validate(length(min = 3, message = "..."))]`
- Implement use case structs that accept `Arc<dyn RepoTrait>` via constructor injection
- Put business logic in `execute()` methods
- Example: `src/application/users/create.rs` - `CreateUserUseCase::execute()`

**Infrastructure (`src/infrastructure/`)**: External integrations

- Implement repository traits from domain layer using SQLx
- Use `sqlx::query_as::<_, EntityType>()` for dynamic queries
- Repository structs hold `DbPool` and implement domain traits with `#[async_trait]`
- Mock implementations in `repositories/mock.rs` for testing (use `Arc<Mutex<Vec<T>>>` pattern)
- Example: `PostgresUserRepository` implements `UserRepository` trait

**Presentation (`src/presentation/`)**: HTTP layer

- **Handlers** (`handlers/`): Async functions that extract `State<DbPool>` and `ValidatedJson<RequestDTO>`
  - Instantiate concrete repository inside handler: `Arc::new(PostgresRepository::new(pool))`
  - Pass repository to use case constructor, call `execute()`
  - Return `(StatusCode::CREATED, Json(ApiResponse::new(result)))`
- **Routes** (`routes/`): Modular route definitions organized by feature
  - Each module exports a `routes()` function returning `Router<DbPool>`
  - Use idiomatic naming: `routes::auth::routes()`, `routes::users::routes()`
  - Routes use relative paths merged into main router with `.nest()`
- **Router** (`router.rs`): Main application router composing all feature routes
  - Uses `.nest("/api/v1/feature", routes::feature::routes())` for versioning
  - Applies global middleware with `.layer()`

**Shared (`src/shared/`)**: Cross-cutting utilities

- `error.rs`: `AppError` enum with `IntoResponse` impl - all errors convert here
- `validation.rs`: `ValidatedJson<T>` extractor auto-validates and returns 422 on failure
- `response.rs`: `ApiResponse<T>` wrapper for consistent JSON:API format

## Critical Patterns

### Dependency Injection

Constructor-based DI using trait objects:

```rust
pub struct CreateUserUseCase {
    repo: Arc<dyn UserRepository>,  // Trait object, not concrete type
}
```

### Error Flow

- Use `anyhow::Error` in domain/application layers for flexibility
- Convert to `AppError` at presentation boundary via `From` trait
- `AppError::IntoResponse` handles HTTP status codes and JSON error formatting

### Validation

- Add validation rules to request DTOs: `#[validate(email)]`, `#[validate(length(...))]`
- Use `ValidatedJson<T>` extractor in handlers - validation happens automatically
- Validation errors become `AppError::ValidationError` → 422 Unprocessable Entity

### Response Format

Always wrap success responses: `ApiResponse::new(data)` or `.with_meta(json!({...}))`

## Development Workflow

### Database Setup

```bash
# First time setup
sqlx database create
sqlx migrate run

# Create new migration
sqlx migrate add create_table_name
```

### Running the App

```bash
# Development
cargo run

# With Docker
docker-compose up

# Tests
cargo test
```

### Adding a New Feature

1. **Domain**: Create entity struct + repository trait in `src/domain/feature.rs`
2. **Infrastructure**: Implement repository in `src/infrastructure/repositories/feature.rs`
3. **Application**: Create request DTO + use case in `src/application/feature/action.rs`
4. **Presentation - Handlers**: Create handlers in `src/presentation/handlers/feature.rs`
5. **Presentation - Routes**: Create route module in `src/presentation/routes/feature.rs`
   - Export a `routes()` function that returns `Router<DbPool>`
   - Use relative paths (e.g., `"/", "/{id}"`)
6. **Router**: Register in `src/presentation/router.rs` using `.nest("/api/v1/feature", routes::feature::routes())`

### Testing Strategy

- Unit tests use `MockRepository` implementations from `infrastructure/repositories/mock.rs`
- Test use cases by injecting mock repos: `Arc::new(MockUserRepository::default())`
- Integration tests in handler files can test error response formatting

## Key Dependencies

- **Axum 0.8**: HTTP framework with extractors (`State`, custom `ValidatedJson`)
- **SQLx 0.8**: Compile-time verified queries, async Postgres driver
- **Validator 0.20**: Derive-based validation with custom messages
- **Time 0.3**: Use `OffsetDateTime` (not `chrono`) with iso8601 serde serialization

## Environment & Configuration

Required `.env` variables:

```
DATABASE_URL=postgres://user:password@localhost:5432/dbname
RUST_LOG=caxur=debug,tower_http=debug
```

Server runs on `0.0.0.0:3000`, migrations run automatically on startup in `main.rs`.
