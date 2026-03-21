import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { StatusBadge } from "./status-badge"
import { TaskAttachments } from "./task-attachments"
import { Task, TaskStatus } from "@/types/task"
import { Badge } from "@/components/ui/badge"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

interface TaskCardProps {
  task: Task
  selected?: boolean
  onSelect?: (taskId: string) => void
  onStatusChange?: (taskId: string, status: TaskStatus) => void
}

export function TaskCard({ task, selected = false, onSelect, onStatusChange }: TaskCardProps) {
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString()
  }

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'critical':
        return 'destructive'
      case 'high':
        return 'outline'
      case 'normal':
        return 'secondary'
      case 'low':
        return 'secondary'
      default:
        return 'secondary'
    }
  }

  const handleStatusChange = (newStatus: string) => {
    onStatusChange?.(task.id, newStatus as TaskStatus)
  }

  return (
    <Card 
      className={`cursor-pointer transition-all hover:shadow-md ${selected ? 'ring-2 ring-primary' : ''}`}
      onClick={() => onSelect?.(task.id)}
    >
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <CardTitle className="text-lg">{task.title}</CardTitle>
          <div className="flex items-center gap-2">
            <StatusBadge status={task.status} />
            <Select value={task.status} onValueChange={handleStatusChange}>
              <SelectTrigger 
                className="w-32 h-8 text-xs"
                onClick={(e) => e.stopPropagation()}
              >
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="Pending">Pending</SelectItem>
                <SelectItem value="InProgress">In Progress</SelectItem>
                <SelectItem value="Completed">Completed</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        {task.description && (
          <CardDescription>{task.description}</CardDescription>
        )}
      </CardHeader>
      <CardContent>
        <div className="flex items-center justify-between">
          <Badge variant={getPriorityColor(task.priority) as any}>
            {task.priority.charAt(0).toUpperCase() + task.priority.slice(1)}
          </Badge>
          <span className="text-sm text-muted-foreground">
            {formatDate(task.created_at)}
          </span>
        </div>
        {task.dependencies.length > 0 && (
          <div className="mt-2">
            <span className="text-sm text-muted-foreground">
              Dependencies: {task.dependencies.length}
            </span>
          </div>
        )}
        <div className="mt-2">
          <TaskAttachments taskId={task.id} compact={true} />
        </div>
      </CardContent>
    </Card>
  )
}