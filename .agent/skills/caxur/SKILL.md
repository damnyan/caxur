---
name: Caxur Project Development
description: Specialized skill for the Caxur project, enforcing specific clean architecture layers, Rust coding standards, and workflow practices.
---

# Caxur Project Development Skill

This skill governs development within the Caxur project. It enforces strict adherence to Clean Architecture, Domain-Driven Design (DDD), and the project's specific technology stack.

## Core Principles

### 1. Architectural Integrity
- **Dependency Rule**: `Domain` <- `Application` <- `Infrastructure` <- `Presentation`.
- **Pure Domain**: The domain layer (`src/domain/`) MUST NOT depend on any external libraries (no `sqlx`, `axum`, etc.), except for `serde` (if needed), `time`, and `uuid`.
- **Rich Domain Model**: Avoid anemic models. Entities should encapuslate logic and validation.

### 2. Rust Coding Standards (Strict)
- **Formatting**: ALWAYS run `cargo fmt` before finishing.
- **Linting**: ALWAYS run `cargo clippy` and fix ALL warnings.
- **Naming**: `snake_case` for variables/functions, `PascalCase` for types.
- **Error Handling**: Use `Result`/`Option`. No `unwrap()` or `expect()` in production code. Use the project-standard `AppError` in `shared`.

### 3. Development Methodology
- **TDD is Mandatory**: Write failing tests first, then implementation, then refactor.
- **KISS, YAGNI, & DRY**: Keep it simple, don't build for the future, don't repeat yourself.

## Project Structure & Layers

### Domain (`src/domain/`)
- Pure business logic, entities, and repository **traits**.
- **NO** infrastructure details.

### Infrastructure (`src/infrastructure/`)
- Implements repository traits.
- Database models (`*DbModel`) go here in `db/models/`.
- Maps `DbModel` -> `DomainEntity` via `From` trait.

### Application (`src/application/`)
- Use Cases (Interactors) and Commands.
- Orchestrates domain objects to fulfill business rules.

### Presentation (`src/presentation/`)
- HTTP Handlers and DTOs.
- `Dto` structs MUST implement `utoipa::ToSchema`.
- Maps `DomainEntity` -> `Dto`.

## Implementation Workflow (Step-by-Step)

When adding a new feature, follow this **strict** order:

1.  **Domain**: Define the Entity struct and the Repository Trait.
2.  **Infrastructure**: Implement the Repository Trait (e.g., `PostgresRepository`).
    - Create `src/infrastructure/db/models/<entity>_model.rs`.
    - Implement `From<DbModel>` for `Entity`.
3.  **Application**: Create Request DTO/Command (with validation), Use Case struct, and implement logic.
4.  **Presentation**: Create Handler function.
    - Use `ValidatedJson` extractor.
    - Call Use Case.
    - Return `ApiResponse`.
5.  **Router**: Register the new handler in the presentation layer router.

## Technology Stack & Tools

- **Framework**: `axum`
- **Database**: `PostgreSQL` with `sqlx`
- **Docs**: `utoipa` (OpenAPI)
- **Validation**: `validator` crate
- **Logging**: `tracing`

## Verification Checklist

Before marking a task as complete:
1.  [ ] `cargo test` passes.
2.  [ ] `cargo clippy` is clean.
3.  [ ] `cargo fmt` has been run.
4.  [ ] New public APIs are documented with `utoipa`.
