import { useState, useEffect } from 'react'
import { apiService } from '@/services/api'
import { IntegrationConfig, IntegrationSettings, INTEGRATION_CONFIGS } from '@/types/integration'

export function useIntegrations() {
  const [integrations, setIntegrations] = useState<IntegrationConfig[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  // Local storage keys for integration settings
  const STORAGE_KEY_PREFIX = 'luce_integration_'

  const loadIntegrations = async () => {
    try {
      setLoading(true)
      setError(null)
      
      // Get integration status from API
      const response = await apiService.listIntegrations()
      
      // Combine with local config and stored settings
      const integrationsWithConfig = response.integrations.map(integration => {
        const config = INTEGRATION_CONFIGS[integration.name]
        const storedSettings = getStoredSettings(integration.name)
        
        if (!config) {
          return {
            name: integration.name,
            displayName: integration.name,
            description: 'Unknown integration',
            enabled: integration.enabled,
            configured: integration.configured,
            valid: integration.valid,
            fields: [],
            details: integration.details
          }
        }

        // Populate field values from stored settings
        const fieldsWithValues = config.fields.map(field => ({
          ...field,
          value: storedSettings[field.key] as string || ''
        }))

        return {
          ...config,
          enabled: integration.enabled,
          configured: integration.configured,
          valid: integration.valid,
          fields: fieldsWithValues,
          details: integration.details
        }
      })
      
      setIntegrations(integrationsWithConfig)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load integrations')
    } finally {
      setLoading(false)
    }
  }

  const getStoredSettings = (integrationName: string): IntegrationSettings => {
    try {
      const stored = localStorage.getItem(`${STORAGE_KEY_PREFIX}${integrationName}`)
      return stored ? JSON.parse(stored) : {}
    } catch {
      return {}
    }
  }

  const saveIntegrationSettings = async (integrationName: string, settings: IntegrationSettings) => {
    try {
      // Save to local storage
      localStorage.setItem(`${STORAGE_KEY_PREFIX}${integrationName}`, JSON.stringify(settings))
      
      // Update the integration in state
      setIntegrations(prev => 
        prev.map(integration => {
          if (integration.name === integrationName) {
            const fieldsWithValues = integration.fields.map(field => ({
              ...field,
              value: settings[field.key] as string || ''
            }))
            return {
              ...integration,
              fields: fieldsWithValues,
              configured: Object.values(settings).some(value => value !== '')
            }
          }
          return integration
        })
      )
      
      return true
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to save integration settings'
      setError(errorMsg)
      throw new Error(errorMsg)
    }
  }

  const testIntegration = async (integrationName: string) => {
    try {
      setError(null)
      const result = await apiService.testIntegration(integrationName)
      
      // Update the integration status
      setIntegrations(prev => 
        prev.map(integration => 
          integration.name === integrationName 
            ? { ...integration, valid: result.valid, details: result.details }
            : integration
        )
      )
      
      return result
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to test integration'
      setError(errorMsg)
      throw new Error(errorMsg)
    }
  }

  const clearIntegrationSettings = (integrationName: string) => {
    try {
      localStorage.removeItem(`${STORAGE_KEY_PREFIX}${integrationName}`)
      
      // Update the integration in state
      setIntegrations(prev => 
        prev.map(integration => {
          if (integration.name === integrationName) {
            const clearedFields = integration.fields.map(field => ({
              ...field,
              value: ''
            }))
            return {
              ...integration,
              fields: clearedFields,
              configured: false,
              valid: false
            }
          }
          return integration
        })
      )
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to clear integration settings')
    }
  }

  const refreshIntegrations = () => {
    loadIntegrations()
  }

  useEffect(() => {
    loadIntegrations()
  }, [])

  return {
    integrations,
    loading,
    error,
    saveIntegrationSettings,
    testIntegration,
    clearIntegrationSettings,
    refreshIntegrations,
    getStoredSettings
  }
}