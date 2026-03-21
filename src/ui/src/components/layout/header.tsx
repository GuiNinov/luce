import { AddTaskModal } from "@/components/task/add-task-modal"
import { TaskPriority } from "@/types/task"

interface HeaderProps {
  onAddTask: (task: {
    title: string
    description?: string
    priority: TaskPriority
  }) => void
}

export function Header({ onAddTask }: HeaderProps) {
  return (
    <header className="border-b bg-background">
      <div className="container mx-auto px-4 py-4">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Luce</h1>
            <p className="text-muted-foreground">Parallel Task Management</p>
          </div>
          <AddTaskModal onAddTask={onAddTask} />
        </div>
      </div>
    </header>
  )
}