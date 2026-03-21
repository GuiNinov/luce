use async_trait::async_trait;
use chrono::{DateTime, Utc};
use luce_shared::{
    CredentialData, CredentialId, CreateCredentialInput,
    IntegrationCredential, IntegrationType, LuceError, UpdateCredentialInput,
};
use serde_json;
use sqlx::{Row, SqlitePool};
use std::io;
use uuid::Uuid;

use super::CredentialRepository;

/// Repository for managing integration credentials with encryption
pub struct SqliteCredentialRepository {
    pool: SqlitePool,
    encryption_key: String, // In production, use proper key management
}

impl SqliteCredentialRepository {
    pub async fn new(database_url: &str) -> Result<Self, LuceError> {
        let pool = SqlitePool::connect(database_url)
            .await
            .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?;

        // For tests using in-memory database, create the table if it doesn't exist
        if database_url == "sqlite::memory:" {
            sqlx::query(r#"
                CREATE TABLE IF NOT EXISTS integration_credentials (
                    id TEXT PRIMARY KEY,
                    integration_type TEXT NOT NULL,
                    name TEXT NOT NULL,
                    encrypted_data TEXT NOT NULL,
                    is_active BOOLEAN NOT NULL DEFAULT 1,
                    created_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    last_used_at TEXT
                );
            "#)
            .execute(&pool)
            .await
            .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?;
        }

        Ok(Self {
            pool,
            encryption_key: std::env::var("LUCE_ENCRYPTION_KEY")
                .unwrap_or_else(|_| "default-key-change-in-production".to_string()),
        })
    }

    // Simple encryption - in production use proper encryption library
    fn encrypt(&self, data: &str) -> String {
        // TODO: Implement proper encryption with AES or similar
        // For now, just base64 encode (NOT secure)
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.encode(data)
    }

    fn decrypt(&self, encrypted_data: &str) -> Result<String, LuceError> {
        // TODO: Implement proper decryption
        // For now, just base64 decode (NOT secure)
        use base64::{Engine as _, engine::general_purpose};
        general_purpose::STANDARD.decode(encrypted_data)
            .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))
            .and_then(|bytes| {
                String::from_utf8(bytes)
                    .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))
            })
    }

    // Credential CRUD operations
    pub async fn create_credential(
        &self,
        input: CreateCredentialInput,
    ) -> Result<IntegrationCredential, LuceError> {
        let credential_json = serde_json::to_string(&input.credentials)
            .map_err(LuceError::SerializationError)?;
        
        let encrypted_data = self.encrypt(&credential_json);
        
        let credential = IntegrationCredential::new(
            input.integration_type,
            input.name,
            encrypted_data,
        );

        let query = r#"
            INSERT INTO integration_credentials 
            (id, integration_type, name, encrypted_data, is_active, created_at, updated_at, last_used_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(credential.id.to_string())
            .bind(credential.integration_type.to_string())
            .bind(&credential.name)
            .bind(&credential.encrypted_data)
            .bind(credential.is_active)
            .bind(credential.created_at.to_rfc3339())
            .bind(credential.updated_at.to_rfc3339())
            .bind(credential.last_used_at.map(|dt| dt.to_rfc3339()))
            .execute(&self.pool)
            .await
            .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?;

        Ok(credential)
    }

    pub async fn get_credential(&self, id: CredentialId) -> Result<IntegrationCredential, LuceError> {
        let query = r#"
            SELECT id, integration_type, name, encrypted_data, is_active, 
                   created_at, updated_at, last_used_at
            FROM integration_credentials 
            WHERE id = ?
        "#;

        let row = sqlx::query(query)
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?
            .ok_or_else(|| LuceError::TaskNotFound { id: id })?;

        self.row_to_credential(row)
    }

    pub async fn list_credentials(
        &self,
        integration_type: Option<IntegrationType>,
        active_only: bool,
    ) -> Result<Vec<IntegrationCredential>, LuceError> {
        let mut query = "SELECT id, integration_type, name, encrypted_data, is_active, created_at, updated_at, last_used_at FROM integration_credentials WHERE 1=1".to_string();
        let mut params: Vec<String> = vec![];

        if let Some(int_type) = integration_type {
            query.push_str(" AND integration_type = ?");
            params.push(int_type.to_string());
        }

        if active_only {
            query.push_str(" AND is_active = 1");
        }

        query.push_str(" ORDER BY created_at DESC");

        let mut sql_query = sqlx::query(&query);
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let rows = sql_query
            .fetch_all(&self.pool)
            .await
            .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?;

        let mut credentials = Vec::new();
        for row in rows {
            credentials.push(self.row_to_credential(row)?);
        }

        Ok(credentials)
    }

    pub async fn update_credential(
        &self,
        id: CredentialId,
        input: UpdateCredentialInput,
    ) -> Result<IntegrationCredential, LuceError> {
        let mut credential = self.get_credential(id).await?;

        if let Some(name) = input.name {
            credential.name = name;
        }

        if let Some(credentials) = input.credentials {
            let credential_json = serde_json::to_string(&credentials)
                .map_err(LuceError::SerializationError)?;
            credential.encrypted_data = self.encrypt(&credential_json);
        }

        if let Some(is_active) = input.is_active {
            credential.is_active = is_active;
        }

        credential.updated_at = Utc::now();

        let query = r#"
            UPDATE integration_credentials 
            SET name = ?, encrypted_data = ?, is_active = ?, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(&credential.name)
            .bind(&credential.encrypted_data)
            .bind(credential.is_active)
            .bind(credential.updated_at.to_rfc3339())
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?;

        Ok(credential)
    }

    pub async fn delete_credential(&self, id: CredentialId) -> Result<(), LuceError> {
        let query = "DELETE FROM integration_credentials WHERE id = ?";

        let result = sqlx::query(query)
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?;

        if result.rows_affected() == 0 {
            return Err(LuceError::TaskNotFound { id });
        }

        Ok(())
    }

    pub async fn mark_credential_used(&self, id: CredentialId) -> Result<(), LuceError> {
        let query = "UPDATE integration_credentials SET last_used_at = ?, updated_at = ? WHERE id = ?";

        let now = Utc::now();
        sqlx::query(query)
            .bind(now.to_rfc3339())
            .bind(now.to_rfc3339())
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?;

        Ok(())
    }

    // Get decrypted credential data for use
    pub async fn get_credential_data(&self, id: CredentialId) -> Result<CredentialData, LuceError> {
        let credential = self.get_credential(id).await?;
        let decrypted_json = self.decrypt(&credential.encrypted_data)?;
        let credential_data: CredentialData = serde_json::from_str(&decrypted_json)
            .map_err(LuceError::SerializationError)?;

        // Mark as used
        self.mark_credential_used(id).await?;

        Ok(credential_data)
    }

    fn row_to_credential(&self, row: sqlx::sqlite::SqliteRow) -> Result<IntegrationCredential, LuceError> {
        let id_str: String = row.get("id");
        let integration_type_str: String = row.get("integration_type");
        let created_at_str: String = row.get("created_at");
        let updated_at_str: String = row.get("updated_at");
        let last_used_at_str: Option<String> = row.get("last_used_at");

        Ok(IntegrationCredential {
            id: Uuid::parse_str(&id_str)
                .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?,
            integration_type: integration_type_str.parse()
                .map_err(|e: String| LuceError::IoError(io::Error::other(e)))?,
            name: row.get("name"),
            encrypted_data: row.get("encrypted_data"),
            is_active: row.get("is_active"),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?
                .with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                .map_err(|e| LuceError::IoError(io::Error::other(e.to_string())))?
                .with_timezone(&Utc),
            last_used_at: last_used_at_str
                .map(|s| DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| LuceError::IoError(io::Error::other(e.to_string()))))
                .transpose()?
                .map(|dt| dt.with_timezone(&Utc)),
        })
    }
}

#[async_trait]
impl CredentialRepository for SqliteCredentialRepository {
    async fn create_credential(&self, input: CreateCredentialInput) -> Result<IntegrationCredential, LuceError> {
        self.create_credential(input).await
    }

    async fn get_credential(&self, id: CredentialId) -> Result<IntegrationCredential, LuceError> {
        self.get_credential(id).await
    }

    async fn list_credentials(&self, integration_type: Option<IntegrationType>, active_only: bool) -> Result<Vec<IntegrationCredential>, LuceError> {
        self.list_credentials(integration_type, active_only).await
    }

    async fn update_credential(&self, id: CredentialId, input: UpdateCredentialInput) -> Result<IntegrationCredential, LuceError> {
        self.update_credential(id, input).await
    }

    async fn delete_credential(&self, id: CredentialId) -> Result<(), LuceError> {
        self.delete_credential(id).await
    }

    async fn mark_credential_used(&self, id: CredentialId) -> Result<(), LuceError> {
        self.mark_credential_used(id).await
    }

    async fn get_credential_data(&self, id: CredentialId) -> Result<CredentialData, LuceError> {
        self.get_credential_data(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_repo() -> SqliteCredentialRepository {
        SqliteCredentialRepository::new("sqlite::memory:")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_encryption_roundtrip() {
        let repo = create_test_repo().await;
        let original = r#"{"access_token": "secret123", "repo": "test/repo"}"#;
        
        let encrypted = repo.encrypt(original);
        let decrypted = repo.decrypt(&encrypted).unwrap();
        
        assert_eq!(original, decrypted);
        assert_ne!(original, encrypted); // Should be different when encrypted
    }

    #[tokio::test]
    async fn test_credential_crud() {
        let repo = create_test_repo().await;
        
        // Create credential
        let input = CreateCredentialInput {
            integration_type: IntegrationType::GitHub,
            name: "Test GitHub".to_string(),
            credentials: CredentialData::GitHub {
                access_token: "token123".to_string(),
                default_repo: Some("owner/repo".to_string()),
                webhook_secret: Some("secret".to_string()),
            },
        };

        let created = repo.create_credential(input).await.unwrap();
        assert_eq!(created.name, "Test GitHub");
        assert_eq!(created.integration_type, IntegrationType::GitHub);
        assert!(created.is_active);

        // Get credential
        let retrieved = repo.get_credential(created.id).await.unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, created.name);

        // Update credential
        let update_input = UpdateCredentialInput {
            name: Some("Updated GitHub".to_string()),
            credentials: None,
            is_active: Some(false),
        };

        let updated = repo.update_credential(created.id, update_input).await.unwrap();
        assert_eq!(updated.name, "Updated GitHub");
        assert!(!updated.is_active);

        // List credentials
        let credentials = repo.list_credentials(None, false).await.unwrap();
        assert_eq!(credentials.len(), 1);

        // Delete credential
        repo.delete_credential(created.id).await.unwrap();
        
        let result = repo.get_credential(created.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_credential_data() {
        let repo = create_test_repo().await;
        
        let credentials = CredentialData::GitHub {
            access_token: "token123".to_string(),
            default_repo: Some("owner/repo".to_string()),
            webhook_secret: Some("secret".to_string()),
        };

        let input = CreateCredentialInput {
            integration_type: IntegrationType::GitHub,
            name: "Test".to_string(),
            credentials: credentials.clone(),
        };

        let created = repo.create_credential(input).await.unwrap();
        
        // Get decrypted data
        let decrypted = repo.get_credential_data(created.id).await.unwrap();
        assert_eq!(decrypted, credentials);

        // Verify last_used_at was updated
        let updated_credential = repo.get_credential(created.id).await.unwrap();
        assert!(updated_credential.last_used_at.is_some());
    }
}