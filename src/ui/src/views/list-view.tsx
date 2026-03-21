import { TaskCard } from "@/components/task/task-card"
import { Task, TaskStatus } from "@/types/task"

interface ListViewProps {
  tasks: Task[]
  selectedTasks: string[]
  onSelectTask: (taskId: string) => void
  onStatusChange: (taskId: string, status: TaskStatus) => void
}

export function ListView({ tasks, selectedTasks, onSelectTask, onStatusChange }: ListViewProps) {
  return (
    <div className="space-y-4">
      {tasks.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-muted-foreground">No tasks yet. Add your first task to get started!</p>
        </div>
      ) : (
        <div className="grid gap-4">
          {tasks.map((task) => (
            <TaskCard
              key={task.id}
              task={task}
              selected={selectedTasks.includes(task.id)}
              onSelect={onSelectTask}
              onStatusChange={onStatusChange}
            />
          ))}
        </div>
      )}
    </div>
  )
}