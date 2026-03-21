import { useState } from 'react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Header } from "@/components/layout/header"
import { ListView } from "@/views/list-view"
import { GraphView } from "@/views/graph-view"
import { useTasks } from "@/hooks/use-tasks"
import { ViewMode } from "@/types/task"
import { List, Network } from 'lucide-react'

function App() {
  const { tasks, edges, addTask, updateTaskStatus } = useTasks()
  const [selectedTasks, setSelectedTasks] = useState<string[]>([])
  const [viewMode, setViewMode] = useState<ViewMode>('list')

  const handleSelectTask = (taskId: string) => {
    setSelectedTasks(prev => 
      prev.includes(taskId)
        ? prev.filter(id => id !== taskId)
        : [...prev, taskId]
    )
  }

  return (
    <div className="min-h-screen bg-background">
      <Header onAddTask={addTask} />
      
      <main className="container mx-auto px-4 py-6">
        <Tabs value={viewMode} onValueChange={(value) => setViewMode(value as ViewMode)}>
          <TabsList className="grid w-full max-w-[400px] grid-cols-2">
            <TabsTrigger value="list" className="flex items-center gap-2">
              <List className="h-4 w-4" />
              List View
            </TabsTrigger>
            <TabsTrigger value="graph" className="flex items-center gap-2">
              <Network className="h-4 w-4" />
              Graph View
            </TabsTrigger>
          </TabsList>

          <TabsContent value="list" className="mt-6">
            <ListView
              tasks={tasks}
              selectedTasks={selectedTasks}
              onSelectTask={handleSelectTask}
              onStatusChange={updateTaskStatus}
            />
          </TabsContent>

          <TabsContent value="graph" className="mt-6">
            <GraphView
              tasks={tasks}
              edges={edges}
              selectedTasks={selectedTasks}
              onSelectTask={handleSelectTask}
              onStatusChange={updateTaskStatus}
              onAddTask={addTask}
            />
          </TabsContent>
        </Tabs>
      </main>
    </div>
  )
}

export default App