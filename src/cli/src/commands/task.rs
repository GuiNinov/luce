use crate::services::LuceService;
use crate::TaskCommands;
use luce_shared::{TaskPriority, TaskStatus};
use luce_core::TaskFilter;

pub async fn handle_task_command(cmd: TaskCommands, service: &LuceService) -> anyhow::Result<()> {
    match cmd {
        TaskCommands::Create {
            title,
            description,
            priority,
            dependencies,
            metadata,
        } => {
            create_task(
                title,
                description,
                priority,
                dependencies,
                metadata,
                service,
            )
            .await
        }
        TaskCommands::List {
            status,
            session,
            priority,
            available,
            blocked,
            limit,
        } => {
            list_tasks(
                status, session, priority, available, blocked, limit, service,
            )
            .await
        }
        TaskCommands::Show { task_id } => show_task(task_id, service).await,
        TaskCommands::Update {
            task_id,
            title,
            description,
            priority,
            status,
        } => update_task(task_id, title, description, priority, status, service).await,
        TaskCommands::Start { task_id, session } => start_task(task_id, session, service).await,
        TaskCommands::Complete { task_id } => complete_task(task_id, service).await,
        TaskCommands::Fail {
            task_id,
            block_dependents,
        } => fail_task(task_id, block_dependents, service).await,
        TaskCommands::AddDependency {
            task_id,
            dependency_id,
        } => add_dependency(task_id, dependency_id, service).await,
        TaskCommands::RemoveDependency {
            task_id,
            dependency_id,
        } => remove_dependency(task_id, dependency_id, service).await,
        TaskCommands::Assign {
            task_id,
            session_id,
        } => assign_task(task_id, session_id, service).await,
        TaskCommands::Unassign { task_id } => unassign_task(task_id, service).await,
        TaskCommands::Delete { task_id, force } => delete_task(task_id, force, service).await,
        TaskCommands::AddMetadata {
            task_id,
            key,
            value,
        } => add_metadata(task_id, key, value, service).await,
        TaskCommands::RemoveMetadata { task_id, key } => {
            remove_metadata(task_id, key, service).await
        }
    }
}

async fn create_task(
    title: String,
    description: Option<String>,
    priority: String,
    dependencies: Option<String>,
    metadata: Vec<String>,
    service: &LuceService,
) -> anyhow::Result<()> {
    // Parse priority
    let task_priority = match priority.to_lowercase().as_str() {
        "low" => TaskPriority::Low,
        "normal" => TaskPriority::Normal,
        "high" => TaskPriority::High,
        "critical" => TaskPriority::Critical,
        _ => {
            eprintln!(
                "Invalid priority '{}'. Must be one of: low, normal, high, critical",
                priority
            );
            return Ok(());
        }
    };

    // Create task via service
    let mut task = service
        .create_task(title.clone(), description.clone(), Some(task_priority))
        .await?;

    // Parse metadata and apply it if provided
    for meta in metadata {
        if let Some((key, value)) = meta.split_once('=') {
            task = task.with_metadata(key.to_string(), value.to_string());
        } else {
            eprintln!("Invalid metadata format '{}'. Use key=value format", meta);
        }
    }

    println!("Task created successfully!");
    println!("ID: {}", task.id);
    println!("Title: {}", task.title);
    if let Some(desc) = &task.description {
        println!("Description: {}", desc);
    }
    println!("Priority: {:?}", task.priority);
    println!("Status: {:?}", task.status);

    if !task.metadata.is_empty() {
        println!("Metadata:");
        for (key, value) in &task.metadata {
            println!("  {}: {}", key, value);
        }
    }

    // TODO: Parse and add dependencies when graph management is connected
    if let Some(deps) = dependencies {
        println!("Dependencies to add: {}", deps);
        println!("Note: Dependency management will be implemented when core package is connected");
    }

    Ok(())
}

async fn list_tasks(
    status: Option<String>,
    session: Option<String>,
    priority: Option<String>,
    available: bool,
    blocked: bool,
    limit: Option<usize>,
    service: &LuceService,
) -> anyhow::Result<()> {
    // Build filter - the current TaskFilter is simpler than expected, so we'll filter on the service side for now
    let filter = if let Some(status_str) = status {
        match status_str.to_lowercase().as_str() {
            "pending" => Some(TaskFilter::ByStatus(TaskStatus::Pending)),
            "ready" => Some(TaskFilter::ByStatus(TaskStatus::Ready)),
            "in-progress" | "inprogress" => Some(TaskFilter::ByStatus(TaskStatus::InProgress)),
            "completed" => Some(TaskFilter::ByStatus(TaskStatus::Completed)),
            "failed" => Some(TaskFilter::ByStatus(TaskStatus::Failed)),
            "blocked" => Some(TaskFilter::ByStatus(TaskStatus::Blocked)),
            _ => {
                eprintln!("Invalid status '{}'. Must be one of: pending, ready, in-progress, completed, failed, blocked", status_str);
                return Ok(());
            }
        }
    } else if let Some(session_id) = session {
        Some(TaskFilter::BySession(session_id))
    } else if available {
        Some(TaskFilter::Unassigned)
    } else {
        Some(TaskFilter::All)
    };

    // Note: Priority filtering, blocking status, and limit are not currently supported by the core TaskFilter.
    // These features will need to be implemented in the future.
    if priority.is_some() {
        eprintln!("Warning: Priority filtering not yet implemented in core");
    }
    if blocked {
        eprintln!("Warning: Blocked status filtering not yet implemented in core");
    }
    if limit.is_some() {
        eprintln!("Warning: Result limiting not yet implemented in core");
    }

    // Fetch tasks
    match service.list_tasks(filter).await {
        Ok(tasks) => {
            if tasks.is_empty() {
                println!("No tasks found matching the specified criteria.");
            } else {
                println!("Found {} task(s):\n", tasks.len());

                for task in tasks {
                    println!("ID: {}", task.id);
                    println!("Title: {}", task.title);
                    println!("Status: {:?}", task.status);
                    println!("Priority: {:?}", task.priority);

                    if let Some(session) = &task.assigned_session {
                        println!("Session: {}", session);
                    }

                    if let Some(desc) = &task.description {
                        println!("Description: {}", desc);
                    }

                    println!("Created: {}", task.created_at);
                    println!("Updated: {}", task.updated_at);
                    println!("{}", "-".repeat(40));
                }
            }
        }
        Err(e) => {
            eprintln!("Error listing tasks: {}", e);
        }
    }

    Ok(())
}

async fn show_task(task_id: String, service: &LuceService) -> anyhow::Result<()> {
    match service.get_task(&task_id).await {
        Ok(task) => {
            println!("Task Details:");
            println!("ID: {}", task.id);
            println!("Title: {}", task.title);

            if let Some(desc) = &task.description {
                println!("Description: {}", desc);
            }

            println!("Priority: {:?}", task.priority);
            println!("Status: {:?}", task.status);

            if let Some(session) = &task.assigned_session {
                println!("Assigned Session: {}", session);
            }

            if !task.metadata.is_empty() {
                println!("Metadata:");
                for (key, value) in &task.metadata {
                    println!("  {}: {}", key, value);
                }
            }

            println!("Created: {}", task.created_at);
            println!("Updated: {}", task.updated_at);
        }
        Err(e) => {
            eprintln!("Error retrieving task: {}", e);
        }
    }

    Ok(())
}

async fn update_task(
    task_id: String,
    title: Option<String>,
    description: Option<String>,
    priority: Option<String>,
    status: Option<String>,
    service: &LuceService,
) -> anyhow::Result<()> {
    // TODO: Connect to core package for actual implementation
    println!("Updating task: {}", task_id);

    if let Some(t) = title {
        println!("  New title: {}", t);
    }
    if let Some(d) = description {
        println!("  New description: {}", d);
    }
    if let Some(p) = priority {
        println!("  New priority: {}", p);
    }
    if let Some(s) = status {
        println!("  New status: {}", s);
    }

    println!("Note: Task updates will be implemented when core package is connected");

    Ok(())
}

async fn start_task(
    task_id: String,
    session: Option<String>,
    service: &LuceService,
) -> anyhow::Result<()> {
    // Update task status to in-progress
    match service
        .update_task_status(&task_id, TaskStatus::InProgress)
        .await
    {
        Ok(_) => {
            println!("Task {} started (status updated to in-progress)!", task_id);

            // Assign to session if provided
            if let Some(session_id) = session {
                match service
                    .assign_task_to_session(&task_id, session_id.clone())
                    .await
                {
                    Ok(_) => {
                        println!("Task assigned to session: {}", session_id);
                    }
                    Err(e) => {
                        eprintln!("Warning: Task started but session assignment failed: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error starting task: {}", e);
        }
    }

    Ok(())
}

async fn complete_task(task_id: String, service: &LuceService) -> anyhow::Result<()> {
    match service
        .update_task_status(&task_id, TaskStatus::Completed)
        .await
    {
        Ok(_) => {
            println!("Task {} marked as completed successfully!", task_id);
        }
        Err(e) => {
            eprintln!("Error completing task: {}", e);
        }
    }

    Ok(())
}

async fn fail_task(
    task_id: String,
    block_dependents: bool,
    service: &LuceService,
) -> anyhow::Result<()> {
    match service
        .update_task_status(&task_id, TaskStatus::Failed)
        .await
    {
        Ok(_) => {
            println!("Task {} marked as failed!", task_id);

            if block_dependents {
                println!("Note: Dependent task blocking will be implemented when graph management is connected");
            }
        }
        Err(e) => {
            eprintln!("Error marking task as failed: {}", e);
        }
    }

    Ok(())
}

async fn add_dependency(
    task_id: String,
    dependency_id: String,
    service: &LuceService,
) -> anyhow::Result<()> {
    // TODO: Connect to core package for actual implementation
    println!(
        "Adding dependency: {} depends on {}",
        task_id, dependency_id
    );
    println!("Note: Dependency management will be implemented when core package is connected");

    Ok(())
}

async fn remove_dependency(
    task_id: String,
    dependency_id: String,
    service: &LuceService,
) -> anyhow::Result<()> {
    // TODO: Connect to core package for actual implementation
    println!(
        "Removing dependency: {} no longer depends on {}",
        task_id, dependency_id
    );
    println!("Note: Dependency management will be implemented when core package is connected");

    Ok(())
}

async fn assign_task(
    task_id: String,
    session_id: String,
    service: &LuceService,
) -> anyhow::Result<()> {
    match service
        .assign_task_to_session(&task_id, session_id.clone())
        .await
    {
        Ok(_) => {
            println!(
                "Task {} assigned to session {} successfully!",
                task_id, session_id
            );
        }
        Err(e) => {
            eprintln!("Error assigning task: {}", e);
        }
    }

    Ok(())
}

async fn unassign_task(task_id: String, service: &LuceService) -> anyhow::Result<()> {
    match service.unassign_task(&task_id).await {
        Ok(_) => {
            println!("Task {} unassigned successfully!", task_id);
        }
        Err(e) => {
            eprintln!("Error unassigning task: {}", e);
        }
    }

    Ok(())
}

async fn delete_task(task_id: String, force: bool, service: &LuceService) -> anyhow::Result<()> {
    // TODO: Connect to core package for actual implementation
    println!("Deleting task: {}", task_id);

    if force {
        println!("  Force deletion enabled");
    }

    println!("Note: Task deletion will be implemented when core package is connected");

    Ok(())
}

async fn add_metadata(
    task_id: String,
    key: String,
    value: String,
    service: &LuceService,
) -> anyhow::Result<()> {
    // TODO: Connect to core package for actual implementation
    println!("Adding metadata to task {}: {}={}", task_id, key, value);
    println!("Note: Metadata management will be implemented when core package is connected");

    Ok(())
}

async fn remove_metadata(
    task_id: String,
    key: String,
    service: &LuceService,
) -> anyhow::Result<()> {
    // TODO: Connect to core package for actual implementation
    println!("Removing metadata from task {}: {}", task_id, key);
    println!("Note: Metadata management will be implemented when core package is connected");

    Ok(())
}
