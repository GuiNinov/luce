import { useState, useEffect } from 'react'
import { apiService } from '@/services/api'
import { Credential, CreateCredentialRequest, UpdateCredentialRequest } from '@/types/credentials'

export function useCredentials() {
  const [credentials, setCredentials] = useState<Credential[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const loadCredentials = async () => {
    try {
      setLoading(true)
      setError(null)
      const credList = await apiService.listCredentials()
      setCredentials(credList)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load credentials')
    } finally {
      setLoading(false)
    }
  }

  const createCredential = async (request: CreateCredentialRequest) => {
    try {
      const newCredential = await apiService.createCredential(request)
      setCredentials(prev => [...prev, newCredential])
      return newCredential
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to create credential'
      setError(errorMsg)
      throw new Error(errorMsg)
    }
  }

  const updateCredential = async (id: string, request: UpdateCredentialRequest) => {
    try {
      const updatedCredential = await apiService.updateCredential(id, request)
      setCredentials(prev => 
        prev.map(cred => cred.id === id ? updatedCredential : cred)
      )
      return updatedCredential
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to update credential'
      setError(errorMsg)
      throw new Error(errorMsg)
    }
  }

  const deleteCredential = async (id: string) => {
    try {
      await apiService.deleteCredential(id)
      setCredentials(prev => prev.filter(cred => cred.id !== id))
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : 'Failed to delete credential'
      setError(errorMsg)
      throw new Error(errorMsg)
    }
  }

  const refreshCredentials = () => {
    loadCredentials()
  }

  useEffect(() => {
    loadCredentials()
  }, [])

  return {
    credentials,
    loading,
    error,
    createCredential,
    updateCredential,
    deleteCredential,
    refreshCredentials
  }
}