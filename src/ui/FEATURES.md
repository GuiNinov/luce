# UI Package Features Plan - MVP

The `ui` package provides a minimal web-based interface for the Luce task management system, built with React, TypeScript, and React Flow. This MVP focuses on core task management functionality with a clean, shadcn/ui design system.

## Overview

The UI package is implemented as a React-based single-page application (SPA) that provides essential task management through two primary views: graph visualization and list view, with basic task creation capabilities.

### Core Technologies
- **React 18**: Modern React with hooks and TypeScript
- **TypeScript**: Type-safe development with full IntelliSense
- **Vite**: Fast development server and build tool
- **Tailwind CSS**: Utility-first CSS framework
- **shadcn/ui**: Beautiful, accessible component library
- **React Flow**: Advanced graph visualization and interaction
- **Radix UI**: Headless UI primitives for accessibility

## Design System

### Shadcn-Inspired Design Principles

The UI follows shadcn/ui design patterns adapted for Yew:

#### Color Palette:
- **Background**: `hsl(0 0% 100%)` / `hsl(222.2 84% 4.9%)` (light/dark)
- **Foreground**: `hsl(222.2 84% 4.9%)` / `hsl(210 40% 98%)` (light/dark)
- **Primary**: `hsl(222.2 47.4% 11.2%)` / `hsl(210 40% 98%)` (light/dark)
- **Muted**: `hsl(210 40% 96%)` / `hsl(217.2 32.6% 17.5%)` (light/dark)
- **Border**: `hsl(214.3 31.8% 91.4%)` / `hsl(217.2 32.6% 17.5%)` (light/dark)

#### Task Status Colors:
- **Pending**: `hsl(210 40% 96%)` (muted background)
- **Ready**: `hsl(142.1 76.2% 36.3%)` (green)
- **InProgress**: `hsl(221.2 83.2% 53.3%)` (blue)
- **Completed**: `hsl(142.1 70.6% 45.3%)` (darker green)
- **Failed**: `hsl(0 84.2% 60.2%)` (red)
- **Blocked**: `hsl(38.5 92.1% 50.2%)` (orange)

#### Typography:
- **Font Family**: `Inter, ui-sans-serif, system-ui`
- **Font Sizes**: Tailwind scale (text-xs to text-4xl)
- **Font Weights**: 400 (normal), 500 (medium), 600 (semibold)

### Component Design Guidelines

All components follow shadcn conventions:
- Clean, minimal borders with subtle shadows
- Consistent border radius (`rounded-md`, `rounded-lg`)
- Hover states with smooth transitions
- Focus states with ring utilities
- Consistent spacing using Tailwind scale

## Core Components

### 1. View Toggle (`view_toggle.rs`)

Simple toggle between graph and list views.

#### Features:
- **Tab-style Toggle**: Clean tab interface switching between views
- **Icon Support**: Graph icon (network) and list icon (list)
- **Active State**: Clear visual indication of current view
- **Keyboard Navigation**: Arrow key navigation between tabs

#### Component Structure:
```rust
// Tabs component with two options
- Graph View Tab
- List View Tab
```

### 2. Graph View (`graph_view.rs`)

Simplified graph visualization for task dependencies.

#### Features:
- **SVG Rendering**: Lightweight vector graphics for nodes and edges
- **Basic Layout**: Simple force-directed positioning
- **Node Interactions**: Click to select, basic hover states
- **Status Visualization**: Color-coded nodes based on task status
- **Zoom Controls**: Simple zoom in/out buttons

#### Visual Elements:
- **Task Nodes**: Rounded rectangles with shadcn styling
- **Dependency Lines**: Simple arrows showing relationships
- **Node Labels**: Task titles with truncation
- **Status Indicators**: Border colors matching task status

### 3. List View (`list_view.rs`)

Clean table/card view of all tasks.

#### Features:
- **Card Layout**: shadcn Card components for each task
- **Status Badges**: Small status indicators with appropriate colors
- **Basic Sorting**: Click column headers to sort
- **Task Selection**: Checkbox selection for individual tasks

#### Card Structure:
```
Task Card
├── Title (heading)
├── Description (muted text)
├── Status Badge
├── Priority Indicator
└── Creation Date
```

### 4. Add Task Interface (`add_task.rs`)

Simple task creation modal/form.

#### Features:
- **Modal Dialog**: shadcn Dialog component for task creation
- **Form Fields**: Title, description, priority selection
- **Add Button**: Prominent action button to trigger modal
- **Form Validation**: Basic required field validation
- **Cancel/Save**: Standard modal actions

#### Form Fields:
- **Title**: Required text input
- **Description**: Optional textarea
- **Priority**: Select dropdown (Low, Normal, High, Critical)

## Reusability Guidelines

### Global Components Architecture

Following shadcn patterns, all root UI components must come from a global `components` folder:

```
src/ui/
├── components/
│   ├── ui/                  # Core shadcn-style components
│   │   ├── button.rs
│   │   ├── card.rs
│   │   ├── dialog.rs
│   │   ├── badge.rs
│   │   ├── input.rs
│   │   ├── select.rs
│   │   ├── textarea.rs
│   │   └── tabs.rs
│   ├── task/                # Task-specific components
│   │   ├── task_node.rs
│   │   ├── task_card.rs
│   │   ├── task_form.rs
│   │   └── status_badge.rs
│   └── layout/              # Layout components
│       ├── header.rs
│       ├── main_content.rs
│       └── view_container.rs
├── views/                   # Page-level components
│   ├── graph_view.rs
│   ├── list_view.rs
│   └── add_task_modal.rs
├── hooks/                   # Custom Yew hooks
│   ├── use_tasks.rs
│   └── use_view_state.rs
└── lib.rs
```

### Component Reusability Rules

#### 1. **UI Components** (`components/ui/`)
- **Purpose**: Foundational, reusable components matching shadcn patterns
- **Dependencies**: Only Yew, no business logic
- **Styling**: Self-contained with consistent shadcn classes
- **Props**: Generic, not tied to specific domains

**Example Button Component:**
```rust
#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    pub children: Children,
    pub variant: Option<ButtonVariant>, // primary, secondary, destructive
    pub size: Option<ButtonSize>,       // sm, md, lg
    pub disabled: Option<bool>,
    pub onclick: Option<Callback<MouseEvent>>,
}

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    // Implementation with shadcn classes
}
```

#### 2. **Domain Components** (`components/task/`, `components/layout/`)
- **Purpose**: Business-specific reusable components
- **Dependencies**: Can use UI components and business logic
- **Composition**: Built from UI components
- **Reusability**: Usable across different views

**Example Task Card:**
```rust
#[derive(Properties, PartialEq)]
pub struct TaskCardProps {
    pub task: Task,
    pub on_select: Option<Callback<Uuid>>,
    pub selected: bool,
}

#[function_component(TaskCard)]
pub fn task_card(props: &TaskCardProps) -> Html {
    // Uses Card, Badge, Button from ui/ components
}
```

#### 3. **View Components** (`views/`)
- **Purpose**: Page-level components that compose smaller components
- **Dependencies**: Domain components, hooks, business logic
- **Composition**: Orchestrate multiple components
- **Specificity**: Tied to specific application views

### Component Import Patterns

All components must follow strict import hierarchy:

```rust
// ✅ Correct: Views import from components
use crate::components::ui::{Button, Card, Dialog};
use crate::components::task::{TaskCard, StatusBadge};
use crate::components::layout::Header;

// ❌ Wrong: Direct component implementation in views
// impl Button { ... } // Should be in components/ui/button.rs

// ❌ Wrong: Circular imports
// UI components importing domain components
```

### Styling Guidelines

#### Component Styling Rules:
1. **UI Components**: Self-contained Tailwind classes, no external dependencies
2. **Consistent Variants**: Use enums for variant props (primary/secondary, sm/md/lg)
3. **Composable Classes**: Allow className prop for additional styling
4. **No Hardcoded Colors**: Use CSS variables or Tailwind semantic classes
5. **Responsive Design**: Mobile-first approach with responsive utilities

#### Example Component with Styling:
```rust
#[derive(Properties, PartialEq)]
pub struct CardProps {
    pub children: Children,
    pub class: Option<AttrValue>,
}

#[function_component(Card)]
pub fn card(props: &CardProps) -> Html {
    let classes = classes!(
        "rounded-lg",
        "border",
        "border-border",
        "bg-card",
        "text-card-foreground",
        "shadow-sm",
        props.class.as_ref().map(|c| c.as_str())
    );
    
    html! {
        <div class={classes}>
            { for props.children.iter() }
        </div>
    }
}
```

### Testing Strategy for Reusable Components

#### Component Testing Requirements:
1. **UI Components**: Visual regression tests and prop validation
2. **Domain Components**: Business logic tests and integration tests
3. **View Components**: End-to-end interaction tests

#### Test Organization:
```
tests/
├── components/
│   ├── ui/
│   │   ├── button_test.rs
│   │   └── card_test.rs
│   └── task/
│       └── task_card_test.rs
└── integration/
    ├── graph_view_test.rs
    └── list_view_test.rs
```

## Technical Implementation

### MVP Component Architecture

```
App (main.rs)
├── View State (Graph/List toggle)
├── Task State Management
└── Add Task Modal State

Layout
├── Header (Title + Add Task Button)
├── View Toggle (Tabs)
└── Main Content (Graph or List View)

Views
├── Graph View (SVG-based)
└── List View (Card-based)

Add Task Modal
├── Task Form
├── Validation
└── Submit/Cancel Actions
```

### State Management (Simplified)

**Global State (Yew Context)**:
- Current task list
- Active view (graph/list)
- Add task modal visibility
- Selected tasks

**Local Component State**:
- Form inputs (add task modal)
- Graph zoom/pan state
- Task selection state

### Development Phases

#### Phase 1: Core Infrastructure
- Yew app with Tailwind setup
- Basic layout and routing
- shadcn-style component foundations

#### Phase 2: View Toggle & List View  
- Tab-style view toggle
- Card-based task list
- Basic task display

#### Phase 3: Add Task Functionality
- Modal dialog for task creation
- Form validation and submission
- Task creation integration

#### Phase 4: Graph View
- SVG-based graph rendering
- Simple node positioning
- Basic interactions

## Integration Requirements

### Shared Package Dependencies
- Task and TaskGraph data structures
- TaskStatus and TaskPriority enums
- Basic CRUD operations

### Future API Integration Points
- Task creation endpoint
- Task list retrieval
- Task updates and status changes

## Success Metrics (MVP)

### Core Functionality
- Toggle between graph and list views
- Create new tasks with title, description, priority
- Display tasks in both views with status colors
- Basic task selection and interaction

### Design Quality
- Consistent shadcn visual design
- Responsive layout on different screen sizes
- Smooth transitions and interactions
- Clean, minimal interface

### Code Quality
- Reusable component architecture
- Proper separation of UI and domain components
- Type-safe component props
- Comprehensive testing coverage

This simplified MVP focuses on the core functionality needed to demonstrate the Luce task management concept while establishing a solid foundation for future enhancements.