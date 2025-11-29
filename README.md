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
