import { useState, useEffect } from 'react'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Credential, CreateCredentialRequest, CredentialType, CREDENTIAL_TYPES } from "@/types/credentials"

interface CredentialModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  credential?: Credential
  onSave: (data: CreateCredentialRequest) => Promise<void>
}

export function CredentialModal({ open, onOpenChange, credential, onSave }: CredentialModalProps) {
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [credentialType, setCredentialType] = useState<CredentialType>('api_key')
  const [credentialData, setCredentialData] = useState<Record<string, string>>({})
  const [isSubmitting, setIsSubmitting] = useState(false)

  const isEditing = !!credential

  useEffect(() => {
    if (credential) {
      setName(credential.name)
      setDescription(credential.description || '')
      setCredentialType(credential.credential_type)
      setCredentialData({})
    } else {
      setName('')
      setDescription('')
      setCredentialType('api_key')
      setCredentialData({})
    }
  }, [credential, open])

  const handleSubmit = async () => {
    if (!name.trim()) return

    setIsSubmitting(true)
    try {
      const requestData: CreateCredentialRequest = {
        name: name.trim(),
        description: description.trim() || undefined,
        credential_type: credentialType,
        credential_data: credentialData
      }

      await onSave(requestData)
      onOpenChange(false)
    } catch (error) {
      console.error('Failed to save credential:', error)
    } finally {
      setIsSubmitting(false)
    }
  }

  const getCredentialFields = () => {
    switch (credentialType) {
      case 'ssh_key':
        return (
          <div className="grid gap-4">
            <div>
              <Label htmlFor="private_key">Private Key</Label>
              <Textarea
                id="private_key"
                placeholder="-----BEGIN RSA PRIVATE KEY-----"
                value={credentialData.private_key || ''}
                onChange={(e) => setCredentialData(prev => ({ ...prev, private_key: e.target.value }))}
                className="font-mono text-sm"
                rows={6}
              />
            </div>
            <div>
              <Label htmlFor="username">Username (optional)</Label>
              <Input
                id="username"
                value={credentialData.username || ''}
                onChange={(e) => setCredentialData(prev => ({ ...prev, username: e.target.value }))}
                placeholder="git, ubuntu, etc."
              />
            </div>
            <div>
              <Label htmlFor="host">Host (optional)</Label>
              <Input
                id="host"
                value={credentialData.host || ''}
                onChange={(e) => setCredentialData(prev => ({ ...prev, host: e.target.value }))}
                placeholder="github.com, server.example.com"
              />
            </div>
          </div>
        )
      case 'api_key':
        return (
          <div className="grid gap-4">
            <div>
              <Label htmlFor="api_key">API Key</Label>
              <Input
                id="api_key"
                type="password"
                value={credentialData.api_key || ''}
                onChange={(e) => setCredentialData(prev => ({ ...prev, api_key: e.target.value }))}
                placeholder="Enter API key"
              />
            </div>
            <div>
              <Label htmlFor="api_url">API URL (optional)</Label>
              <Input
                id="api_url"
                value={credentialData.api_url || ''}
                onChange={(e) => setCredentialData(prev => ({ ...prev, api_url: e.target.value }))}
                placeholder="https://api.example.com"
              />
            </div>
          </div>
        )
      case 'database':
        return (
          <div className="grid gap-4">
            <div>
              <Label htmlFor="connection_string">Connection String</Label>
              <Input
                id="connection_string"
                type="password"
                value={credentialData.connection_string || ''}
                onChange={(e) => setCredentialData(prev => ({ ...prev, connection_string: e.target.value }))}
                placeholder="postgresql://user:pass@host:5432/db"
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label htmlFor="username">Username</Label>
                <Input
                  id="username"
                  value={credentialData.username || ''}
                  onChange={(e) => setCredentialData(prev => ({ ...prev, username: e.target.value }))}
                />
              </div>
              <div>
                <Label htmlFor="password">Password</Label>
                <Input
                  id="password"
                  type="password"
                  value={credentialData.password || ''}
                  onChange={(e) => setCredentialData(prev => ({ ...prev, password: e.target.value }))}
                />
              </div>
            </div>
          </div>
        )
      case 'oauth_token':
        return (
          <div className="grid gap-4">
            <div>
              <Label htmlFor="access_token">Access Token</Label>
              <Input
                id="access_token"
                type="password"
                value={credentialData.access_token || ''}
                onChange={(e) => setCredentialData(prev => ({ ...prev, access_token: e.target.value }))}
                placeholder="Enter access token"
              />
            </div>
            <div>
              <Label htmlFor="refresh_token">Refresh Token (optional)</Label>
              <Input
                id="refresh_token"
                type="password"
                value={credentialData.refresh_token || ''}
                onChange={(e) => setCredentialData(prev => ({ ...prev, refresh_token: e.target.value }))}
                placeholder="Enter refresh token"
              />
            </div>
            <div>
              <Label htmlFor="scope">Scope (optional)</Label>
              <Input
                id="scope"
                value={credentialData.scope || ''}
                onChange={(e) => setCredentialData(prev => ({ ...prev, scope: e.target.value }))}
                placeholder="read:user, repo, etc."
              />
            </div>
          </div>
        )
      default:
        return null
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>{isEditing ? 'Edit Credential' : 'Add New Credential'}</DialogTitle>
          <DialogDescription>
            {isEditing ? 'Update credential information.' : 'Add a new credential to securely store authentication data.'}
          </DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <div>
            <Label htmlFor="name">Name</Label>
            <Input
              id="name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Enter credential name"
            />
          </div>

          <div>
            <Label htmlFor="description">Description (optional)</Label>
            <Textarea
              id="description"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Describe what this credential is used for"
              rows={2}
            />
          </div>

          <div>
            <Label htmlFor="type">Type</Label>
            <Select value={credentialType} onValueChange={(value) => setCredentialType(value as CredentialType)}>
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {CREDENTIAL_TYPES.map(type => (
                  <SelectItem key={type.value} value={type.value}>
                    {type.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {getCredentialFields()}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={isSubmitting || !name.trim()}>
            {isSubmitting ? 'Saving...' : isEditing ? 'Update' : 'Create'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}