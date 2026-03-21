import { useState } from 'react'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { IntegrationConfig, IntegrationSettings } from "@/types/integration"
import { Settings, CheckCircle, XCircle, AlertCircle, TestTube, Save, Trash2 } from "lucide-react"
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible"

interface IntegrationCardProps {
  integration: IntegrationConfig
  onSave: (name: string, settings: IntegrationSettings) => Promise<void>
  onTest: (name: string) => Promise<any>
  onClear: (name: string) => void
}

export function IntegrationCard({ integration, onSave, onTest, onClear }: IntegrationCardProps) {
  const [isOpen, setIsOpen] = useState(false)
  const [settings, setSettings] = useState<IntegrationSettings>(
    integration.fields.reduce((acc, field) => ({
      ...acc,
      [field.key]: field.value || ''
    }), {})
  )
  const [isSaving, setIsSaving] = useState(false)
  const [isTesting, setIsTesting] = useState(false)

  const getStatusIcon = () => {
    if (!integration.configured) {
      return <Settings className="h-4 w-4 text-muted-foreground" />
    }
    if (integration.valid) {
      return <CheckCircle className="h-4 w-4 text-green-600" />
    }
    return <XCircle className="h-4 w-4 text-red-600" />
  }

  const getStatusBadge = () => {
    if (!integration.configured) {
      return <Badge variant="outline">Not Configured</Badge>
    }
    if (integration.valid) {
      return <Badge className="bg-green-100 text-green-800">Connected</Badge>
    }
    return <Badge variant="destructive">Error</Badge>
  }

  const handleSave = async () => {
    setIsSaving(true)
    try {
      await onSave(integration.name, settings)
    } finally {
      setIsSaving(false)
    }
  }

  const handleTest = async () => {
    setIsTesting(true)
    try {
      await onTest(integration.name)
    } finally {
      setIsTesting(false)
    }
  }

  const handleClear = () => {
    if (confirm(`Are you sure you want to clear all settings for ${integration.displayName}?`)) {
      onClear(integration.name)
      setSettings(integration.fields.reduce((acc, field) => ({
        ...acc,
        [field.key]: ''
      }), {}))
    }
  }

  const hasChanges = integration.fields.some(field => 
    settings[field.key] !== field.value
  )

  const isValid = integration.fields
    .filter(field => field.required)
    .every(field => settings[field.key])

  return (
    <Card>
      <Collapsible open={isOpen} onOpenChange={setIsOpen}>
        <CollapsibleTrigger asChild>
          <CardHeader className="cursor-pointer hover:bg-muted/50">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                {getStatusIcon()}
                <div>
                  <CardTitle className="text-lg">{integration.displayName}</CardTitle>
                  <CardDescription>{integration.description}</CardDescription>
                </div>
              </div>
              <div className="flex items-center gap-2">
                {getStatusBadge()}
                <Settings className={`h-4 w-4 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
              </div>
            </div>
          </CardHeader>
        </CollapsibleTrigger>

        <CollapsibleContent>
          <CardContent className="pt-0">
            <div className="space-y-4">
              {integration.fields.map((field) => (
                <div key={field.key} className="space-y-2">
                  <Label htmlFor={`${integration.name}-${field.key}`}>
                    {field.label}
                    {field.required && <span className="text-destructive ml-1">*</span>}
                  </Label>
                  <Input
                    id={`${integration.name}-${field.key}`}
                    type={field.type === 'password' ? 'password' : 'text'}
                    value={settings[field.key] as string || ''}
                    onChange={(e) => setSettings(prev => ({
                      ...prev,
                      [field.key]: e.target.value
                    }))}
                    placeholder={field.placeholder}
                  />
                  {field.description && (
                    <p className="text-sm text-muted-foreground">{field.description}</p>
                  )}
                </div>
              ))}

              {integration.details && !integration.valid && (
                <div className="flex items-start gap-2 p-3 bg-destructive/10 border border-destructive/20 rounded-md">
                  <AlertCircle className="h-4 w-4 text-destructive mt-0.5 flex-shrink-0" />
                  <div className="text-sm">
                    <p className="font-medium text-destructive">Connection Error</p>
                    <p className="text-destructive/80 mt-1">{integration.details}</p>
                  </div>
                </div>
              )}

              <div className="flex items-center gap-2 pt-2">
                <Button 
                  onClick={handleSave} 
                  disabled={!hasChanges || !isValid || isSaving}
                  size="sm"
                >
                  <Save className="h-4 w-4 mr-2" />
                  {isSaving ? 'Saving...' : 'Save'}
                </Button>
                
                <Button 
                  onClick={handleTest} 
                  variant="outline" 
                  disabled={!integration.configured || isTesting}
                  size="sm"
                >
                  <TestTube className="h-4 w-4 mr-2" />
                  {isTesting ? 'Testing...' : 'Test Connection'}
                </Button>
                
                <Button 
                  onClick={handleClear} 
                  variant="outline" 
                  disabled={!integration.configured}
                  size="sm"
                  className="ml-auto"
                >
                  <Trash2 className="h-4 w-4 mr-2" />
                  Clear
                </Button>
              </div>
            </div>
          </CardContent>
        </CollapsibleContent>
      </Collapsible>
    </Card>
  )
}