import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Credential } from "@/types/credentials"
import { Trash2, Edit, Key, Database, Code, Shield } from "lucide-react"

interface CredentialCardProps {
  credential: Credential
  onEdit: (credential: Credential) => void
  onDelete: (id: string) => void
}

export function CredentialCard({ credential, onEdit, onDelete }: CredentialCardProps) {
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString()
  }

  const getTypeIcon = (type: string) => {
    switch (type) {
      case 'ssh_key':
        return <Key className="h-4 w-4" />
      case 'api_key':
        return <Code className="h-4 w-4" />
      case 'database':
        return <Database className="h-4 w-4" />
      case 'oauth_token':
        return <Shield className="h-4 w-4" />
      default:
        return <Key className="h-4 w-4" />
    }
  }

  const getTypeBadgeVariant = (type: string) => {
    switch (type) {
      case 'ssh_key':
        return 'default'
      case 'api_key':
        return 'secondary'
      case 'database':
        return 'outline'
      case 'oauth_token':
        return 'secondary'
      default:
        return 'default'
    }
  }

  const formatType = (type: string) => {
    return type.split('_').map(word => 
      word.charAt(0).toUpperCase() + word.slice(1)
    ).join(' ')
  }

  return (
    <Card className="hover:shadow-md transition-shadow">
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-2">
            {getTypeIcon(credential.credential_type)}
            <div>
              <CardTitle className="text-lg">{credential.name}</CardTitle>
              {credential.description && (
                <CardDescription className="mt-1">{credential.description}</CardDescription>
              )}
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onEdit(credential)}
              className="h-8 w-8 p-0"
            >
              <Edit className="h-4 w-4" />
            </Button>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => onDelete(credential.id)}
              className="h-8 w-8 p-0 hover:text-destructive"
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <div className="flex items-center justify-between">
          <Badge variant={getTypeBadgeVariant(credential.credential_type) as any}>
            {formatType(credential.credential_type)}
          </Badge>
          <span className="text-sm text-muted-foreground">
            Created {formatDate(credential.created_at)}
          </span>
        </div>
      </CardContent>
    </Card>
  )
}