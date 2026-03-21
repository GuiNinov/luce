export type TaskStatus = 'pending' | 'ready' | 'in-progress' | 'completed' | 'failed' | 'blocked'

export type TaskPriority = 'low' | 'normal' | 'high' | 'critical'

export interface Task {
  id: string
  title: string
  description?: string
  status: TaskStatus
  priority: TaskPriority
  dependencies: string[]
  created_at: string
  updated_at?: string
  completed_at?: string
}

export interface TaskGraph {
  tasks: Task[]
  edges: TaskEdge[]
}

export interface TaskEdge {
  from: string
  to: string
}

export type ViewMode = 'graph' | 'list'