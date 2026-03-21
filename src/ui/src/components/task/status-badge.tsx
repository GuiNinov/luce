import { Badge } from "@/components/ui/badge"
import { TaskStatus } from "@/types/task"

interface StatusBadgeProps {
  status: TaskStatus
}

export function StatusBadge({ status }: StatusBadgeProps) {
  const getStatusText = (status: TaskStatus): string => {
    switch (status) {
      case 'pending':
        return 'Pending'
      case 'ready':
        return 'Ready'
      case 'in-progress':
        return 'In Progress'
      case 'completed':
        return 'Completed'
      case 'failed':
        return 'Failed'
      case 'blocked':
        return 'Blocked'
      default:
        return status
    }
  }

  return (
    <Badge variant={status}>
      {getStatusText(status)}
    </Badge>
  )
}