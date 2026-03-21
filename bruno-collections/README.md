# Luce API Bruno Collections

This directory contains [Bruno](https://www.usebruno.com/) API collections for testing the Luce API endpoints.

## Getting Started

### 1. Install Bruno
Download and install Bruno from [https://www.usebruno.com/](https://www.usebruno.com/)

### 2. Open the Collection
1. Open Bruno
2. Click "Open Collection"
3. Navigate to this directory: `bruno-collections/luce-api`
4. Select the folder to load the collection

### 3. Start the API Server
Before running the requests, make sure the Luce API server is running:

```bash
# From the project root
cargo run --bin luce-api
```

The server will start on `http://localhost:3000` by default.

### 4. Use the Requests
The collection includes the following requests:

## Available Endpoints

### Core Operations
1. **Health Check** - Verify server is running
2. **Create Task** - Create a new task
3. **List Tasks** - Get all tasks with optional filtering
4. **Get Task** - Retrieve a specific task by ID
5. **Update Task** - Update task properties
6. **Delete Task** - Remove a task
7. **Get Ready Tasks** - Get tasks ready for execution
8. **Mark Task Completed** - Mark a task as completed

### Specialized Requests
9. **Create Task with Dependencies** - Create a task that depends on others
10. **List Completed Tasks** - Filter for completed tasks only
11. **List In Progress Tasks** - Filter for in-progress tasks only

## Environment Variables

The collection uses the `local` environment with these variables:
- `baseUrl`: `http://localhost:3000`
- `apiUrl`: `{{baseUrl}}/api/v1`

You can modify these in the `environments/local.bru` file if your server runs on a different port.

## Request Variables

Some requests use variables like `{{taskId}}`. You'll need to:
1. Run "Create Task" first to get a real task ID
2. Copy the task ID from the response
3. Update the `taskId` variable in requests that need it

## Testing Workflow

Recommended testing sequence:

1. **Health Check** - Verify server connectivity
2. **Create Task** - Create a test task and note the returned ID
3. **List Tasks** - Verify the task appears in the list
4. **Get Task** - Retrieve the specific task using its ID
5. **Update Task** - Modify the task properties
6. **Get Ready Tasks** - See tasks available for execution
7. **Mark Task Completed** - Complete the task
8. **List Completed Tasks** - Verify it appears in completed list
9. **Delete Task** - Clean up (optional)

## API Response Format

### Task Object
```json
{
  "id": "uuid",
  "title": "string",
  "description": "string | null",
  "status": "Pending | InProgress | Completed",
  "dependencies": ["uuid"],
  "created_at": "datetime",
  "updated_at": "datetime"
}
```

### List Response
```json
{
  "tasks": [/* Task objects */],
  "total_count": 0
}
```

### Error Response
```json
{
  "error": "string",
  "details": "string"
}
```

## Notes

- All task IDs are UUIDs
- The API uses JSON for request/response bodies
- Status values are: `Pending`, `InProgress`, `Completed`
- Dependencies create a directed acyclic graph (DAG)
- Circular dependencies are automatically detected and rejected

## Troubleshooting

### Server Not Running
If you get connection errors, ensure the API server is running:
```bash
cargo run --bin luce-api
```

### Invalid UUID Errors
Make sure you're using real task UUIDs from Create Task responses, not the placeholder UUIDs in the templates.

### Database Issues
The API uses SQLite. The database file will be created automatically at `luce.db` in the project root.