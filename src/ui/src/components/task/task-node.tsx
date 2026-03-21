import { Handle, Position } from 'reactflow'
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { StatusBadge } from "./status-badge"
import { Task, TaskStatus } from "@/types/task"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

interface TaskNodeProps {
  data: {
    task: Task
    selected: boolean
    onStatusChange?: (taskId: string, status: TaskStatus) => void
  }
}

export function TaskNode({ data }: TaskNodeProps) {
  const { task, selected, onStatusChange } = data

  const getStatusBorderColor = (status: string) => {
    switch (status) {
      case 'completed':
        return 'border-task-completed'
      case 'in-progress':
        return 'border-task-in-progress'
      case 'ready':
        return 'border-task-ready'
      case 'failed':
        return 'border-task-failed'
      case 'blocked':
        return 'border-task-blocked'
      default:
        return 'border-task-pending'
    }
  }

  const handleStatusChange = (newStatus: string) => {
    onStatusChange?.(task.id, newStatus as TaskStatus)
  }

  return (
    <div className={`min-w-[220px] max-w-[320px] ${selected ? 'ring-2 ring-primary' : ''}`}>
      {/* Input handles on the left */}
      <Handle 
        type="target" 
        position={Position.Left} 
        className="w-3 h-3 bg-gray-400 border-2 border-white"
      />
      
      {/* Output handles on the right */}
      <Handle 
        type="source" 
        position={Position.Right} 
        className="w-3 h-3 bg-gray-400 border-2 border-white"
      />
      
      <Card className={`bg-white ${getStatusBorderColor(task.status)} border-2 shadow-md hover:shadow-lg transition-all`}>
        <CardHeader className="pb-2">
          <div className="flex items-start justify-between">
            <CardTitle className="text-sm font-medium line-clamp-2 text-gray-900">
              {task.title}
            </CardTitle>
            <div className="flex flex-col gap-1">
              <StatusBadge status={task.status} />
              <Select value={task.status} onValueChange={handleStatusChange}>
                <SelectTrigger 
                  className="w-24 h-6 text-xs"
                  onClick={(e) => e.stopPropagation()}
                >
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="pending">Pending</SelectItem>
                  <SelectItem value="ready">Ready</SelectItem>
                  <SelectItem value="in-progress">In Progress</SelectItem>
                  <SelectItem value="completed">Completed</SelectItem>
                  <SelectItem value="failed">Failed</SelectItem>
                  <SelectItem value="blocked">Blocked</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </CardHeader>
        {task.description && (
          <CardContent className="pt-0">
            <p className="text-xs text-gray-600 line-clamp-3">
              {task.description}
            </p>
          </CardContent>
        )}
      </Card>
    </div>
  )
}