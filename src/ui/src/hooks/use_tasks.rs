use yew::prelude::*;
use luce_shared::task::{Task, TaskPriority};

#[hook]
pub fn use_tasks() -> (Vec<Task>, Callback<Task>) {
    let tasks = use_state(|| {
        // Mock data for development
        vec![
            Task::new("Implement UI components".to_string())
                .with_description("Create shadcn-style components for the interface".to_string())
                .with_priority(TaskPriority::High),
            Task::new("Add graph visualization".to_string())
                .with_description("Build SVG-based graph view for task dependencies".to_string())
                .with_priority(TaskPriority::Normal),
            Task::new("Setup routing".to_string())
                .with_description("Configure navigation between different views".to_string())
                .with_priority(TaskPriority::Low),
        ]
    });
    
    let add_task = {
        let tasks = tasks.clone();
        Callback::from(move |new_task: Task| {
            let mut current_tasks = (*tasks).clone();
            current_tasks.push(new_task);
            tasks.set(current_tasks);
        })
    };

    ((*tasks).clone(), add_task)
}