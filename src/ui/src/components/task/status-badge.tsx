import { Badge } from "@/components/ui/badge"
import { TaskStatus, getStatusKey } from "@/types/task"

interface StatusBadgeProps {
  status: TaskStatus
}

export function StatusBadge({ status }: StatusBadgeProps) {
  const getStatusText = (status: TaskStatus): string => {
    switch (status) {
      case 'Pending':
        return 'Pending'
      case 'Ready':
        return 'Ready'
      case 'InProgress':
        return 'In Progress'
      case 'Completed':
        return 'Completed'
      case 'Failed':
        return 'Failed'
      case 'Blocked':
        return 'Blocked'
      default:
        return status
    }
  }

  const getVariant = (status: TaskStatus) => {
    const statusKey = getStatusKey(status)
    switch (statusKey) {
      case 'pending':
        return 'pending'
      case 'ready':
        return 'ready'
      case 'in-progress':
        return 'in-progress'
      case 'completed':
        return 'completed'
      case 'failed':
        return 'failed'
      case 'blocked':
        return 'blocked'
      default:
        return 'pending'
    }
  }

  return (
    <Badge variant={getVariant(status) as any}>
      {getStatusText(status)}
    </Badge>
  )
}