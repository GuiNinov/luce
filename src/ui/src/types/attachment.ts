export interface Attachment {
  id: string
  task_id: string
  attachment_type: string
  title: string
  url: string
  identifier: string
  created_at: string
  updated_at: string
}

export interface CreateGitHubAttachmentRequest {
  issue_number?: number
  pr_number?: number
  title?: string
  body?: string
  head_branch?: string
  base_branch?: string
  draft?: boolean
}