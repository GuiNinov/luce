export interface IntegrationConfig {
  name: string
  displayName: string
  description: string
  enabled: boolean
  configured: boolean
  valid: boolean
  fields: IntegrationField[]
  details?: any
}

export interface IntegrationField {
  key: string
  label: string
  type: 'text' | 'password' | 'url' | 'select'
  required: boolean
  description?: string
  placeholder?: string
  options?: { label: string; value: string }[]
  value?: string
}

export interface IntegrationSettings {
  [key: string]: string | boolean | number
}

export const INTEGRATION_CONFIGS: Record<string, Omit<IntegrationConfig, 'enabled' | 'configured' | 'valid' | 'details'>> = {
  github: {
    name: 'github',
    displayName: 'GitHub',
    description: 'Connect to GitHub for issue and PR management',
    fields: [
      {
        key: 'token',
        label: 'Personal Access Token',
        type: 'password',
        required: true,
        description: 'GitHub Personal Access Token with repo and issue permissions',
        placeholder: 'ghp_xxxxxxxxxxxxxxxxxxxx'
      },
      {
        key: 'owner',
        label: 'Repository Owner',
        type: 'text',
        required: true,
        description: 'GitHub username or organization name',
        placeholder: 'octocat'
      },
      {
        key: 'repo',
        label: 'Repository Name',
        type: 'text',
        required: true,
        description: 'Name of the repository to integrate with',
        placeholder: 'my-project'
      }
    ]
  },
  slack: {
    name: 'slack',
    displayName: 'Slack',
    description: 'Send notifications and updates to Slack channels',
    fields: [
      {
        key: 'webhook_url',
        label: 'Webhook URL',
        type: 'url',
        required: true,
        description: 'Slack webhook URL for sending messages',
        placeholder: 'https://hooks.slack.com/services/...'
      },
      {
        key: 'channel',
        label: 'Default Channel',
        type: 'text',
        required: false,
        description: 'Default Slack channel for notifications',
        placeholder: '#general'
      }
    ]
  },
  linear: {
    name: 'linear',
    displayName: 'Linear',
    description: 'Sync with Linear for issue tracking and project management',
    fields: [
      {
        key: 'api_key',
        label: 'API Key',
        type: 'password',
        required: true,
        description: 'Linear API key for accessing your workspace',
        placeholder: 'lin_api_xxxxxxxxxx'
      },
      {
        key: 'team_id',
        label: 'Team ID',
        type: 'text',
        required: false,
        description: 'Linear team ID (leave empty for default team)',
        placeholder: 'team-abc123'
      }
    ]
  }
}