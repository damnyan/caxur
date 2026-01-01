# Project Rules & Guidelines
(Derived from README.md)

## Core Principles
- **Clean Architecture**: Adhere strictly to the dependency rule: `Domain` <- `Application` <- `Infrastructure` <- `Presentation`.
- **KISS**: Keep implementation simple; use dependency injection via constructors; avoid unnecessary abstraction.
- **DRY**: Centralize common logic (validation, error mapping, response formatting) in the `shared` module.
- **TDD**: Write tests first; decouple business logic from external concerns to facilitate unit testing.

## Project Structure & Layers
1. **Domain** (`src/domain/`): Pure entities and repository interfaces (Traits). No external dependencies.
2. **Application** (`src/application/`): Business logic, use cases, and commands. Depends only on Domain.
3. **Infrastructure** (`src/infrastructure/`): Database (SQLx), repository implementations. Depends on Domain.
4. **Presentation** (`src/presentation/`): HTTP handlers, router. Depends on Application.
5. **Shared** (`src/shared/`): Common utilities used across layers.

## Implementation Workflow
When adding a new feature, follow this order:
1. **Domain**: Define Entity struct and Repository Trait.
2. **Infrastructure**: Implement Repository Trait (e.g., `PostgresRepository`).
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

### Documentation
- Document all public API endpoints using `utoipa` macros.
- Ensure Swagger UI (`/swagger-ui`) accurately reflects the API.
