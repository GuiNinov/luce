import { useState } from "react"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
  DialogFooter,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Plus } from "lucide-react"
import { TaskPriority } from "@/types/task"

interface AddTaskModalProps {
  onAddTask: (task: {
    title: string
    description?: string
    priority: TaskPriority
    dependencyId?: string
    dependencyType?: 'input' | 'output'
  }) => void
  initialConnection?: {
    taskId: string
    taskTitle: string
    type: 'input' | 'output'
  }
  open?: boolean
  onOpenChange?: (open: boolean) => void
}

export function AddTaskModal({ 
  onAddTask, 
  initialConnection, 
  open: controlledOpen, 
  onOpenChange: controlledOnOpenChange 
}: AddTaskModalProps) {
  const [internalOpen, setInternalOpen] = useState(false)
  const [title, setTitle] = useState("")
  const [description, setDescription] = useState("")
  const [priority, setPriority] = useState<TaskPriority>("normal")

  const isControlled = controlledOpen !== undefined
  const open = isControlled ? controlledOpen : internalOpen
  const setOpen = isControlled ? (controlledOnOpenChange || (() => {})) : setInternalOpen

  const handleSubmit = () => {
    if (!title.trim()) return

    onAddTask({
      title: title.trim(),
      description: description.trim() || undefined,
      priority,
      dependencyId: initialConnection?.taskId,
      dependencyType: initialConnection?.type,
    })

    setTitle("")
    setDescription("")
    setPriority("normal")
    setOpen(false)
  }

  const getConnectionDescription = () => {
    if (!initialConnection) return null
    
    const isInput = initialConnection.type === 'input'
    return (
      <div className="p-3 bg-blue-50 rounded-md border border-blue-200">
        <p className="text-sm text-blue-800">
          {isInput ? '📥' : '📤'} This task will be {isInput ? 'a prerequisite for' : 'dependent on'}: 
          <span className="font-semibold ml-1">"{initialConnection.taskTitle}"</span>
        </p>
      </div>
    )
  }

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      {!isControlled && (
        <DialogTrigger asChild>
          <Button>
            <Plus className="h-4 w-4 mr-2" />
            Add Task
          </Button>
        </DialogTrigger>
      )}
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>
            {initialConnection ? 'Add Connected Task' : 'Add New Task'}
          </DialogTitle>
          <DialogDescription>
            {initialConnection 
              ? 'Create a task with automatic dependency connection.'
              : 'Create a new task for your project workflow.'
            }
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          {getConnectionDescription()}
          <div className="grid gap-2">
            <Label htmlFor="title">Title</Label>
            <Input
              id="title"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder="Enter task title"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Enter task description (optional)"
              rows={3}
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="priority">Priority</Label>
            <Select value={priority} onValueChange={(value: TaskPriority) => setPriority(value)}>
              <SelectTrigger>
                <SelectValue placeholder="Select priority" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="low">Low</SelectItem>
                <SelectItem value="normal">Normal</SelectItem>
                <SelectItem value="high">High</SelectItem>
                <SelectItem value="critical">Critical</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => setOpen(false)}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!title.trim()}>
            Add Task
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}