import { useState } from 'react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Header } from "@/components/layout/header"
import { ListView } from "@/views/list-view"
import { GraphView } from "@/views/graph-view"
import { CredentialsView } from "@/views/credentials-view"
import { useTasks } from "@/hooks/use-tasks"
import { ViewMode } from "@/types/task"
import { List, Network, Key } from 'lucide-react'

function App() {
  const { tasks, edges, loading, error, addTask, updateTaskStatus, refreshTasks } = useTasks()
  const [selectedTasks, setSelectedTasks] = useState<string[]>([])
  const [viewMode, setViewMode] = useState<ViewMode | 'credentials'>('list')

  const handleSelectTask = (taskId: string) => {
    setSelectedTasks(prev => 
      prev.includes(taskId)
        ? prev.filter(id => id !== taskId)
        : [...prev, taskId]
    )
  }

  if (loading) {
    return (
      <div className="min-h-screen bg-background">
        <Header onAddTask={addTask} />
        <main className="container mx-auto px-4 py-6">
          <div className="flex items-center justify-center h-64">
            <div className="text-center">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto"></div>
              <p className="mt-2 text-muted-foreground">Loading tasks...</p>
            </div>
          </div>
        </main>
      </div>
    )
  }

  if (error) {
    return (
      <div className="min-h-screen bg-background">
        <Header onAddTask={addTask} />
        <main className="container mx-auto px-4 py-6">
          <div className="flex items-center justify-center h-64">
            <div className="text-center">
              <p className="text-destructive mb-4">Error loading tasks: {error}</p>
              <button 
                onClick={refreshTasks}
                className="px-4 py-2 bg-primary text-primary-foreground rounded hover:bg-primary/90"
              >
                Try Again
              </button>
            </div>
          </div>
        </main>
      </div>
    )
  }

  return (
    <div className="min-h-screen bg-background">
      <Header onAddTask={addTask} />
      
      <main className="container mx-auto px-4 py-6">
        <Tabs value={viewMode} onValueChange={(value) => setViewMode(value as ViewMode | 'credentials')}>
          <TabsList className="grid w-full max-w-[600px] grid-cols-3">
            <TabsTrigger value="list" className="flex items-center gap-2">
              <List className="h-4 w-4" />
              List View
            </TabsTrigger>
            <TabsTrigger value="graph" className="flex items-center gap-2">
              <Network className="h-4 w-4" />
              Graph View
            </TabsTrigger>
            <TabsTrigger value="credentials" className="flex items-center gap-2">
              <Key className="h-4 w-4" />
              Credentials
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

          <TabsContent value="credentials" className="mt-6">
            <CredentialsView />
          </TabsContent>
        </Tabs>
      </main>
    </div>
  )
}

export default App