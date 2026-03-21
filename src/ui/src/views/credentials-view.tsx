import { useState } from 'react'
import { Button } from "@/components/ui/button"
import { Plus, RefreshCw } from "lucide-react"
import { CredentialCard } from "@/components/credentials/credential-card"
import { CredentialModal } from "@/components/credentials/credential-modal"
import { useCredentials } from "@/hooks/use-credentials"
import { Credential, CreateCredentialRequest } from "@/types/credentials"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"

export function CredentialsView() {
  const { credentials, loading, error, createCredential, updateCredential, deleteCredential, refreshCredentials } = useCredentials()
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [editingCredential, setEditingCredential] = useState<Credential | undefined>()
  const [deletingCredential, setDeletingCredential] = useState<string | null>(null)

  const handleAddCredential = () => {
    setEditingCredential(undefined)
    setIsModalOpen(true)
  }

  const handleEditCredential = (credential: Credential) => {
    setEditingCredential(credential)
    setIsModalOpen(true)
  }

  const handleSaveCredential = async (data: CreateCredentialRequest) => {
    if (editingCredential) {
      await updateCredential(editingCredential.id, data)
    } else {
      await createCredential(data)
    }
  }

  const handleDeleteCredential = async () => {
    if (deletingCredential) {
      await deleteCredential(deletingCredential)
      setDeletingCredential(null)
    }
  }

  if (loading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold">Credentials</h1>
            <p className="text-muted-foreground">Manage authentication credentials for your integrations</p>
          </div>
        </div>
        <div className="flex items-center justify-center h-64">
          <div className="text-center">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto"></div>
            <p className="mt-2 text-muted-foreground">Loading credentials...</p>
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
            <h1 className="text-2xl font-bold">Credentials</h1>
            <p className="text-muted-foreground">Manage authentication credentials for your integrations</p>
          </div>
        </div>
        <div className="flex items-center justify-center h-64">
          <div className="text-center">
            <p className="text-destructive mb-4">Error loading credentials: {error}</p>
            <Button onClick={refreshCredentials} variant="outline">
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
          <h1 className="text-2xl font-bold">Credentials</h1>
          <p className="text-muted-foreground">Manage authentication credentials for your integrations</p>
        </div>
        <div className="flex items-center gap-2">
          <Button onClick={refreshCredentials} variant="outline" size="sm">
            <RefreshCw className="h-4 w-4 mr-2" />
            Refresh
          </Button>
          <Button onClick={handleAddCredential}>
            <Plus className="h-4 w-4 mr-2" />
            Add Credential
          </Button>
        </div>
      </div>

      {credentials.length === 0 ? (
        <div className="flex items-center justify-center h-64 border-2 border-dashed border-gray-200 rounded-lg">
          <div className="text-center">
            <p className="text-muted-foreground mb-4">No credentials found</p>
            <Button onClick={handleAddCredential}>
              <Plus className="h-4 w-4 mr-2" />
              Add your first credential
            </Button>
          </div>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {credentials.map((credential) => (
            <CredentialCard
              key={credential.id}
              credential={credential}
              onEdit={handleEditCredential}
              onDelete={(id) => setDeletingCredential(id)}
            />
          ))}
        </div>
      )}

      <CredentialModal
        open={isModalOpen}
        onOpenChange={setIsModalOpen}
        credential={editingCredential}
        onSave={handleSaveCredential}
      />

      <AlertDialog open={!!deletingCredential} onOpenChange={() => setDeletingCredential(null)}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete Credential</AlertDialogTitle>
            <AlertDialogDescription>
              Are you sure you want to delete this credential? This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteCredential} className="bg-destructive text-destructive-foreground hover:bg-destructive/90">
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}