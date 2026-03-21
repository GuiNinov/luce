import { Task, TaskEdge } from '@/types/task'

interface TaskPosition {
  x: number
  y: number
}

interface LayoutResult {
  positions: Map<string, TaskPosition>
  layers: {
    completed: Task[]
    roots: Task[]
    chains: Task[][]
  }
}

export function calculateTaskLayout(tasks: Task[], edges: TaskEdge[]): LayoutResult {
  const COMPLETED_X = -400 // Left side for completed tasks
  const ROOT_X = 300 // Center for root tasks
  const CHILD_SPACING_X = 350 // Horizontal spacing between levels
  const TASK_SPACING_Y = 150 // Vertical spacing between tasks
  const LAYER_SPACING_Y = 100 // Extra spacing between different root chains

  const positions = new Map<string, TaskPosition>()
  
  // Separate completed tasks
  const completedTasks = tasks.filter(task => task.status === 'Completed')
  const activeTasks = tasks.filter(task => task.status !== 'Completed')
  
  // Build dependency graph for active tasks
  const dependencyMap = new Map<string, string[]>() // taskId -> [dependentTaskIds]
  const dependentsMap = new Map<string, string[]>() // taskId -> [tasksThatDependOnThis]
  
  activeTasks.forEach(task => {
    dependencyMap.set(task.id, task.dependencies.filter(depId => 
      activeTasks.find(t => t.id === depId) // Only include dependencies that are active
    ))
    dependentsMap.set(task.id, [])
  })
  
  // Fill dependents map
  edges.forEach(edge => {
    const fromTask = activeTasks.find(t => t.id === edge.from)
    const toTask = activeTasks.find(t => t.id === edge.to)
    if (fromTask && toTask) {
      const dependents = dependentsMap.get(edge.from) || []
      dependents.push(edge.to)
      dependentsMap.set(edge.from, dependents)
    }
  })
  
  // Find root tasks (no dependencies or dependencies are completed)
  const rootTasks = activeTasks.filter(task => 
    task.dependencies.length === 0 || 
    task.dependencies.every(depId => 
      completedTasks.find(ct => ct.id === depId)
    )
  )
  
  // Position completed tasks on the left
  let completedY = 0
  completedTasks.forEach((task, index) => {
    positions.set(task.id, {
      x: COMPLETED_X,
      y: completedY
    })
    completedY += TASK_SPACING_Y
  })
  
  // Build task chains starting from root tasks
  const visitedTasks = new Set<string>()
  const taskChains: Task[][] = []
  let currentRootY = 0
  
  rootTasks.forEach(rootTask => {
    if (visitedTasks.has(rootTask.id)) return
    
    const chain = buildTaskChain(rootTask, dependentsMap, activeTasks, visitedTasks)
    taskChains.push(chain)
    
    // Position this chain
    positionTaskChain(chain, ROOT_X, currentRootY, CHILD_SPACING_X, TASK_SPACING_Y, positions)
    
    // Calculate height of this chain to position next one
    const chainHeight = Math.max(chain.length * TASK_SPACING_Y, TASK_SPACING_Y)
    currentRootY += chainHeight + LAYER_SPACING_Y
  })
  
  return {
    positions,
    layers: {
      completed: completedTasks,
      roots: rootTasks,
      chains: taskChains
    }
  }
}

function buildTaskChain(
  rootTask: Task, 
  dependentsMap: Map<string, string[]>, 
  allTasks: Task[], 
  visitedTasks: Set<string>
): Task[][] {
  const chain: Task[][] = []
  const queue: Array<{ task: Task, level: number }> = [{ task: rootTask, level: 0 }]
  
  while (queue.length > 0) {
    const { task, level } = queue.shift()!
    
    if (visitedTasks.has(task.id)) continue
    visitedTasks.add(task.id)
    
    // Ensure chain array has enough levels
    while (chain.length <= level) {
      chain.push([])
    }
    
    chain[level].push(task)
    
    // Add dependent tasks to queue
    const dependents = dependentsMap.get(task.id) || []
    dependents.forEach(dependentId => {
      const dependentTask = allTasks.find(t => t.id === dependentId)
      if (dependentTask && !visitedTasks.has(dependentId)) {
        queue.push({ task: dependentTask, level: level + 1 })
      }
    })
  }
  
  return chain.filter(level => level.length > 0) // Remove empty levels
}

function positionTaskChain(
  chain: Task[][], 
  startX: number, 
  startY: number, 
  spacingX: number, 
  spacingY: number, 
  positions: Map<string, TaskPosition>
) {
  chain.forEach((level, levelIndex) => {
    const x = startX + (levelIndex * spacingX)
    let y = startY
    
    level.forEach((task, taskIndex) => {
      positions.set(task.id, { x, y })
      y += spacingY
    })
  })
}