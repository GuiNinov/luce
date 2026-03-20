# Shared Package Features

The `shared` package provides core data structures and types that form the foundation of the Luce task management system. This package implements the fundamental building blocks for task representation and graph-based dependency management.

## Overview

The shared package contains two primary modules:

- **`task.rs`** - Individual task representation and management
- **`graph.rs`** - Task graph operations and dependency resolution
- **`error.rs`** - Common error types used across the system

## Core Data Structures

### Task (`task.rs`)

The `Task` struct represents an individual work item in the Luce system.

#### Key Features:
- **Unique Identity**: UUID-based task identification
- **Status Tracking**: Comprehensive status lifecycle (Pending → Ready → InProgress → Completed/Failed)
- **Priority Management**: Four-tier priority system (Low, Normal, High, Critical)
- **Dependency Relationships**: Bidirectional dependency tracking (dependencies and dependents)
- **Session Assignment**: Support for multi-session coordination
- **Metadata Storage**: Flexible key-value metadata system
- **Timestamp Tracking**: Creation, update, start, and completion timestamps
- **Duration Calculations**: Built-in methods for measuring task execution time

#### Builder Pattern Support:
```rust
let task = Task::new("Implement feature X".to_string())
    .with_description("Add new functionality to the system".to_string())
    .with_priority(TaskPriority::High)
    .with_metadata("category".to_string(), "development".to_string());
```

#### Status States:
- `Pending` - Task created but dependencies not met
- `Ready` - All dependencies completed, ready for execution
- `InProgress` - Currently being worked on
- `Completed` - Successfully finished
- `Failed` - Execution failed
- `Blocked` - Explicitly blocked (e.g., due to failed dependencies)

### TaskGraph (`graph.rs`)

The `TaskGraph` struct manages collections of tasks with their dependency relationships.

#### Key Features:
- **Dependency Management**: Add/remove dependencies with automatic cycle detection
- **Parallel Readiness**: Calculate which tasks can execute simultaneously
- **Dynamic Unlocking**: Automatically promote tasks to Ready when dependencies complete
- **Session Coordination**: Filter and assign tasks based on execution sessions
- **Graph Analysis**: Topological sorting, cycle detection, root/leaf identification
- **Progress Tracking**: Comprehensive statistics and progress calculation
- **Failure Propagation**: Optional blocking of dependent tasks when dependencies fail

#### Advanced Operations:
- **Topological Sort**: Get execution order respecting all dependencies
- **Cycle Detection**: Find circular dependencies in the graph
- **Root/Leaf Tasks**: Identify tasks with no dependencies/dependents
- **Blocked Task Analysis**: Find tasks waiting on incomplete dependencies
- **Session Management**: Track which tasks are assigned to specific sessions

## Error Handling

The `LuceError` enum provides comprehensive error handling:

- `TaskNotFound` - Operations on non-existent tasks
- `CircularDependency` - Prevention of dependency cycles
- `InvalidStateTransition` - Illegal status changes
- `DependencyError` - General dependency-related errors
- `SerializationError` - JSON serialization failures
- `IoError` - File system operations

## Serialization Support

Both `Task` and `TaskGraph` fully support JSON serialization via `serde`, enabling:
- Persistence to disk
- Network transmission
- State snapshots
- Cross-session coordination

## Testing Coverage

The shared package includes **31 comprehensive unit tests** covering:

### Task Tests (15 tests):
- Creation patterns and builder methods
- Status lifecycle and timestamp management
- Session assignment and metadata operations
- Dependency relationship management
- Duration calculations and serialization
- Edge cases and error conditions

### TaskGraph Tests (16 tests):
- Graph construction and modification
- Dependency management with cycle prevention
- Task readiness and availability calculation
- Session-based filtering and assignment
- Failure handling and propagation
- Advanced graph algorithms (topological sort, cycle detection)
- Statistics and progress tracking
- Serialization and error handling

## Performance Characteristics

- **Task Creation**: O(1) with UUID generation
- **Dependency Addition**: O(V) for cycle detection (where V = number of tasks)
- **Readiness Calculation**: O(V + E) where E = number of dependencies
- **Task Completion**: O(D) where D = number of dependents
- **Topological Sort**: O(V + E) using Kahn's algorithm

## Thread Safety

The data structures are designed to be:
- **Send**: Can be transferred between threads
- **Serializable**: Full serde support for persistence and transmission
- **Cloneable**: Deep copying for multi-threaded scenarios

Note: Thread safety for concurrent modification requires external synchronization (e.g., `Arc<Mutex<TaskGraph>>` or similar).

## Integration Points

The shared package serves as the foundation for:
- **Core Package**: Graph algorithms and parallel execution engine
- **API Package**: REST endpoints and WebSocket communication  
- **CLI Package**: Command-line task management interface
- **MCP Package**: Claude Code integration and session coordination
- **UI Package**: Web-based visualization and interaction

## Future Enhancements

Potential areas for expansion:
- **Custom Priority Types**: User-defined priority schemes
- **Task Templates**: Reusable task patterns
- **Batch Operations**: Efficient bulk task operations
- **Event System**: Task state change notifications
- **Persistence Adapters**: Direct database integration
- **Task Hierarchies**: Parent-child task relationships