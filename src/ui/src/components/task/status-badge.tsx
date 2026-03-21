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
      case 'InProgress':
        return 'In Progress'
      case 'Completed':
        return 'Completed'
      default:
        return status
    }
  }

  const getVariant = (status: TaskStatus) => {
    const statusKey = getStatusKey(status)
    switch (statusKey) {
      case 'pending':
        return 'pending'
      case 'in-progress':
        return 'in-progress'
      case 'completed':
        return 'completed'
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