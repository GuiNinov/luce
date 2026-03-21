import { useState, useCallback } from 'react'
import { Task, TaskPriority, TaskEdge } from '@/types/task'

export function useTasks() {
  const [tasks, setTasks] = useState<Task[]>([])
  const [edges, setEdges] = useState<TaskEdge[]>([])

  const addTask = useCallback((taskData: {
    title: string
    description?: string
    priority: TaskPriority
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

    setTasks(prev => [...prev, newTask])
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