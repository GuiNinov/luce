import { useState, useCallback, useEffect } from 'react'
import { Task, TaskPriority, TaskEdge, TaskStatus, getApiStatus } from '@/types/task'
import { apiService } from '@/services/api'

export function useTasks() {
  const [tasks, setTasks] = useState<Task[]>([])
  const [edges, setEdges] = useState<TaskEdge[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Load tasks on mount
  useEffect(() => {
    loadTasks()
  }, [])

  const loadTasks = useCallback(async () => {
    try {
      setLoading(true)
      setError(null)
      const response = await apiService.listTasks()
      
      // Convert API response to our format and build edges from dependencies
      const apiTasks = response.tasks.map(task => ({
        ...task,
        priority: 'normal' as TaskPriority, // Default priority since API doesn't have it yet
      }))
      
      setTasks(apiTasks)
      
      // Build edges from dependencies
      const newEdges: TaskEdge[] = []
      apiTasks.forEach(task => {
        task.dependencies.forEach(depId => {
          newEdges.push({ from: depId, to: task.id })
        })
      })
      setEdges(newEdges)
      
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load tasks')
      console.error('Failed to load tasks:', err)
    } finally {
      setLoading(false)
    }
  }, [])

  const addTask = useCallback(async (taskData: {
    title: string
    description?: string
    priority: TaskPriority
    dependencyId?: string
    dependencyType?: 'input' | 'output'
  }) => {
    try {
      setError(null)
      
      // Determine dependencies based on connection type
      let dependencies: string[] = []
      if (taskData.dependencyId && taskData.dependencyType) {
        if (taskData.dependencyType === 'output') {
          // New task depends on the existing task
          dependencies = [taskData.dependencyId]
        }
        // For 'input' type, we'll update the existing task's dependencies after creation
      }

      const response = await apiService.createTask({
        title: taskData.title,
        description: taskData.description,
        dependencies,
      })

      const newTask: Task = {
        ...response,
        priority: taskData.priority,
      }

      // Handle 'input' type dependency (new task feeds into existing task)
      if (taskData.dependencyId && taskData.dependencyType === 'input') {
        // Update the existing task to depend on the new task
        const existingTask = tasks.find(t => t.id === taskData.dependencyId)
        if (existingTask) {
          await apiService.updateTask(taskData.dependencyId, {
            // Note: We'd need to update the API to handle dependency updates
            // For now, we'll just reload all tasks
          })
        }
      }

      // Reload tasks to get updated state
      await loadTasks()
      
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create task')
      console.error('Failed to create task:', err)
    }
  }, [tasks, loadTasks])

  const updateTaskStatus = useCallback(async (taskId: string, status: TaskStatus) => {
    try {
      setError(null)
      await apiService.updateTask(taskId, { status })
      
      // Update local state optimistically
      setTasks(prev => 
        prev.map(task => 
          task.id === taskId 
            ? { 
                ...task, 
                status,
                updated_at: new Date().toISOString(),
              }
            : task
        )
      )
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update task')
      console.error('Failed to update task:', err)
      // Reload tasks to revert optimistic update
      loadTasks()
    }
  }, [loadTasks])

  const addDependency = useCallback(async (fromTaskId: string, toTaskId: string) => {
    try {
      setError(null)
      const toTask = tasks.find(t => t.id === toTaskId)
      if (!toTask) return

      const newDependencies = [...toTask.dependencies, fromTaskId]
      
      // Note: API doesn't currently support dependency updates directly
      // We'd need to enhance the API for this functionality
      // For now, we'll just update local state
      setTasks(prev => 
        prev.map(task => 
          task.id === toTaskId
            ? { ...task, dependencies: newDependencies }
            : task
        )
      )

      setEdges(prev => [...prev, { from: fromTaskId, to: toTaskId }])
      
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to add dependency')
      console.error('Failed to add dependency:', err)
    }
  }, [tasks])

  return {
    tasks,
    edges,
    loading,
    error,
    addTask,
    updateTaskStatus,
    addDependency,
    refreshTasks: loadTasks,
  }
}