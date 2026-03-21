import { useState, useCallback } from 'react'
import { Task, TaskPriority, TaskEdge } from '@/types/task'

export function useTasks() {
  const [tasks, setTasks] = useState<Task[]>([])
  const [edges, setEdges] = useState<TaskEdge[]>([])

  const addTask = useCallback((taskData: {
    title: string
    description?: string
    priority: TaskPriority
    dependencyId?: string
    dependencyType?: 'input' | 'output'
  }) => {
    const newTask: Task = {
      id: crypto.randomUUID(),
      title: taskData.title,
      description: taskData.description,
      status: 'pending',
      priority: taskData.priority,
      dependencies: [],
      created_at: new Date().toISOString(),
    }

    setTasks(prev => {
      const updatedTasks = [...prev, newTask]
      
      // Handle dependency connections
      if (taskData.dependencyId && taskData.dependencyType) {
        if (taskData.dependencyType === 'input') {
          // New task feeds into the existing task
          return updatedTasks.map(task => 
            task.id === taskData.dependencyId
              ? { ...task, dependencies: [...task.dependencies, newTask.id] }
              : task
          )
        } else {
          // New task depends on the existing task
          return updatedTasks.map(task => 
            task.id === newTask.id
              ? { ...task, dependencies: [taskData.dependencyId] }
              : task
          )
        }
      }
      
      return updatedTasks
    })

    // Add edge for graph visualization
    if (taskData.dependencyId && taskData.dependencyType) {
      setEdges(prev => {
        if (taskData.dependencyType === 'input') {
          // New task -> existing task
          return [...prev, { from: newTask.id, to: taskData.dependencyId }]
        } else {
          // Existing task -> new task
          return [...prev, { from: taskData.dependencyId, to: newTask.id }]
        }
      })
    }
  }, [])

  const updateTaskStatus = useCallback((taskId: string, status: Task['status']) => {
    setTasks(prev => 
      prev.map(task => 
        task.id === taskId 
          ? { 
              ...task, 
              status,
              updated_at: new Date().toISOString(),
              completed_at: status === 'completed' ? new Date().toISOString() : undefined
            }
          : task
      )
    )
  }, [])

  const addDependency = useCallback((fromTaskId: string, toTaskId: string) => {
    // Add dependency to task
    setTasks(prev => 
      prev.map(task => 
        task.id === toTaskId
          ? { ...task, dependencies: [...task.dependencies, fromTaskId] }
          : task
      )
    )

    // Add edge
    setEdges(prev => [...prev, { from: fromTaskId, to: toTaskId }])
  }, [])

  return {
    tasks,
    edges,
    addTask,
    updateTaskStatus,
    addDependency,
  }
}