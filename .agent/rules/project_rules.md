---
trigger: always_on
---

# Project Rules & Guidelines

## Core Principles
- **Clean Architecture**: Adhere strictly to the dependency rule: `Domain` <- `Application` <- `Infrastructure` <- `Presentation`.
- **Domain-Driven Design (DDD)**: Maintain a rich domain model; avoid anemic models.
- **KISS**: Keep implementation simple; use dependency injection via constructors; avoid unnecessary abstraction.
- **YAGNI (You Aren't Gonna Need It)**: Do not implement features or code "just in case". Implement only what is needed now.
- **DRY**: Centralize common logic (validation, error mapping, response formatting) in the `shared` module.

## Project Structure & Layers
1. **Domain** (`src/domain/`): Pure entities and repository interfaces (Traits). No external dependencies (NO `sqlx`, NO `axum`, NO `utoipa`). `serde`, `time`, `uuid` are allowed.
2. **Application** (`src/application/`): Business logic, use cases, and commands. Depends only on Domain.
3. **Infrastructure** (`src/infrastructure/`): Database (SQLx), repository implementations. Depends on Domain.
   - Database models must exist in `src/infrastructure/db/models/`.
   - Must implement `From<*DbModel> for DomainEntity`.
4. **Presentation** (`src/presentation/`): HTTP handlers, router. Depends on Application.
   - DTOs define the API contracts and must derive `utoipa::ToSchema`.
   - Do NOT use domain entities directly for API requests/responses.
5. **Shared** (`src/shared/`): Common utilities used across layers.

## Rich Domain Model Pattern
We avoid "Anemic Domain Models". Domain entities should contain business logic, not just data.
- **Factory Methods**: Use static methods on the entity to create valid instances (e.g., `Claims::new_access_token(...)`).
- **Validation**: Entities should enforce their own validity where possible.
- **Helper Methods**: Encapsulate domain knowledge (e.g., `permission.description()`) within the entity.
- **Encapsulation**: If a field implies complex rules, wrap it and provide methods to manipulate it.

## Naming Conventions
- **Domain Entities**: `User`, `Role` (No suffix)
- **Database Models**: `UserDbModel` (Suffix: `DbModel`)
- **API Resources**: `UserResource`, `PermissionDto` (Suffix: `Resource` or `Dto`)
- **Repositories**: `PostgresUserRepository` (Implementation), `UserRepository` (Trait)

## Development Methodology
- **TDD (Test-Driven Development)**: Write tests first; decouple business logic from external concerns to facilitate unit testing.

## Implementation Workflow
When adding a new feature, follow this order:
1. **Domain**: Define Entity struct and Repository Trait.
2. **Infrastructure**: Implement Repository Trait (e.g., `PostgresRepository`) and add/update `DbModel`.
3. **Application**: Create Request DTO (with validation), Use Case struct, and implement `execute` logic.
4. **Presentation**: Create Handler function, use `ValidatedJson`, call Use Case, return `ApiResponse`.
5. **Router**: Register the new handler.

## Development Standards

### Validation
- Use `validator` crate.
- Request structs must derive `Validate`.
- Use `ValidatedJson` extractor from `shared` to automatically validate requests.

### Error Handling
- Use `AppError` (from `shared/error.rs`) for all errors.
- Ensure `AppError` maps to appropriate HTTP status codes.

### Response Format
- Responses must follow JSON:API structure.
- Wrap success responses in `ApiResponse::new(data)`.

### Testing & Coverage
- **Coverage Goal**: Maintain **98%+** code coverage.
- **Unit Tests**: Place in `#[cfg(test)]` modules within the source file. Use `MockRepository` for application layer tests.
- **Integration Tests**: Place in `tests/` directory; use real DB via `TEST_DATABASE_URL`.
- **Pre-commit**:
  - `cargo test` (All tests pass)
  - `cargo tarpaulin` (Coverage check)
  - `cargo clippy` (No warnings)
  - `cargo fmt` (Code formatted)

### Code Style
- **No Fully Qualified Names (FQN)**: Always import types, functions, and modules at the top of the file. Do not use fully qualified names (e.g., `use crate::domain::User;` instead of `crate::domain::User::new()`).

### Documentation
- Document all public API endpoints using `utoipa` macros.
- Ensure Swagger UI (`/swagger-ui`) accurately reflects the API.
