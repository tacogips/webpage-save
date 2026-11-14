# CLAUDE.md

Guidance for Claude Code when working with this codebase.

## Response Rules

- Start first response with "I will continue thinking and providing output in English."
- Always respond in English regardless of user's language
- Acknowledge reading CLAUDE.md in first response
- Declare cargo commands use CARGO_TERM_QUIET=true

### Task Management

**Special Commands**:

- "show tasks/todos" → Display current session TODOs
- "show plan" → Display planned tasks
- "continue tasks" → Start working on pending tasks

## Git Commit Policy

- Follow commit message format from this document
- Auto-proceed without user confirmation
- **NO Claude Code attribution** - commits appear as user-made only
- Execute all "How to change code" steps before completing
- Never delete tests without deliberation
- Begin with "Your instruction is {corrected English}" for English instructions

## How to change code

**MANDATORY** steps after ANY code changes:

### Pre-modification Review Check

1. Search for review comments in modification area (`REV-` prefix)
2. Check commit history for review context
3. Respect review guidance unless functional changes require deviation

### Required Steps

1. Follow code style guidelines
2. Update documentation (//!, ///) and verify: `CARGO_TERM_QUIET=true cargo doc --no-deps`
3. **Commit Only When Requested**: Commits should only be made when explicitly instructed by the user:

   a) Stage files (`git add`)
   b) Show summary with message and diff stats
   c) Execute commit
   d) Show result

**Summary Format**: Files, TODOs, Future Plan, commit message
**Color Coding**: 🔴 Deletions, 🟢 Additions/Modifications, 🟡 Renames

**IMPORTANT**: Never skip steps unless explicitly told.

### Test Guidelines

- Fix code to make tests pass (never remove test cases)
- Add tests for new functionality
- Maintain test coverage and strictness
- Use designated test resources when available
- Include meaningful assertions with appropriate timeouts

### Communication Guidelines

- Seek clarification for ambiguous instructions
- Understand user goals before implementation
- Present options when unsure about details

### Git Commit Message Format

**Structure** (9 sections):

1. **Objective**: Purpose, goals, challenges addressed
2. **Primary Changes**: Main changes and intent
3. **Key Technical Concepts**: Technologies, frameworks involved
4. **Files and Code Sections**: Modified files with summaries
5. **Problem Solving**: Issues addressed (include bug reproduction for fixes)
6. **Impact**: Effect on overall project
7. **Related Commits**: `Related: abc123d, def456a`
8. **Unresolved TODOs**: `- [ ]` format
9. **Future Plan**: `- [ ]` format

**Bug Fix Rule**: Always include reproduction method in commit log with fictional values for sensitive data.

## Build & Run Commands

- Build: `cargo build`
- Run: `cargo run`
- Release: `cargo build --release`
- Test: `cargo test [test_name]`
- Lint: `cargo clippy`
- Format: `cargo fmt`

### Quiet Cargo Commands

Use `CARGO_TERM_QUIET=true` prefix to reduce output.

## Code Style Guidelines

- Rust 2024 edition, rustfmt default settings
- snake_case naming, prefer Result<T,E> over unwrap()
- Organize imports: std first, then external crates
- Use structured logging (env_logger, tracing)
- Type annotations for public functions
- Effective ownership system (avoid unnecessary clones)
- Module declaration order: `pub mod`, `mod`, `pub use`, `use` (each block separated by blank line)
- Place structs/enums with their implementations together

### Implementation Principles

- **Divide and Conquer**: Apply divide and conquer approach to break down complex implementations into appropriate methods and split files as needed for maintainability
- **DRY (Don't Repeat Yourself)**: Properly consolidate common processing logic and avoid recreating existing shared functionality. Always check for existing common utilities before implementing new ones
- **Code Reusability**: Identify and extract reusable components into separate modules or functions to promote code sharing across the codebase
- **Modular Design**: Structure code into logical modules with clear responsibilities and minimal coupling between components

### General Coding Principles

**SOLID Principles**:
- **Single Responsibility**: Each function/struct should have one reason to change
- **Open/Closed**: Open for extension, closed for modification
- **Liskov Substitution**: Subtypes must be substitutable for their base types
- **Interface Segregation**: Clients shouldn't depend on interfaces they don't use
- **Dependency Inversion**: Depend on abstractions, not concretions

**Clean Code Practices**:
- **Meaningful Names**: Use descriptive, searchable names that express intent
- **Small Functions**: Functions should do one thing well (max 20-30 lines)
- **Function Arguments**: Minimize argument count (ideally 0-2, maximum 3)
- **Comments**: Code should be self-documenting; comments explain "why", not "what"
- **Consistent Formatting**: Follow project's formatting standards religiously

**Error Handling & Robustness**:
- **Fail Fast**: Detect and report errors as early as possible
- **Graceful Degradation**: System should continue operating with reduced functionality
- **Input Validation**: Always validate inputs at system boundaries
- **Resource Management**: Properly handle resource cleanup (RAII pattern)
- **Defensive Programming**: Assume inputs can be malicious or malformed

**Performance & Efficiency**:
- **Premature Optimization**: Avoid unless profiling shows actual bottlenecks
- **Big O Awareness**: Understand algorithmic complexity of your solutions
- **Memory Management**: Minimize allocations, avoid memory leaks
- **Lazy Loading**: Load resources only when needed
- **Caching Strategy**: Cache expensive operations with appropriate invalidation

**Testing & Quality**:
- **Test-Driven Development**: Write tests before implementation when possible
- **Test Coverage**: Aim for high coverage but focus on critical paths
- **Test Isolation**: Tests should be independent and repeatable
- **Mock External Dependencies**: Use mocks for external services/APIs
- **Integration Tests**: Test the complete flow of critical features

**Security & Safety**:
- **Principle of Least Privilege**: Grant minimum necessary permissions
- **Input Sanitization**: Never trust user input; validate and sanitize
- **Secure Defaults**: Use secure configurations by default
- **Secrets Management**: Never hardcode credentials; use environment variables
- **Audit Trail**: Log security-relevant events for monitoring

### Rust-Specific Guidelines

**Error Handling & Reliability**:
- Always use `Result<T, E>` instead of `unwrap()` or `panic!()`
- Implement proper error propagation using `?` operator
- Use `anyhow` for error handling in application code
- Handle all async operations with proper timeout handling

**Performance & Memory**:
- Avoid unnecessary `clone()` operations - use references when possible
- Use `ahash` for HashMap operations (already in dependencies)
- Implement pagination for large data sets
- Use streaming for large API responses

**Async Programming**:
- Use tokio runtime features appropriately
- Implement proper cancellation handling
- Use `futures` utilities for complex async operations
- Apply timeouts to external API calls

**Security**:
- Never log sensitive data (tokens, personal information)
- Use `rustls` for TLS connections
- Validate all external inputs
- Use secure defaults for HTTP clients

**Testing & Documentation**:
- Write unit tests for all public functions
- Use `mockito` for HTTP mocking in tests
- Document all public APIs with `///` comments
- Use `serial_test` for tests requiring isolation

**Configuration Management**:
- Use `toml` for configuration files
- Support environment variables
- Use `dirs` for platform-specific paths
