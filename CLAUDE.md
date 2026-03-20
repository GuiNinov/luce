# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Repository Overview

Luce is a graph-based task management system optimized for parallel execution workflows. It enables multiple Claude Code sessions to coordinate on complex, multi-threaded work by treating tasks as nodes in a directed acyclic graph (DAG) where dependencies define execution constraints, not artificial ordering.

**Key Documents**:
- `SPEC.md` - Complete project specification and vision
- `PLAN.md` - Implementation roadmap and technical decisions

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
- `shared/src/task.rs` - Core Task and TaskGraph data structures
- `core/src/graph.rs` - Dependency resolution and parallel readiness calculation
- `mcp/src/server.rs` - Claude Code integration and session coordination