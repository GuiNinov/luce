# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

Luce is a graph-based task management system optimized for parallel execution workflows. It enables multiple Claude Code sessions to coordinate on complex, multi-threaded work by treating tasks as nodes in a directed acyclic graph (DAG) where dependencies define execution constraints, not artificial ordering.

**Key Documents**:
- `SPEC.md` - Complete project specification and vision
- `PLAN.md` - Implementation roadmap and technical decisions
- `src/shared/FEATURES.md` - Detailed shared package functionality and API reference

## Development Setup

This is a multi-package Rust workspace project. To set up:

1. Ensure Rust is installed with `cargo --version`
2. The workspace contains these packages:
   - `shared` - Common data structures and traits
   - `core` - Graph engine and task management logic
   - `cli` - Command-line interface
   - `api` - REST API server
   - `mcp` - Model Context Protocol server for Claude Code integration
   - `ui` - Yew-based web frontend

## Commands

Development commands (will be established as packages are created):
- `cargo build` - Build all workspace packages
- `cargo test` - Run all tests across workspace
- `cargo run --bin luce-cli` - Run CLI interface
- `cargo run --bin luce-api` - Start API server
- `cargo run --bin luce-mcp` - Start MCP server

## Architecture

**Multi-Package Workspace Structure**:
```
src/
├── shared/     # Core data types, traits, utilities
├── core/       # Task graph engine, dependency resolution
├── cli/        # Command-line interface
├── api/        # REST endpoints and WebSocket server
├── mcp/        # MCP server for Claude Code integration
└── ui/         # Yew frontend for graph visualization
```

**Core Concepts**:
- **TaskGraph**: DAG representing tasks and dependencies
- **Parallel Readiness Engine**: Calculates which tasks can execute simultaneously
- **Session Coordination**: Multiple Claude sessions coordinate through shared state
- **Dynamic Unlocking**: Task completion immediately enables dependent tasks

**Development Approach**:
- **LLM-Driven**: Designed for development with multiple Claude Code sessions
- **Package Independence**: Each package can be developed in parallel
- **Test-First**: Comprehensive testing strategy across all packages
- **Integration-Focused**: Clear interfaces between packages for seamless coordination

**Key Files to Understand**:
- `shared/src/task.rs` - Core Task data structures and methods
- `shared/src/graph.rs` - TaskGraph implementation and dependency management
- `core/src/graph.rs` - Dependency resolution and parallel readiness calculation
- `mcp/src/server.rs` - Claude Code integration and session coordination

## Development Guidelines

### Testing Requirements

**All code MUST include comprehensive unit tests.** This is a fundamental requirement for the Luce project.

**Testing Standards:**
- **Coverage**: Every public method and function must have corresponding tests
- **Edge Cases**: Test boundary conditions, error scenarios, and invalid inputs
- **Documentation**: Tests serve as living documentation of expected behavior
- **Regression Prevention**: Tests prevent future changes from breaking existing functionality

**Testing Practices:**
- Write tests **before** or **alongside** implementation (TDD/BDD approach)
- Use descriptive test names that explain what is being tested
- Include both positive and negative test cases
- Test error conditions and edge cases thoroughly
- Ensure tests are deterministic and don't rely on external state

**Current Testing Status:**
- `shared` package: **31 comprehensive unit tests** covering all functionality
- Target for all packages: **90%+ test coverage**

**Test Categories:**
1. **Unit Tests**: Test individual functions and methods in isolation
2. **Integration Tests**: Test interactions between modules
3. **Property Tests**: Test invariants and properties (where applicable)
4. **Performance Tests**: Validate performance characteristics for critical paths

**Running Tests:**
```bash
# Run all tests
cargo test

# Run tests for specific package
cargo test -p luce-shared
cargo test -p luce-core

# Run tests with verbose output
cargo test -- --nocapture
```

**Test Organization:**
- Place unit tests in `#[cfg(test)]` modules within source files
- Use integration tests in `tests/` directories for cross-module testing
- Group related tests using `mod` statements for organization

### Code Quality Standards

**Documentation:**
- All public APIs must have rustdoc comments
- Include usage examples in documentation
- Document complex algorithms and design decisions

**Error Handling:**
- Use the `LuceError` enum for all error types
- Provide meaningful error messages
- Handle all error cases explicitly (no unwrap() in production code)

**Performance:**
- Profile critical paths and document performance characteristics
- Use appropriate data structures for the use case
- Consider memory allocation patterns in hot paths

### Mandatory Code Quality Workflow

**CRITICAL REQUIREMENTS - MUST BE FOLLOWED:**

Claude Code sessions **MUST** complete these steps before concluding any code changes:

1. **Run Tests First**: Execute `cargo test` to ensure all existing functionality continues to work
   - All tests must pass before proceeding
   - Fix any test failures before making additional changes
   - Add new tests for any new functionality

2. **Apply Code Quality Checks**: Run linting and formatting after all changes
   ```bash
   # Format code
   cargo fmt --all
   
   # Run Clippy with strict settings
   cargo clippy --all-targets --all-features -- -D warnings
   ```
   - Code must pass all Clippy checks without warnings
   - Formatting must be consistent across the codebase
   - Fix all linting issues before concluding work

**Workflow Summary:**
1. Make code changes
2. `cargo test` (fix failures if any)
3. `cargo fmt --all`
4. `cargo clippy --all-targets --all-features -- -D warnings` (fix issues if any)
5. Final verification: `cargo test` (ensure formatting didn't break anything)

**Non-compliance**: Any code changes that don't follow this workflow may be rejected or require rework.

### Package Development Guidelines

**Independence Principle:**
- Each package should be developable in parallel
- Minimize cross-package dependencies where possible
- Use well-defined interfaces between packages

**Shared Package Usage:**
- All packages should depend on `shared` for core types
- Don't duplicate types or logic that belongs in `shared`
- Extend `shared` types through traits rather than modification when possible