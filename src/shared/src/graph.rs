use crate::task::{Task, TaskId, TaskStatus};
use crate::LuceError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGraph {
    pub tasks: HashMap<TaskId, Task>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TaskGraph {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            tasks: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_task(&mut self, mut task: Task) -> TaskId {
        let task_id = task.id;
        task.updated_at = Utc::now();
        self.tasks.insert(task_id, task);
        self.updated_at = Utc::now();
        task_id
    }

    pub fn get_task(&self, task_id: &TaskId) -> Option<&Task> {
        self.tasks.get(task_id)
    }

    pub fn get_task_mut(&mut self, task_id: &TaskId) -> Option<&mut Task> {
        self.updated_at = Utc::now();
        self.tasks.get_mut(task_id)
    }

    pub fn remove_task(&mut self, task_id: &TaskId) -> Option<Task> {
        if let Some(task) = self.tasks.remove(task_id) {
            for other_task in self.tasks.values_mut() {
                other_task.remove_dependency(*task_id);
                other_task.remove_dependent(*task_id);
            }
            self.updated_at = Utc::now();
            Some(task)
        } else {
            None
        }
    }

    pub fn add_dependency(
        &mut self,
        task_id: TaskId,
        dependency_id: TaskId,
    ) -> Result<(), LuceError> {
        if task_id == dependency_id {
            return Err(LuceError::CircularDependency);
        }

        if !self.tasks.contains_key(&task_id) {
            return Err(LuceError::TaskNotFound { id: task_id });
        }

        if !self.tasks.contains_key(&dependency_id) {
            return Err(LuceError::TaskNotFound { id: dependency_id });
        }

        if self.would_create_cycle(task_id, dependency_id) {
            return Err(LuceError::CircularDependency);
        }

        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.add_dependency(dependency_id);
        }

        if let Some(dependency) = self.tasks.get_mut(&dependency_id) {
            dependency.add_dependent(task_id);
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove_dependency(
        &mut self,
        task_id: TaskId,
        dependency_id: TaskId,
    ) -> Result<(), LuceError> {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.remove_dependency(dependency_id);
        }

        if let Some(dependency) = self.tasks.get_mut(&dependency_id) {
            dependency.remove_dependent(task_id);
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn get_ready_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| {
                matches!(task.status, TaskStatus::Ready)
                    || (matches!(task.status, TaskStatus::Pending)
                        && task.dependencies.iter().all(|dep_id| {
                            self.tasks.get(dep_id).is_some_and(|dep| dep.is_completed())
                        }))
            })
            .collect()
    }

    pub fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.status == status)
            .collect()
    }

    pub fn get_available_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| {
                task.assigned_session.is_none()
                    && match task.status {
                        TaskStatus::Ready => true,
                        TaskStatus::Pending => task.dependencies.iter().all(|dep_id| {
                            self.tasks.get(dep_id).is_some_and(|dep| dep.is_completed())
                        }),
                        _ => false,
                    }
            })
            .collect()
    }

    pub fn get_blocked_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| {
                matches!(task.status, TaskStatus::Pending)
                    && !task
                        .dependencies
                        .iter()
                        .all(|dep_id| self.tasks.get(dep_id).is_some_and(|dep| dep.is_completed()))
            })
            .collect()
    }

    pub fn get_tasks_assigned_to_session(&self, session_id: &str) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.assigned_session.as_deref() == Some(session_id))
            .collect()
    }

    pub fn update_task_readiness(&mut self) {
        let ready_task_ids: Vec<TaskId> =
            self.tasks
                .iter()
                .filter_map(|(id, task)| {
                    if matches!(task.status, TaskStatus::Pending)
                        && task.dependencies.iter().all(|dep_id| {
                            self.tasks.get(dep_id).is_some_and(|dep| dep.is_completed())
                        })
                    {
                        Some(*id)
                    } else {
                        None
                    }
                })
                .collect();

        let has_ready_tasks = !ready_task_ids.is_empty();

        for task_id in ready_task_ids {
            if let Some(task) = self.tasks.get_mut(&task_id) {
                task.set_status(TaskStatus::Ready);
            }
        }

        if has_ready_tasks {
            self.updated_at = Utc::now();
        }
    }

    pub fn complete_task(&mut self, task_id: TaskId) -> Result<Vec<TaskId>, LuceError> {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.set_status(TaskStatus::Completed);
            let dependents = task.dependents.clone();

            self.update_task_readiness();

            let newly_ready: Vec<TaskId> = dependents
                .iter()
                .filter(|dep_id| {
                    self.tasks
                        .get(dep_id)
                        .is_some_and(|t| matches!(t.status, TaskStatus::Ready))
                })
                .copied()
                .collect();

            self.updated_at = Utc::now();
            Ok(newly_ready)
        } else {
            Err(LuceError::TaskNotFound { id: task_id })
        }
    }

    pub fn fail_task(
        &mut self,
        task_id: TaskId,
        block_dependents: bool,
    ) -> Result<Vec<TaskId>, LuceError> {
        if let Some(task) = self.tasks.get_mut(&task_id) {
            task.set_status(TaskStatus::Failed);
            let dependents = task.dependents.clone();

            let mut blocked_tasks = Vec::new();

            if block_dependents {
                for dependent_id in dependents {
                    if let Some(dependent) = self.tasks.get_mut(&dependent_id) {
                        if !dependent.is_terminal() {
                            dependent.set_status(TaskStatus::Blocked);
                            blocked_tasks.push(dependent_id);
                        }
                    }
                }
            }

            self.updated_at = Utc::now();
            Ok(blocked_tasks)
        } else {
            Err(LuceError::TaskNotFound { id: task_id })
        }
    }

    pub fn get_task_dependencies(&self, task_id: &TaskId) -> Result<Vec<&Task>, LuceError> {
        if let Some(task) = self.tasks.get(task_id) {
            Ok(task
                .dependencies
                .iter()
                .filter_map(|dep_id| self.tasks.get(dep_id))
                .collect())
        } else {
            Err(LuceError::TaskNotFound { id: *task_id })
        }
    }

    pub fn get_task_dependents(&self, task_id: &TaskId) -> Result<Vec<&Task>, LuceError> {
        if let Some(task) = self.tasks.get(task_id) {
            Ok(task
                .dependents
                .iter()
                .filter_map(|dep_id| self.tasks.get(dep_id))
                .collect())
        } else {
            Err(LuceError::TaskNotFound { id: *task_id })
        }
    }

    pub fn get_root_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.dependencies.is_empty())
            .collect()
    }

    pub fn get_leaf_tasks(&self) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.dependents.is_empty())
            .collect()
    }

    pub fn topological_sort(&self) -> Result<Vec<&Task>, LuceError> {
        let mut in_degree: HashMap<TaskId, usize> = HashMap::new();
        let mut queue: VecDeque<TaskId> = VecDeque::new();
        let mut result: Vec<&Task> = Vec::new();

        for task in self.tasks.values() {
            in_degree.insert(task.id, task.dependencies.len());
            if task.dependencies.is_empty() {
                queue.push_back(task.id);
            }
        }

        while let Some(task_id) = queue.pop_front() {
            if let Some(task) = self.tasks.get(&task_id) {
                result.push(task);

                for &dependent_id in &task.dependents {
                    if let Some(degree) = in_degree.get_mut(&dependent_id) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push_back(dependent_id);
                        }
                    }
                }
            }
        }

        if result.len() != self.tasks.len() {
            Err(LuceError::CircularDependency)
        } else {
            Ok(result)
        }
    }

    pub fn find_cycles(&self) -> Vec<Vec<TaskId>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();
        let mut cycles = Vec::new();

        for task_id in self.tasks.keys() {
            if !visited.contains(task_id) {
                self.dfs_find_cycles(
                    *task_id,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn dfs_find_cycles(
        &self,
        task_id: TaskId,
        visited: &mut HashSet<TaskId>,
        rec_stack: &mut HashSet<TaskId>,
        path: &mut Vec<TaskId>,
        cycles: &mut Vec<Vec<TaskId>>,
    ) {
        visited.insert(task_id);
        rec_stack.insert(task_id);
        path.push(task_id);

        if let Some(task) = self.tasks.get(&task_id) {
            for &dependent_id in &task.dependents {
                if !visited.contains(&dependent_id) {
                    self.dfs_find_cycles(dependent_id, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(&dependent_id) {
                    if let Some(cycle_start) = path.iter().position(|&id| id == dependent_id) {
                        cycles.push(path[cycle_start..].to_vec());
                    }
                }
            }
        }

        path.pop();
        rec_stack.remove(&task_id);
    }

    fn would_create_cycle(&self, task_id: TaskId, dependency_id: TaskId) -> bool {
        let mut visited = HashSet::new();
        self.has_path_to_dependency(dependency_id, task_id, &mut visited)
    }

    fn has_path_to_dependency(
        &self,
        start: TaskId,
        target: TaskId,
        visited: &mut HashSet<TaskId>,
    ) -> bool {
        if start == target {
            return true;
        }

        if visited.contains(&start) {
            return false;
        }

        visited.insert(start);

        if let Some(task) = self.tasks.get(&start) {
            for &dependency_id in &task.dependencies {
                if self.has_path_to_dependency(dependency_id, target, visited) {
                    return true;
                }
            }
        }

        false
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn completed_task_count(&self) -> usize {
        self.tasks.values().filter(|t| t.is_completed()).count()
    }

    pub fn pending_task_count(&self) -> usize {
        self.tasks.values().filter(|t| t.is_pending()).count()
    }

    pub fn ready_task_count(&self) -> usize {
        self.tasks.values().filter(|t| t.is_ready()).count()
    }

    pub fn in_progress_task_count(&self) -> usize {
        self.tasks.values().filter(|t| t.is_in_progress()).count()
    }

    pub fn failed_task_count(&self) -> usize {
        self.tasks.values().filter(|t| t.is_failed()).count()
    }

    pub fn blocked_task_count(&self) -> usize {
        self.tasks.values().filter(|t| t.is_blocked()).count()
    }

    pub fn assigned_task_count(&self) -> usize {
        self.tasks.values().filter(|t| t.is_assigned()).count()
    }

    pub fn progress_percentage(&self) -> f64 {
        if self.tasks.is_empty() {
            return 100.0;
        }

        let completed = self.completed_task_count() as f64;
        let total = self.tasks.len() as f64;
        (completed / total) * 100.0
    }

    pub fn clear(&mut self) {
        self.tasks.clear();
        self.updated_at = Utc::now();
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn contains_task(&self, task_id: &TaskId) -> bool {
        self.tasks.contains_key(task_id)
    }
}

impl Default for TaskGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{TaskPriority, TaskStatus};
    use uuid::Uuid;

    #[test]
    fn test_task_graph_creation() {
        let graph = TaskGraph::new();
        assert_eq!(graph.task_count(), 0);
        assert_eq!(graph.completed_task_count(), 0);
        assert_eq!(graph.pending_task_count(), 0);
        assert!(graph.is_empty());
        assert_eq!(graph.progress_percentage(), 100.0);
    }

    #[test]
    fn test_task_graph_add_remove() {
        let mut graph = TaskGraph::new();
        let task = Task::new("Test task".to_string());
        let task_id = task.id;

        let added_id = graph.add_task(task);
        assert_eq!(added_id, task_id);
        assert_eq!(graph.task_count(), 1);
        assert!(!graph.is_empty());
        assert!(graph.contains_task(&task_id));

        let removed_task = graph.remove_task(&task_id);
        assert!(removed_task.is_some());
        assert_eq!(graph.task_count(), 0);
        assert!(graph.is_empty());
        assert!(!graph.contains_task(&task_id));
    }

    #[test]
    fn test_dependency_management() {
        let mut graph = TaskGraph::new();

        let task1 = Task::new("Task 1".to_string());
        let task2 = Task::new("Task 2".to_string());
        let task1_id = task1.id;
        let task2_id = task2.id;

        graph.add_task(task1);
        graph.add_task(task2);

        let result = graph.add_dependency(task2_id, task1_id);
        assert!(result.is_ok());

        let task2 = graph.get_task(&task2_id).unwrap();
        assert!(task2.dependencies.contains(&task1_id));

        let task1 = graph.get_task(&task1_id).unwrap();
        assert!(task1.dependents.contains(&task2_id));

        let dependencies = graph.get_task_dependencies(&task2_id).unwrap();
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].id, task1_id);

        let dependents = graph.get_task_dependents(&task1_id).unwrap();
        assert_eq!(dependents.len(), 1);
        assert_eq!(dependents[0].id, task2_id);
    }

    #[test]
    fn test_circular_dependency_prevention() {
        let mut graph = TaskGraph::new();

        let task1 = Task::new("Task 1".to_string());
        let task2 = Task::new("Task 2".to_string());
        let task1_id = task1.id;
        let task2_id = task2.id;

        graph.add_task(task1);
        graph.add_task(task2);

        let result1 = graph.add_dependency(task2_id, task1_id);
        assert!(result1.is_ok());

        let result2 = graph.add_dependency(task1_id, task2_id);
        assert!(matches!(result2, Err(LuceError::CircularDependency)));
    }

    #[test]
    fn test_self_dependency_prevention() {
        let mut graph = TaskGraph::new();

        let task = Task::new("Task".to_string());
        let task_id = task.id;

        graph.add_task(task);

        let result = graph.add_dependency(task_id, task_id);
        assert!(matches!(result, Err(LuceError::CircularDependency)));
    }

    #[test]
    fn test_task_readiness_calculation() {
        let mut graph = TaskGraph::new();

        let task1 = Task::new("Task 1".to_string());
        let task2 = Task::new("Task 2".to_string());
        let task1_id = task1.id;
        let task2_id = task2.id;

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_dependency(task2_id, task1_id).unwrap();

        let ready_tasks = graph.get_ready_tasks();
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, task1_id);

        graph.complete_task(task1_id).unwrap();

        let ready_tasks = graph.get_ready_tasks();
        assert_eq!(ready_tasks.len(), 1);
        assert_eq!(ready_tasks[0].id, task2_id);
    }

    #[test]
    fn test_task_completion_unlocks_dependents() {
        let mut graph = TaskGraph::new();

        let task1 = Task::new("Task 1".to_string());
        let task2 = Task::new("Task 2".to_string());
        let task3 = Task::new("Task 3".to_string());
        let task1_id = task1.id;
        let task2_id = task2.id;
        let task3_id = task3.id;

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_task(task3);

        graph.add_dependency(task2_id, task1_id).unwrap();
        graph.add_dependency(task3_id, task1_id).unwrap();

        let newly_ready = graph.complete_task(task1_id).unwrap();
        assert_eq!(newly_ready.len(), 2);
        assert!(newly_ready.contains(&task2_id));
        assert!(newly_ready.contains(&task3_id));
    }

    #[test]
    fn test_get_available_tasks() {
        let mut graph = TaskGraph::new();

        let mut task1 = Task::new("Available task".to_string());
        task1.set_status(TaskStatus::Ready);

        let mut task2 = Task::new("Assigned task".to_string());
        task2.set_status(TaskStatus::Ready);
        task2.assign_to_session("session-1".to_string());

        let task3 = Task::new("Blocked task".to_string());
        let task4 = Task::new("Dependency task".to_string());
        let task3_id = task3.id;
        let task4_id = task4.id;

        let task1_id = task1.id;

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_task(task3);
        graph.add_task(task4);

        graph.add_dependency(task3_id, task4_id).unwrap();

        let available = graph.get_available_tasks();
        assert_eq!(available.len(), 2);
        let available_ids: Vec<_> = available.iter().map(|t| t.id).collect();
        assert!(available_ids.contains(&task1_id));
        assert!(available_ids.contains(&task4_id));
        assert!(!available_ids.contains(&task3_id));
    }

    #[test]
    fn test_get_blocked_tasks() {
        let mut graph = TaskGraph::new();

        let task1 = Task::new("Task 1".to_string());
        let task2 = Task::new("Task 2".to_string());
        let task1_id = task1.id;
        let task2_id = task2.id;

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_dependency(task2_id, task1_id).unwrap();

        let blocked = graph.get_blocked_tasks();
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].id, task2_id);
    }

    #[test]
    fn test_session_assignment_filtering() {
        let mut graph = TaskGraph::new();

        let mut task1 = Task::new("Task 1".to_string());
        let mut task2 = Task::new("Task 2".to_string());
        let task3 = Task::new("Task 3".to_string());

        task1.assign_to_session("session-1".to_string());
        task2.assign_to_session("session-2".to_string());

        let task1_id = task1.id;
        let task2_id = task2.id;

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_task(task3);

        let session1_tasks = graph.get_tasks_assigned_to_session("session-1");
        assert_eq!(session1_tasks.len(), 1);
        assert_eq!(session1_tasks[0].id, task1_id);

        let session2_tasks = graph.get_tasks_assigned_to_session("session-2");
        assert_eq!(session2_tasks.len(), 1);
        assert_eq!(session2_tasks[0].id, task2_id);

        let session3_tasks = graph.get_tasks_assigned_to_session("session-3");
        assert_eq!(session3_tasks.len(), 0);
    }

    #[test]
    fn test_task_failure_with_blocking() {
        let mut graph = TaskGraph::new();

        let task1 = Task::new("Task 1".to_string());
        let task2 = Task::new("Task 2".to_string());
        let task1_id = task1.id;
        let task2_id = task2.id;

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_dependency(task2_id, task1_id).unwrap();

        let blocked_tasks = graph.fail_task(task1_id, true).unwrap();
        assert_eq!(blocked_tasks.len(), 1);
        assert_eq!(blocked_tasks[0], task2_id);

        let task2 = graph.get_task(&task2_id).unwrap();
        assert!(task2.is_blocked());
    }

    #[test]
    fn test_root_and_leaf_tasks() {
        let mut graph = TaskGraph::new();

        let task1 = Task::new("Root task".to_string());
        let task2 = Task::new("Middle task".to_string());
        let task3 = Task::new("Leaf task".to_string());
        let task1_id = task1.id;
        let task2_id = task2.id;
        let task3_id = task3.id;

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_task(task3);

        graph.add_dependency(task2_id, task1_id).unwrap();
        graph.add_dependency(task3_id, task2_id).unwrap();

        let roots = graph.get_root_tasks();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id, task1_id);

        let leaves = graph.get_leaf_tasks();
        assert_eq!(leaves.len(), 1);
        assert_eq!(leaves[0].id, task3_id);
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = TaskGraph::new();

        let task1 = Task::new("Task 1".to_string());
        let task2 = Task::new("Task 2".to_string());
        let task3 = Task::new("Task 3".to_string());
        let task1_id = task1.id;
        let task2_id = task2.id;
        let task3_id = task3.id;

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_task(task3);

        graph.add_dependency(task2_id, task1_id).unwrap();
        graph.add_dependency(task3_id, task2_id).unwrap();

        let sorted = graph.topological_sort().unwrap();
        assert_eq!(sorted.len(), 3);

        let sorted_ids: Vec<_> = sorted.iter().map(|t| t.id).collect();
        let task1_pos = sorted_ids.iter().position(|&id| id == task1_id).unwrap();
        let task2_pos = sorted_ids.iter().position(|&id| id == task2_id).unwrap();
        let task3_pos = sorted_ids.iter().position(|&id| id == task3_id).unwrap();

        assert!(task1_pos < task2_pos);
        assert!(task2_pos < task3_pos);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = TaskGraph::new();

        let task1 = Task::new("Task 1".to_string());
        let task2 = Task::new("Task 2".to_string());
        let task3 = Task::new("Task 3".to_string());
        let task1_id = task1.id;
        let task2_id = task2.id;
        let task3_id = task3.id;

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_task(task3);

        graph.add_dependency(task2_id, task1_id).unwrap();
        graph.add_dependency(task3_id, task2_id).unwrap();

        let cycles = graph.find_cycles();
        assert!(cycles.is_empty());

        // Force a cycle by directly modifying the graph (bypassing validation)
        if let Some(task1) = graph.tasks.get_mut(&task1_id) {
            task1.add_dependency(task3_id);
        }
        if let Some(task3) = graph.tasks.get_mut(&task3_id) {
            task3.add_dependent(task1_id);
        }

        let cycles = graph.find_cycles();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_task_statistics() {
        let mut graph = TaskGraph::new();

        let mut task1 = Task::new("Completed task".to_string());
        task1.set_status(TaskStatus::Completed);

        let task2 = Task::new("Pending task".to_string());

        let mut task3 = Task::new("Ready task".to_string());
        task3.set_status(TaskStatus::Ready);

        let mut task4 = Task::new("In progress task".to_string());
        task4.set_status(TaskStatus::InProgress);

        let mut task5 = Task::new("Failed task".to_string());
        task5.set_status(TaskStatus::Failed);

        let mut task6 = Task::new("Blocked task".to_string());
        task6.set_status(TaskStatus::Blocked);

        let mut task7 = Task::new("Assigned task".to_string());
        task7.assign_to_session("session-1".to_string());

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_task(task3);
        graph.add_task(task4);
        graph.add_task(task5);
        graph.add_task(task6);
        graph.add_task(task7);

        assert_eq!(graph.task_count(), 7);
        assert_eq!(graph.completed_task_count(), 1);
        assert_eq!(graph.pending_task_count(), 2); // task2 and task7
        assert_eq!(graph.ready_task_count(), 1);
        assert_eq!(graph.in_progress_task_count(), 1);
        assert_eq!(graph.failed_task_count(), 1);
        assert_eq!(graph.blocked_task_count(), 1);
        assert_eq!(graph.assigned_task_count(), 1);

        let progress = graph.progress_percentage();
        assert!((progress - 14.285714285714286).abs() < 0.001); // 1/7 * 100
    }

    #[test]
    fn test_task_graph_clear() {
        let mut graph = TaskGraph::new();

        let task1 = Task::new("Task 1".to_string());
        let task2 = Task::new("Task 2".to_string());

        graph.add_task(task1);
        graph.add_task(task2);

        assert_eq!(graph.task_count(), 2);
        assert!(!graph.is_empty());

        graph.clear();

        assert_eq!(graph.task_count(), 0);
        assert!(graph.is_empty());
        assert_eq!(graph.progress_percentage(), 100.0);
    }

    #[test]
    fn test_task_graph_serialization() {
        let mut graph = TaskGraph::new();

        let task1 = Task::new("Task 1".to_string())
            .with_description("First task".to_string())
            .with_priority(TaskPriority::High);
        let task2 = Task::new("Task 2".to_string());
        let task1_id = task1.id;
        let task2_id = task2.id;

        graph.add_task(task1);
        graph.add_task(task2);
        graph.add_dependency(task2_id, task1_id).unwrap();

        let serialized = serde_json::to_string(&graph).unwrap();
        let deserialized: TaskGraph = serde_json::from_str(&serialized).unwrap();

        assert_eq!(graph.task_count(), deserialized.task_count());
        assert_eq!(
            graph.get_task(&task1_id).unwrap().title,
            deserialized.get_task(&task1_id).unwrap().title
        );
        assert_eq!(
            graph.get_task(&task2_id).unwrap().dependencies,
            deserialized.get_task(&task2_id).unwrap().dependencies
        );
    }

    #[test]
    fn test_nonexistent_task_operations() {
        let mut graph = TaskGraph::new();
        let fake_id1 = Uuid::new_v4();
        let fake_id2 = Uuid::new_v4();

        // Self-dependency is caught as CircularDependency, not TaskNotFound
        assert!(matches!(
            graph.add_dependency(fake_id1, fake_id1),
            Err(LuceError::CircularDependency)
        ));

        // Different IDs should be TaskNotFound
        assert!(matches!(
            graph.add_dependency(fake_id1, fake_id2),
            Err(LuceError::TaskNotFound { .. })
        ));

        assert!(matches!(
            graph.complete_task(fake_id1),
            Err(LuceError::TaskNotFound { .. })
        ));

        assert!(matches!(
            graph.fail_task(fake_id1, false),
            Err(LuceError::TaskNotFound { .. })
        ));

        assert!(matches!(
            graph.get_task_dependencies(&fake_id1),
            Err(LuceError::TaskNotFound { .. })
        ));

        assert!(matches!(
            graph.get_task_dependents(&fake_id1),
            Err(LuceError::TaskNotFound { .. })
        ));
    }
}
