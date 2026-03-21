import { useState, useEffect } from 'react'
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Attachment } from "@/types/attachment"
import { apiService } from "@/services/api"
import { ExternalLink, Github, Paperclip, RefreshCw } from "lucide-react"

interface TaskAttachmentsProps {
  taskId: string
  compact?: boolean
}

export function TaskAttachments({ taskId, compact = false }: TaskAttachmentsProps) {
  const [attachments, setAttachments] = useState<Attachment[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const loadAttachments = async () => {
    try {
      setLoading(true)
      setError(null)
      const attachmentList = await apiService.listTaskAttachments(taskId)
      setAttachments(attachmentList)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load attachments')
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    loadAttachments()
  }, [taskId])

  const getAttachmentIcon = (type: string) => {
    switch (type.toLowerCase()) {
      case 'github':
      case 'github_issue':
      case 'github_pr':
        return <Github className="h-4 w-4" />
      default:
        return <Paperclip className="h-4 w-4" />
    }
  }

  const getAttachmentColor = (type: string) => {
    switch (type.toLowerCase()) {
      case 'github':
      case 'github_issue':
        return 'bg-orange-100 text-orange-800'
      case 'github_pr':
        return 'bg-green-100 text-green-800'
      default:
        return 'bg-gray-100 text-gray-800'
    }
  }

  const formatAttachmentType = (type: string) => {
    return type.replace('_', ' ').replace(/\b\w/g, l => l.toUpperCase())
  }

  if (loading) {
    return compact ? (
      <div className="flex items-center gap-1 text-xs text-muted-foreground">
        <RefreshCw className="h-3 w-3 animate-spin" />
        Loading...
      </div>
    ) : (
      <div className="text-center py-4">
        <RefreshCw className="h-4 w-4 animate-spin mx-auto mb-2" />
        <p className="text-sm text-muted-foreground">Loading attachments...</p>
      </div>
    )
  }

  if (error) {
    return compact ? (
      <div className="text-xs text-destructive">Error loading attachments</div>
    ) : (
      <div className="text-center py-4">
        <p className="text-sm text-destructive mb-2">Error: {error}</p>
        <Button variant="outline" size="sm" onClick={loadAttachments}>
          <RefreshCw className="h-4 w-4 mr-2" />
          Retry
        </Button>
      </div>
    )
  }

  if (attachments.length === 0) {
    return compact ? (
      <div className="text-xs text-muted-foreground">No attachments</div>
    ) : (
      <div className="text-center py-4 text-sm text-muted-foreground">
        No attachments found
      </div>
    )
  }

  if (compact) {
    return (
      <div className="flex items-center gap-1">
        <Paperclip className="h-3 w-3 text-muted-foreground" />
        <span className="text-xs text-muted-foreground">{attachments.length} attachment{attachments.length !== 1 ? 's' : ''}</span>
      </div>
    )
  }

  return (
    <div className="space-y-2">
      <h4 className="text-sm font-medium flex items-center gap-2">
        <Paperclip className="h-4 w-4" />
        Attachments ({attachments.length})
      </h4>
      <div className="space-y-2">
        {attachments.map((attachment) => (
          <div
            key={attachment.id}
            className="flex items-center justify-between p-2 rounded-md border bg-card"
          >
            <div className="flex items-center gap-2 min-w-0 flex-1">
              {getAttachmentIcon(attachment.attachment_type)}
              <div className="min-w-0 flex-1">
                <p className="text-sm font-medium truncate">{attachment.title}</p>
                <div className="flex items-center gap-2">
                  <Badge 
                    variant="outline" 
                    className={`text-xs ${getAttachmentColor(attachment.attachment_type)}`}
                  >
                    {formatAttachmentType(attachment.attachment_type)}
                  </Badge>
                  <span className="text-xs text-muted-foreground">{attachment.identifier}</span>
                </div>
              </div>
            </div>
            <Button
              variant="ghost"
              size="sm"
              className="h-8 w-8 p-0"
              onClick={() => window.open(attachment.url, '_blank')}
            >
              <ExternalLink className="h-4 w-4" />
            </Button>
          </div>
        ))}
      </div>
    </div>
  )
}