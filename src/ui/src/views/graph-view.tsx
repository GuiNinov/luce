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
import { Task, TaskEdge, TaskStatus } from '@/types/task'

const nodeTypes = {
  taskNode: TaskNode,
}

interface GraphViewProps {
  tasks: Task[]
  edges: TaskEdge[]
  selectedTasks: string[]
  onSelectTask: (taskId: string) => void
  onStatusChange: (taskId: string, status: TaskStatus) => void
}

function GraphViewContent({ tasks, edges, selectedTasks, onSelectTask, onStatusChange }: GraphViewProps) {
  const [nodes, setNodes, onNodesChange] = useNodesState([])
  const [flowEdges, setEdges, onEdgesChange] = useEdgesState([])

  // Convert tasks to nodes
  useEffect(() => {
    const flowNodes: Node[] = tasks.map((task, index) => ({
      id: task.id,
      type: 'taskNode',
      position: { x: (index % 3) * 300, y: Math.floor(index / 3) * 200 },
      data: {
        task,
        selected: selectedTasks.includes(task.id),
        onStatusChange,
      },
    }))

    setNodes(flowNodes)
  }, [tasks, selectedTasks, onStatusChange, setNodes])

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
    <div className="w-full h-[600px] border rounded-lg bg-background">
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
      >
        <Controls />
        <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
      </ReactFlow>
    </div>
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