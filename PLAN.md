# Luce Implementation Plan

## Project Structure Overview

Luce will be implemented as a multi-package Rust workspace optimized for LLM-driven development. The modular architecture allows different components to be developed and tested independently while maintaining clear separation of concerns.

## Workspace Structure

```
luce/
├── Cargo.toml              # Workspace root
├── src/
│   ├── shared/             # Common structs and traits
│   ├── core/               # Core graph task management engine
│   ├── cli/                # Command-line interface
│   ├── api/                # REST API server
│   ├── mcp/                # Model Context Protocol server
│   └── ui/                 # Yew-based web frontend
├── SPEC.md                 # Project specification
├── PLAN.md                # Implementation plan (this file)
└── CLAUDE.md              # Claude Code guidance
```

## Package Responsibilities

### `shared` - Common Foundation
**Purpose**: Core data structures, traits, and utilities used across all packages

**Key Components**:
- Task and TaskGraph data structures
- Dependency and status enums
- Session management types
- Error handling and result types
- Serialization/deserialization traits
- Database schema definitions

**Dependencies**: Minimal (serde, uuid, chrono)

### `core` - Graph Engine
**Purpose**: Core task management logic and graph operations

**Key Components**:
- TaskGraph implementation with parallel readiness calculation
- Dependency resolution engine
- Session coordination protocol
- Persistence layer (SQLite integration)
- Event system for real-time updates
- Conflict detection and resolution

**Dependencies**: shared, sqlx, tokio, petgraph

### `cli` - Command Line Interface
**Purpose**: Developer-friendly CLI for task management

**Key Components**:
- Task creation and modification commands
- Graph visualization (ASCII/Unicode)
- Session management operations
- Development workflow integration
- Configuration management
- Interactive task browser

**Dependencies**: shared, core, clap, crossterm, tui-rs

### `api` - REST Server
**Purpose**: HTTP API for external integrations and web frontend

**Key Components**:
- RESTful endpoints for all task operations
- WebSocket support for real-time updates
- Authentication and session management
- API documentation (OpenAPI/Swagger)
- Rate limiting and error handling
- CORS configuration for web frontend

**Dependencies**: shared, core, axum, tower, serde_json

### `mcp` - Model Context Protocol Server
**Purpose**: Native integration with Claude Code and other AI systems

**Key Components**:
- MCP server implementation for Claude Code integration
- Tool definitions for task operations
- Session coordination with multiple Claude instances
- File system integration and conflict detection
- Git workflow awareness
- Progress reporting and state synchronization

**Dependencies**: shared, core, mcp-sdk, tokio

### `ui` - Web Frontend
**Purpose**: Interactive graph visualization and task management interface

**Key Components**:
- Interactive task graph visualization (using D3.js-like capabilities)
- Real-time updates via WebSocket
- Task creation and editing interface
- Session monitoring dashboard
- Parallel execution timeline view
- Dependency relationship editor

**Dependencies**: shared, yew, web-sys, wasm-bindgen, gloo

## Development Phases

### Phase 1: Foundation (shared + core)
**Goal**: Establish core data structures and graph engine

**Deliverables**:
- Task and TaskGraph data structures
- Basic dependency resolution
- SQLite persistence layer
- Unit tests for core functionality
- Simple integration test suite

**Success Criteria**:
- Can create tasks with dependencies
- Parallel readiness calculation works correctly
- Basic persistence operations function
- Core engine handles 1000+ task graphs efficiently

### Phase 2: CLI Interface
**Goal**: Provide developer-friendly command-line access

**Deliverables**:
- Complete CLI with all task operations
- ASCII graph visualization
- Configuration management
- Interactive task browser
- Integration with existing development workflows

**Success Criteria**:
- CLI can manage complex task graphs
- Intuitive commands for common operations
- Good performance with large graphs
- Clear error messages and help text

### Phase 3: Multi-Session Coordination (MCP)
**Goal**: Enable multiple Claude Code sessions to coordinate

**Deliverables**:
- MCP server with full task operation support
- Session registration and coordination
- Atomic task claiming mechanism
- File system conflict detection
- Real-time state synchronization

**Success Criteria**:
- Multiple Claude sessions can work simultaneously
- No race conditions in task claiming
- Proper rollback when sessions disconnect
- File conflicts are detected and prevented

### Phase 4: Web API and Frontend
**Goal**: Provide web-based interface and external integrations

**Deliverables**:
- Complete REST API with WebSocket support
- Interactive web frontend with graph visualization
- Real-time updates across all interfaces
- API documentation and examples

**Success Criteria**:
- Web interface provides full functionality
- Real-time updates work reliably
- Good performance with complex graphs
- API is easy to integrate with external tools

## Key Design Decisions

### Database Strategy
- **Local-First**: SQLite for primary storage, enabling offline operation
- **Future Distributed**: Architecture supports future PostgreSQL/distributed options
- **Event Sourcing**: Consider event log for state changes to enable replay/debugging

### Concurrency Model
- **Async-First**: Tokio-based async runtime throughout
- **Lock-Free**: Minimize locks through message passing and atomic operations
- **Session Isolation**: Clear ownership boundaries to prevent conflicts

### Error Handling
- **Structured Errors**: Custom error types with context
- **Graceful Degradation**: System remains functional when components fail
- **Recovery Mechanisms**: Automatic recovery from common failure scenarios

### Integration Philosophy
- **MCP-Native**: Designed specifically for Claude Code integration
- **API-Extensible**: Clean APIs for future integrations
- **Tool-Agnostic**: Core engine independent of specific tools

## Development Workflow

### LLM-Driven Development Approach
1. **Package-by-Package**: Develop each package with dedicated Claude sessions
2. **Test-First**: Write comprehensive tests before implementation
3. **Documentation-Driven**: Maintain up-to-date docs throughout development
4. **Integration Points**: Define clear interfaces between packages early

### Quality Assurance
- **Unit Tests**: Comprehensive test coverage for all packages
- **Integration Tests**: End-to-end scenarios across package boundaries
- **Performance Tests**: Benchmarks for graph operations and API response times
- **Concurrency Tests**: Race condition detection and stress testing

### Continuous Integration
- **Automated Testing**: Run full test suite on all changes
- **Performance Regression**: Detect performance degradation early
- **Documentation Updates**: Ensure docs stay current with code changes
- **Security Scanning**: Regular dependency vulnerability scans

## Success Metrics

### Development Velocity
- **Package Independence**: Ability to develop packages in parallel
- **Clear Interfaces**: Minimal integration friction between packages
- **Test Coverage**: >90% test coverage across all packages
- **Documentation Quality**: Up-to-date docs for all public interfaces

### System Performance
- **Graph Operations**: <100ms for readiness calculation on 1000 task graphs
- **API Response Time**: <50ms for typical task operations
- **Real-Time Updates**: <200ms latency for state synchronization
- **Memory Efficiency**: Reasonable memory usage with large graphs

### Integration Success
- **Claude Code Integration**: Seamless multi-session coordination
- **Web Interface**: Responsive and intuitive graph visualization
- **API Adoption**: External tools can easily integrate
- **Developer Experience**: CLI provides excellent development workflow

## Risk Mitigation

### Technical Risks
- **Graph Complexity**: Start with simple algorithms, optimize later
- **Concurrency Bugs**: Extensive testing and formal verification where needed
- **Performance**: Profile early and often, benchmark against requirements
- **Integration Challenges**: Prototype integrations early in development

### Project Risks
- **Scope Creep**: Stick to core functionality, defer advanced features
- **Over-Engineering**: Build minimum viable solution first
- **Documentation Drift**: Keep docs updated as part of development process
- **Testing Debt**: Maintain high test coverage from the beginning

This plan provides a roadmap for building Luce as a production-ready parallel task management system while maintaining the flexibility for LLM-driven development and future enhancements.