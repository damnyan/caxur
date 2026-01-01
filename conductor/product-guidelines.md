# Product Guidelines

## Documentation and Communication
- **Tone and Voice**: Casual and conversational. The documentation and comments should be friendly, easy to read, and helpful to the user.
- **Style**: Prioritize conciseness. Explanations should be brief and to the point.
- **Standards**: Strictly adhere to Rust's official style guides and documentation conventions.

## Architectural Principles
- **Clean Architecture**: Strictly follow the separation of concerns and the dependency rule (`Domain` <- `Application` <- `Infrastructure` <- `Presentation`).
- **KISS (Keep It Simple, Stupid)**: Avoid over-engineering. Use the simplest solution that effectively meets the requirements.
- **DRY (Don't Repeat Yourself)**: Centralize common logic like validation, error mapping, and response formatting to prevent duplication.
- **Pragmatism**: Balance architectural purity with practical ease of use. Decisions should favor long-term maintainability while remaining accessible to developers.

## Dependency Management
- **Feature-Rich Strategy**: Include popular, well-maintained, and high-quality crates to provide a comprehensive and production-ready feature set out-of-the-box.
- **Selection Criteria**: Prioritize performance, security, and community trust when selecting new dependencies.

## Testing Strategy
- **Pragmatic Testing**: Focus on ensuring core logic, business rules, and critical paths are well-tested. 
- **Quality over Quantity**: While high coverage is valued, the priority is on meaningful tests that prevent regressions and verify correct behavior.
- **Balance**: Ensure testing provides confidence without becoming a bottleneck for rapid development.
