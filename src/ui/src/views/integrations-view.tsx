import { useState } from 'react'
import { Button } from "@/components/ui/button"
import { RefreshCw, Settings, CheckCircle, XCircle } from "lucide-react"
import { IntegrationCard } from "@/components/integrations/integration-card"
import { useIntegrations } from "@/hooks/use-integrations"
import { Badge } from "@/components/ui/badge"

export function IntegrationsView() {
  const { 
    integrations, 
    loading, 
    error, 
    saveIntegrationSettings, 
    testIntegration, 
    clearIntegrationSettings, 
    refreshIntegrations 
  } = useIntegrations()
  
  const [testResults, setTestResults] = useState<Record<string, any>>({})

  const handleTest = async (name: string) => {
    try {
      const result = await testIntegration(name)
      setTestResults(prev => ({ ...prev, [name]: result }))
      return result
    } catch (error) {
      setTestResults(prev => ({ ...prev, [name]: { valid: false, error: error.message } }))
      throw error
    }
  }

  const getConfiguredCount = () => {
    return integrations.filter(integration => integration.configured).length
  }

  const getConnectedCount = () => {
    return integrations.filter(integration => integration.configured && integration.valid).length
  }

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Integrations</h1>
            <p className="text-muted-foreground">Configure external service connections and credentials</p>
          </div>
        </div>
        <div className="flex items-center justify-center h-64">
          <div className="text-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto"></div>
            <p className="mt-2 text-muted-foreground">Loading integrations...</p>
          </div>
        </div>
      </div>
    )
  }

  if (error) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Integrations</h1>
            <p className="text-muted-foreground">Configure external service connections and credentials</p>
          </div>
        </div>
        <div className="flex items-center justify-center h-64">
          <div className="text-center">
            <p className="text-destructive mb-4">Error loading integrations: {error}</p>
            <Button onClick={refreshIntegrations} variant="outline">
              <RefreshCw className="h-4 w-4 mr-2" />
              Try Again
            </Button>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">Integrations</h1>
          <p className="text-muted-foreground">Configure external service connections and credentials</p>
        </div>
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Settings className="h-4 w-4" />
            <span>{getConfiguredCount()}/{integrations.length} configured</span>
          </div>
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <CheckCircle className="h-4 w-4 text-green-600" />
            <span>{getConnectedCount()} connected</span>
          </div>
          <Button onClick={refreshIntegrations} variant="outline" size="sm">
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh
          </Button>
        </div>
      </div>

      {/* Status Overview */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <div className="flex items-center justify-between p-4 border rounded-lg">
          <div>
            <p className="text-sm text-muted-foreground">Total Integrations</p>
            <p className="text-2xl font-bold">{integrations.length}</p>
          </div>
          <Settings className="h-8 w-8 text-muted-foreground" />
        </div>
        <div className="flex items-center justify-between p-4 border rounded-lg">
          <div>
            <p className="text-sm text-muted-foreground">Configured</p>
            <p className="text-2xl font-bold text-blue-600">{getConfiguredCount()}</p>
          </div>
          <CheckCircle className="h-8 w-8 text-blue-600" />
        </div>
        <div className="flex items-center justify-between p-4 border rounded-lg">
          <div>
            <p className="text-sm text-muted-foreground">Connected</p>
            <p className="text-2xl font-bold text-green-600">{getConnectedCount()}</p>
          </div>
          <CheckCircle className="h-8 w-8 text-green-600" />
        </div>
      </div>

      {/* Integration Cards */}
      <div className="space-y-4">
        {integrations.map((integration) => (
          <IntegrationCard
            key={integration.name}
            integration={integration}
            onSave={saveIntegrationSettings}
            onTest={handleTest}
            onClear={clearIntegrationSettings}
          />
        ))}
      </div>

      {/* Information Section */}
      <div className="mt-8 p-4 bg-muted/50 rounded-lg">
        <h3 className="font-medium mb-2">About Integration Configuration</h3>
        <div className="text-sm text-muted-foreground space-y-2">
          <p>• Credentials are stored locally in your browser and are used to connect to external services</p>
          <p>• Use the "Test Connection" button to verify your credentials are working correctly</p>
          <p>• Integration status is checked with the Luce API to ensure proper connectivity</p>
          <p>• Clear settings if you need to reconfigure or remove stored credentials</p>
        </div>
      </div>
    </div>
  )
}