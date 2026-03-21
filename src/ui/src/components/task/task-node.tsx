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
import { Plus } from "lucide-react"

interface TaskNodeProps {
  data: {
    task: Task
    selected: boolean
    onStatusChange?: (taskId: string, status: TaskStatus) => void
    onCreateConnectedTask?: (sourceTaskId: string, connectionType: 'input' | 'output') => void
    xPos?: number
  }
}

export function TaskNode({ data }: TaskNodeProps) {
  const { task, selected, onStatusChange, onCreateConnectedTask, xPos } = data

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

  // Determine which zone the task is in
  const isCompleted = task.status === 'completed'
  const isInActiveZone = (xPos || 0) >= 250 && (xPos || 0) <= 1050
  const isInCompletedZone = (xPos || 0) < 250
  
  // Enhanced styling for active work zone
  const nodeOpacity = isCompleted ? 'opacity-60' : 'opacity-100'
  const textColor = isCompleted ? 'text-gray-500' : 'text-gray-900'
  const descriptionColor = isCompleted ? 'text-gray-400' : 'text-gray-600'
  
  // Enhanced styling for active zone tasks
  const activeZoneEnhancement = isInActiveZone && !isCompleted ? {
    shadow: 'shadow-lg hover:shadow-xl',
    border: 'border-2',
    glow: 'ring-1 ring-blue-100',
    scale: 'hover:scale-105',
    handleColor: 'bg-blue-500 hover:bg-blue-600',
    animation: task.status === 'ready' ? 'animate-pulse' : ''
  } : {
    shadow: 'shadow-md hover:shadow-lg',
    border: 'border-2', 
    glow: '',
    scale: '',
    handleColor: 'bg-gray-400 hover:bg-gray-500',
    animation: ''
  }

  const handleStatusChange = (newStatus: string) => {
    onStatusChange?.(task.id, newStatus as TaskStatus)
  }

  const handleInputHandleClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    onCreateConnectedTask?.(task.id, 'input')
  }

  const handleOutputHandleClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    onCreateConnectedTask?.(task.id, 'output')
  }

  return (
    <div className={`min-w-[220px] max-w-[320px] relative ${selected ? 'ring-2 ring-primary' : ''} ${nodeOpacity}`}>
      {/* Input handle on the left - hidden for completed tasks */}
      {!isCompleted && (
        <div 
          className={`absolute left-[-12px] top-1/2 transform -translate-y-1/2 z-10 w-6 h-6 ${activeZoneEnhancement.handleColor} rounded-full flex items-center justify-center cursor-pointer transition-colors shadow-sm ${activeZoneEnhancement.animation}`}
          onClick={handleInputHandleClick}
          title="Click to create a prerequisite task"
        >
          <Plus className="w-3 h-3 text-white" />
        </div>
      )}
      <Handle 
        type="target" 
        position={Position.Left} 
        className="opacity-0"
      />
      
      {/* Output handle on the right - hidden for completed tasks */}
      {!isCompleted && (
        <div 
          className={`absolute right-[-12px] top-1/2 transform -translate-y-1/2 z-10 w-6 h-6 ${activeZoneEnhancement.handleColor} rounded-full flex items-center justify-center cursor-pointer transition-colors shadow-sm ${activeZoneEnhancement.animation}`}
          onClick={handleOutputHandleClick}
          title="Click to create a dependent task"
        >
          <Plus className="w-3 h-3 text-white" />
        </div>
      )}
      <Handle 
        type="source" 
        position={Position.Right} 
        className="opacity-0"
      />
      
      <Card className={`bg-white ${getStatusBorderColor(task.status)} ${activeZoneEnhancement.border} ${activeZoneEnhancement.shadow} ${activeZoneEnhancement.glow} ${activeZoneEnhancement.scale} transition-all duration-200`}>
        <CardHeader className="pb-2">
          <div className="flex items-start justify-between">
            <CardTitle className={`text-sm font-medium line-clamp-2 ${textColor}`}>
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
            <p className={`text-xs line-clamp-3 ${descriptionColor}`}>
              {task.description}
            </p>
          </CardContent>
        )}
      </Card>
    </div>
  )
}