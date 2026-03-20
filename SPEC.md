# Luce: Parallel Task Manager Specification

## Project Overview

Luce is a graph-based task management system optimized for parallel execution workflows. Unlike traditional todo managers that enforce linear task sequences, Luce treats tasks as nodes in a directed acyclic graph (DAG) where dependencies define execution constraints, not artificial ordering.

The system is designed for knowledge workers who naturally operate multiple parallel work streams, particularly those using AI coding assistants like Claude Code across multiple concurrent sessions.

## Problem Statement

### Current Limitations of Serial Todo Managers

Traditional task management tools suffer from fundamental design assumptions that limit productivity:

1. **Linear Mental Model**: Force users to think in sequences when work is naturally parallel
2. **Artificial Bottlenecks**: Create dependencies where none exist, blocking parallelizable work
3. **Poor Parallel Visibility**: No mechanism to identify what can be done simultaneously
4. **No Coordination**: Multiple execution contexts (people, AI sessions) have no shared coordination
5. **Static Dependencies**: Cannot dynamically unlock new work as prerequisites complete

### The Parallel Work Reality

Modern knowledge work, especially software development, involves:
- Multiple independent feature branches
- Parallel research and investigation threads
- Concurrent code review and testing cycles
- Distributed team members working simultaneously
- AI assistants handling parallel subtasks

## Core Vision

Luce reimagines task management through a **graph-centric, parallel-first approach**:

- **Tasks as Graph Nodes**: Each task is a node with explicit dependency relationships
- **Dynamic Parallel Readiness**: Continuous calculation of which tasks can execute now
- **Multi-Session Coordination**: Multiple Claude Code sessions (or human workers) coordinate through shared state
- **Real-Time Dependency Resolution**: Task completion immediately unlocks dependent work
- **Conflict Prevention**: Resource-aware scheduling prevents parallel sessions from interfering

## Key Innovations

### 1. Parallel Readiness Engine

A core engine that continuously evaluates the task graph to identify:
- Tasks with satisfied dependencies (ready for execution)
- Tasks blocked by specific dependencies
- Critical path analysis for priority scheduling
- Resource conflict detection between parallel tasks

### 2. Session Orchestration Protocol

Coordination mechanism for multiple execution contexts:
- **Task Claiming**: Atomic reservation of tasks to prevent conflicts
- **Progress Reporting**: Real-time status updates across sessions
- **Dependency Signaling**: Completion notifications that unlock dependent tasks
- **Rollback Handling**: Recovery when sessions disconnect or fail

### 3. Dynamic Graph Evolution

The task graph evolves as work progresses:
- **Completion Unlocking**: Finished tasks immediately enable dependents
- **Dynamic Discovery**: New tasks can be added as requirements become clear
- **Dependency Refinement**: Initial rough dependencies can be refined during execution
- **Parallel Branch Creation**: One task completion might spawn multiple parallel paths

### 4. Intelligent Resource Management

Prevention of conflicts between parallel work:
- **File Lock Awareness**: Tasks modifying the same files are serialized
- **Dependency Conflict Detection**: Prevents circular or impossible dependencies
- **Resource Pool Management**: Shared resources (databases, APIs) are coordinated
- **Session Isolation**: Clear boundaries between what different sessions can modify

## Target Use Cases

### Software Development
- **Feature Development**: Multiple developers/AI sessions working on independent features
- **Code Review Cycles**: Parallel review, testing, and documentation tasks
- **Deployment Pipelines**: Concurrent staging, testing, and production tasks
- **Bug Investigation**: Parallel debugging threads across different components

### Research Projects
- **Literature Review**: Parallel investigation of different research areas
- **Data Analysis**: Concurrent processing of different datasets
- **Experiment Design**: Parallel development of different experimental approaches
- **Report Writing**: Simultaneous work on different report sections

### Content Creation
- **Multi-Platform Content**: Parallel creation for different channels
- **Research and Writing**: Simultaneous fact-checking and content development
- **Review Cycles**: Parallel editing, fact-checking, and formatting
- **Asset Creation**: Concurrent development of text, images, and multimedia

## Technical Architecture Goals

### Core System Properties
1. **High Concurrency**: Support for many parallel execution contexts
2. **Low Latency**: Near real-time dependency resolution and task unlocking
3. **Consistency**: Strong consistency guarantees for task state and dependencies
4. **Fault Tolerance**: Graceful handling of session failures and network issues
5. **Scalability**: Efficient performance with large task graphs (1000+ tasks)

### Integration Requirements
- **Claude Code Integration**: Native support for multiple Claude sessions
- **File System Awareness**: Integration with file locks and git workflows
- **API Design**: Clean APIs for programmatic task management
- **Cross-Platform**: Support for different development environments

### User Experience Goals
- **Visual Graph Interface**: Intuitive visualization of task dependencies and parallel execution
- **Minimal Friction**: Easy task creation and dependency specification
- **Real-Time Feedback**: Immediate visibility into what can be done now
- **Smart Suggestions**: AI-powered dependency and parallel opportunity detection

## Success Metrics

### Productivity Metrics
- **Parallel Efficiency**: Percentage of available parallel work actually executed simultaneously
- **Dependency Wait Time**: Time tasks spend blocked waiting for dependencies
- **Session Utilization**: How effectively multiple Claude sessions are kept busy
- **Critical Path Optimization**: Reduction in overall project completion time

### System Performance
- **Task Graph Performance**: Response time for dependency calculations on large graphs
- **Session Coordination Latency**: Time between task completion and dependent task availability
- **Conflict Resolution**: Effectiveness at preventing and resolving resource conflicts
- **System Reliability**: Uptime and consistency during multi-session operation

## Success Definition

Luce succeeds when it fundamentally changes how users approach complex, multi-threaded work. Instead of thinking "what should I do next?", users think "what can be done in parallel right now?" This mental model shift, supported by the technical capabilities, should result in measurably faster project completion and higher utilization of available execution contexts.

The ultimate validation is when users cannot imagine returning to linear task management for complex, parallelizable work.