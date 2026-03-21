export type TaskStatus = 'Pending' | 'InProgress' | 'Completed'

export type TaskPriority = 'low' | 'normal' | 'high' | 'critical'

export interface Task {
  id: string
  title: string
  description?: string
  status: TaskStatus
  priority?: TaskPriority // Optional since API doesn't have priority yet
  dependencies: string[]
  created_at: string
  updated_at: string
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

// Helper function to convert API status to lowercase for UI components that expect it
export function getStatusKey(status: TaskStatus): string {
  return status.toLowerCase().replace('inprogress', 'in-progress')
}

// Helper function to convert UI status back to API format
export function getApiStatus(status: string): TaskStatus {
  switch (status) {
    case 'pending': return 'Pending'
    case 'in-progress': return 'InProgress'
    case 'completed': return 'Completed'
    default: return 'Pending'
  }
}