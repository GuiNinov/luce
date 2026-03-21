export interface Credential {
  id: string
  name: string
  credential_type: 'ssh_key' | 'api_key' | 'database' | 'oauth_token'
  description?: string
  created_at: string
  updated_at: string
  metadata: Record<string, any>
}

export interface CreateCredentialRequest {
  name: string
  credential_type: 'ssh_key' | 'api_key' | 'database' | 'oauth_token'
  description?: string
  credential_data: Record<string, any>
}

export interface UpdateCredentialRequest {
  name?: string
  description?: string
  credential_data?: Record<string, any>
}

export type CredentialType = 'ssh_key' | 'api_key' | 'database' | 'oauth_token'

export const CREDENTIAL_TYPES: { value: CredentialType; label: string }[] = [
  { value: 'ssh_key', label: 'SSH Key' },
  { value: 'api_key', label: 'API Key' },
  { value: 'database', label: 'Database' },
  { value: 'oauth_token', label: 'OAuth Token' }
]