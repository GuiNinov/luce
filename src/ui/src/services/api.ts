import { Credential, CreateCredentialRequest, UpdateCredentialRequest } from '@/types/credentials'

const API_BASE_URL = 'http://localhost:3000/api/v1'

export interface TaskResponse {
  id: string
  title: string
  description?: string
  status: 'Pending' | 'Ready' | 'InProgress' | 'Completed' | 'Failed' | 'Blocked'
  dependencies: string[]
  created_at: string
  updated_at: string
}

export interface CreateTaskRequest {
  title: string
  description?: string
  dependencies: string[]
}

export interface UpdateTaskRequest {
  title?: string
  description?: string
  status?: 'Pending' | 'Ready' | 'InProgress' | 'Completed' | 'Failed' | 'Blocked'
}

export interface GraphResponse {
  tasks: TaskResponse[]
  total_count: number
}

export interface ListTasksParams {
  status?: string
  limit?: number
  offset?: number
}

export interface IntegrationStatus {
  name: string
  enabled: boolean
  configured: boolean
  valid: boolean
  details?: any
}

export interface IntegrationsResponse {
  integrations: IntegrationStatus[]
  enabled_count: number
}

class ApiService {
  private async request<T>(
    endpoint: string, 
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${API_BASE_URL}${endpoint}`
    
    const response = await fetch(url, {
      headers: {
        'Content-Type': 'application/json',
        ...options.headers,
      },
      ...options,
    })

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}))
      throw new Error(`API Error: ${response.status} ${response.statusText} - ${errorData.error || 'Unknown error'}`)
    }

    return response.json()
  }

  // Task endpoints
  async listTasks(params: ListTasksParams = {}): Promise<GraphResponse> {
    const searchParams = new URLSearchParams()
    
    if (params.status) searchParams.set('status', params.status)
    if (params.limit) searchParams.set('limit', params.limit.toString())
    if (params.offset) searchParams.set('offset', params.offset.toString())

    const query = searchParams.toString() ? `?${searchParams.toString()}` : ''
    return this.request<GraphResponse>(`/tasks${query}`)
  }

  async createTask(request: CreateTaskRequest): Promise<TaskResponse> {
    return this.request<TaskResponse>('/tasks', {
      method: 'POST',
      body: JSON.stringify(request),
    })
  }

  async getTask(id: string): Promise<TaskResponse> {
    return this.request<TaskResponse>(`/tasks/${id}`)
  }

  async updateTask(id: string, request: UpdateTaskRequest): Promise<TaskResponse> {
    return this.request<TaskResponse>(`/tasks/${id}`, {
      method: 'PUT',
      body: JSON.stringify(request),
    })
  }

  async deleteTask(id: string): Promise<void> {
    await this.request(`/tasks/${id}`, {
      method: 'DELETE',
    })
  }

  async getReadyTasks(): Promise<TaskResponse[]> {
    return this.request<TaskResponse[]>('/tasks/ready')
  }

  async markTaskCompleted(id: string): Promise<TaskResponse> {
    return this.request<TaskResponse>(`/tasks/${id}/complete`, {
      method: 'POST',
    })
  }

  // Integration endpoints
  async listIntegrations(): Promise<IntegrationsResponse> {
    return this.request<IntegrationsResponse>('/integrations')
  }

  async testIntegration(name: string): Promise<IntegrationStatus> {
    return this.request<IntegrationStatus>(`/integrations/${name}/test`, {
      method: 'POST',
    })
  }

  // Credential endpoints
  async listCredentials(): Promise<Credential[]> {
    return this.request<Credential[]>('/credentials')
  }

  async createCredential(request: CreateCredentialRequest): Promise<Credential> {
    return this.request<Credential>('/credentials', {
      method: 'POST',
      body: JSON.stringify(request),
    })
  }

  async getCredential(id: string): Promise<Credential> {
    return this.request<Credential>(`/credentials/${id}`)
  }

  async updateCredential(id: string, request: UpdateCredentialRequest): Promise<Credential> {
    return this.request<Credential>(`/credentials/${id}`, {
      method: 'PUT',
      body: JSON.stringify(request),
    })
  }

  async deleteCredential(id: string): Promise<void> {
    await this.request(`/credentials/${id}`, {
      method: 'DELETE',
    })
  }

  // Health check
  async healthCheck(): Promise<{ status: string; timestamp: string }> {
    return this.request<{ status: string; timestamp: string }>('/health')
  }
}

export const apiService = new ApiService()