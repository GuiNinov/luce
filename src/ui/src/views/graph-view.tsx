import { useCallback, useEffect, useState } from 'react'
import ReactFlow, {
  Node,
  Edge,
  addEdge,
  Connection,
  useNodesState,
  useEdgesState,
  Controls,
  Background,
  BackgroundVariant,
  ReactFlowProvider,
} from 'reactflow'
import 'reactflow/dist/style.css'

import { TaskNode } from '@/components/task/task-node'
import { AddTaskModal } from '@/components/task/add-task-modal'
import { BoardBackground } from '@/components/task/board-background'
import { Task, TaskEdge, TaskStatus, TaskPriority } from '@/types/task'
import { calculateTaskLayout } from '@/lib/layout'

const nodeTypes = {
  taskNode: TaskNode,
}

interface GraphViewProps {
  tasks: Task[]
  edges: TaskEdge[]
  selectedTasks: string[]
  onSelectTask: (taskId: string) => void
  onStatusChange: (taskId: string, status: TaskStatus) => void
  onAddTask: (task: {
    title: string
    description?: string
    priority: TaskPriority
    dependencyId?: string
    dependencyType?: 'input' | 'output'
  }) => void
}

function GraphViewContent({ tasks, edges, selectedTasks, onSelectTask, onStatusChange, onAddTask }: GraphViewProps) {
  const [nodes, setNodes, onNodesChange] = useNodesState([])
  const [flowEdges, setEdges, onEdgesChange] = useEdgesState([])
  const [connectionModal, setConnectionModal] = useState<{
    open: boolean
    taskId: string
    taskTitle: string
    type: 'input' | 'output'
  } | null>(null)

  const handleCreateConnectedTask = useCallback((sourceTaskId: string, connectionType: 'input' | 'output') => {
    const sourceTask = tasks.find(t => t.id === sourceTaskId)
    if (!sourceTask) return

    setConnectionModal({
      open: true,
      taskId: sourceTaskId,
      taskTitle: sourceTask.title,
      type: connectionType,
    })
  }, [tasks])

  // Convert tasks to nodes with automatic layout
  useEffect(() => {
    const layout = calculateTaskLayout(tasks, edges)
    
    const flowNodes: Node[] = tasks.map((task) => {
      const position = layout.positions.get(task.id) || { x: 0, y: 0 }
      
      return {
        id: task.id,
        type: 'taskNode',
        position,
        data: {
          task,
          selected: selectedTasks.includes(task.id),
          onStatusChange,
          onCreateConnectedTask: handleCreateConnectedTask,
          xPos: position.x,
        },
      }
    })

    setNodes(flowNodes)
  }, [tasks, edges, selectedTasks, onStatusChange, handleCreateConnectedTask, setNodes])

  // Convert task edges to flow edges
  useEffect(() => {
    const reactFlowEdges: Edge[] = edges.map((edge) => ({
      id: `${edge.from}-${edge.to}`,
      source: edge.from,
      target: edge.to,
      type: 'smoothstep',
      animated: true,
    }))

    setEdges(reactFlowEdges)
  }, [edges, setEdges])

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  )

  const onNodeClick = useCallback(
    (event: React.MouseEvent, node: Node) => {
      onSelectTask(node.id)
    },
    [onSelectTask]
  )

  return (
    <>
      <div className="w-full h-[600px] border rounded-lg bg-white relative overflow-hidden">
        <BoardBackground width={1600} height={600} />
        <ReactFlow
          nodes={nodes}
          edges={flowEdges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onNodeClick={onNodeClick}
          nodeTypes={nodeTypes}
          fitView
          attributionPosition="bottom-left"
          className="bg-transparent"
        >
          <Controls />
        </ReactFlow>
      </div>
      
      {connectionModal && (
        <AddTaskModal
          open={connectionModal.open}
          onOpenChange={(open) => {
            if (!open) {
              setConnectionModal(null)
            }
          }}
          initialConnection={{
            taskId: connectionModal.taskId,
            taskTitle: connectionModal.taskTitle,
            type: connectionModal.type,
          }}
          onAddTask={onAddTask}
        />
      )}
    </>
  )
}

export function GraphView(props: GraphViewProps) {
  if (props.tasks.length === 0) {
    return (
      <div className="w-full h-[600px] border rounded-lg bg-background flex items-center justify-center">
        <div className="text-center">
          <p className="text-muted-foreground">No tasks yet. Add your first task to see the graph!</p>
        </div>
      </div>
    )
  }

  return (
    <ReactFlowProvider>
      <GraphViewContent {...props} />
    </ReactFlowProvider>
  )
}