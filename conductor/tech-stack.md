# Technology Stack

## Core Technologies
- **Programming Language:** Rust (Edition 2024)
- **Web Framework:** Axum (v0.8)
- **Asynchronous Runtime:** Tokio (v1.48)

## Database & Data Persistence
- **Database:** PostgreSQL
- **SQL Toolkit:** SQLx (v0.8) - Async SQL toolkit with compile-time verification
- **Migrations:** SQLx CLI

## Security & Authentication
- **Password Hashing:** Argon2
- **Token Management:** JSON Web Tokens (JWT) via `jsonwebtoken` crate
- **Cryptography:** P256 (NIST P-256 elliptic curve) for signing keys
- **Validation:** `validator` crate for input validation

## API Documentation & Interface
- **Specification:** OpenAPI v3.0
- **Generator:** `utoipa` (Code-first approach)
- **UI:** Swagger UI (via `utoipa-swagger-ui`)

## Infrastructure & DevOps
- **Containerization:** Docker
- **Orchestration:** Docker Compose (for local development)
- **Logging/Tracing:** `tracing` and `tracing-subscriber`

## Testing
- **Unit & Integration:** Standard `cargo test` framework
- **Coverage:** `tarpaulin`
