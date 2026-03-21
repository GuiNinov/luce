use std::sync::Arc;
use sqlx::SqlitePool;

use crate::repositories::SqliteCredentialRepository;
use crate::usecases::{
    CreateCredentialUseCaseInput, CreateCredentialUseCase, GetCredentialInput, GetCredentialUseCase,
    GetCredentialDataInput, GetCredentialDataUseCase, ListCredentialsInput, ListCredentialsUseCase,
    UpdateCredentialInput, UpdateCredentialUseCase, DeleteCredentialInput, DeleteCredentialUseCase,
    CredentialFilter, UseCase,
};
use luce_shared::{
    CredentialData, CredentialId, IntegrationCredential, IntegrationType, LuceError,
};

pub struct CredentialService {
    credential_repo: Arc<SqliteCredentialRepository>,
    create_credential_uc: CreateCredentialUseCase<Arc<SqliteCredentialRepository>>,
    get_credential_uc: GetCredentialUseCase<Arc<SqliteCredentialRepository>>,
    get_credential_data_uc: GetCredentialDataUseCase<Arc<SqliteCredentialRepository>>,
    list_credentials_uc: ListCredentialsUseCase<Arc<SqliteCredentialRepository>>,
    update_credential_uc: UpdateCredentialUseCase<Arc<SqliteCredentialRepository>>,
    delete_credential_uc: DeleteCredentialUseCase<Arc<SqliteCredentialRepository>>,
}

impl CredentialService {
    pub async fn new(database_url: &str) -> Result<Self, LuceError> {
        // Create shared repository for all operations
        let repo = Arc::new(SqliteCredentialRepository::new(database_url).await?);

        Ok(Self {
            create_credential_uc: CreateCredentialUseCase::new(repo.clone()),
            get_credential_uc: GetCredentialUseCase::new(repo.clone()),
            get_credential_data_uc: GetCredentialDataUseCase::new(repo.clone()),
            list_credentials_uc: ListCredentialsUseCase::new(repo.clone()),
            update_credential_uc: UpdateCredentialUseCase::new(repo.clone()),
            delete_credential_uc: DeleteCredentialUseCase::new(repo.clone()),
            credential_repo: repo,
        })
    }

    pub fn from_pool(pool: SqlitePool) -> Self {
        let repo = Arc::new(SqliteCredentialRepository::from_pool(pool));

        Self {
            create_credential_uc: CreateCredentialUseCase::new(repo.clone()),
            get_credential_uc: GetCredentialUseCase::new(repo.clone()),
            get_credential_data_uc: GetCredentialDataUseCase::new(repo.clone()),
            list_credentials_uc: ListCredentialsUseCase::new(repo.clone()),
            update_credential_uc: UpdateCredentialUseCase::new(repo.clone()),
            delete_credential_uc: DeleteCredentialUseCase::new(repo.clone()),
            credential_repo: repo,
        }
    }

    /// Create a new credential
    pub async fn create_credential(
        &self,
        integration_type: IntegrationType,
        name: String,
        credentials: CredentialData,
    ) -> Result<IntegrationCredential, LuceError> {
        let input = CreateCredentialUseCaseInput::new(integration_type, name, credentials);
        self.create_credential_uc.execute(input).await
    }

    /// Get a credential by ID (metadata only, no sensitive data)
    pub async fn get_credential(&self, id: CredentialId) -> Result<IntegrationCredential, LuceError> {
        let input = GetCredentialInput::new(id);
        self.get_credential_uc.execute(input).await
    }

    /// Get credential data (decrypted) - marks credential as used
    pub async fn get_credential_data(&self, id: CredentialId) -> Result<CredentialData, LuceError> {
        let input = GetCredentialDataInput::new(id);
        self.get_credential_data_uc.execute(input).await
    }

    /// List credentials with optional filtering
    pub async fn list_credentials(
        &self,
        integration_type: Option<IntegrationType>,
        active_only: bool,
    ) -> Result<Vec<IntegrationCredential>, LuceError> {
        let filter = CredentialFilter {
            integration_type,
            active_only,
        };
        let input = ListCredentialsInput::new().with_filter(filter);
        self.list_credentials_uc.execute(input).await
    }

    /// Update a credential
    pub async fn update_credential(
        &self,
        id: CredentialId,
        name: Option<String>,
        credentials: Option<CredentialData>,
        is_active: Option<bool>,
    ) -> Result<IntegrationCredential, LuceError> {
        let mut input = UpdateCredentialInput::new(id);
        
        if let Some(name) = name {
            input = input.with_name(name);
        }
        
        if let Some(credentials) = credentials {
            input = input.with_credentials(credentials);
        }
        
        if let Some(is_active) = is_active {
            input = input.with_active_status(is_active);
        }

        self.update_credential_uc.execute(input).await
    }

    /// Activate a credential
    pub async fn activate_credential(&self, id: CredentialId) -> Result<IntegrationCredential, LuceError> {
        self.update_credential(id, None, None, Some(true)).await
    }

    /// Deactivate a credential
    pub async fn deactivate_credential(&self, id: CredentialId) -> Result<IntegrationCredential, LuceError> {
        self.update_credential(id, None, None, Some(false)).await
    }

    /// Delete a credential
    pub async fn delete_credential(&self, id: CredentialId) -> Result<(), LuceError> {
        let input = DeleteCredentialInput::new(id);
        self.delete_credential_uc.execute(input).await
    }

    /// Get all active credentials for a specific integration type
    pub async fn get_active_credentials_for_type(
        &self,
        integration_type: IntegrationType,
    ) -> Result<Vec<IntegrationCredential>, LuceError> {
        self.list_credentials(Some(integration_type), true).await
    }

    /// Get the first active credential for an integration type (for convenience)
    pub async fn get_first_active_credential_for_type(
        &self,
        integration_type: IntegrationType,
    ) -> Result<Option<IntegrationCredential>, LuceError> {
        let credentials = self.get_active_credentials_for_type(integration_type).await?;
        Ok(credentials.into_iter().next())
    }

    /// Get credential data for the first active credential of a type
    pub async fn get_first_credential_data_for_type(
        &self,
        integration_type: IntegrationType,
    ) -> Result<Option<CredentialData>, LuceError> {
        if let Some(credential) = self.get_first_active_credential_for_type(integration_type).await? {
            let data = self.get_credential_data(credential.id).await?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    /// Check if there are any active credentials for an integration type
    pub async fn has_active_credentials_for_type(
        &self,
        integration_type: IntegrationType,
    ) -> Result<bool, LuceError> {
        let credentials = self.get_active_credentials_for_type(integration_type).await?;
        Ok(!credentials.is_empty())
    }

    /// Get summary statistics
    pub async fn get_credential_stats(&self) -> Result<CredentialStats, LuceError> {
        let all_credentials = self.list_credentials(None, false).await?;
        
        let total_count = all_credentials.len();
        let active_count = all_credentials.iter().filter(|c| c.is_active).count();
        let inactive_count = total_count - active_count;
        
        let github_count = all_credentials.iter().filter(|c| c.integration_type == IntegrationType::GitHub).count();
        let slack_count = all_credentials.iter().filter(|c| c.integration_type == IntegrationType::Slack).count();
        let linear_count = all_credentials.iter().filter(|c| c.integration_type == IntegrationType::Linear).count();

        Ok(CredentialStats {
            total_count,
            active_count,
            inactive_count,
            github_count,
            slack_count,
            linear_count,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CredentialStats {
    pub total_count: usize,
    pub active_count: usize,
    pub inactive_count: usize,
    pub github_count: usize,
    pub slack_count: usize,
    pub linear_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use luce_shared::CredentialData;

    async fn create_test_service() -> CredentialService {
        CredentialService::new("sqlite::memory:").await.unwrap()
    }

    #[tokio::test]
    async fn test_credential_service_lifecycle() {
        let service = create_test_service().await;

        // Create GitHub credential
        let github_data = CredentialData::GitHub {
            access_token: "ghp_test_token".to_string(),
            default_repo: Some("test/repo".to_string()),
            webhook_secret: Some("webhook_secret".to_string()),
        };

        let credential = service
            .create_credential(IntegrationType::GitHub, "Test GitHub".to_string(), github_data.clone())
            .await
            .unwrap();

        assert_eq!(credential.integration_type, IntegrationType::GitHub);
        assert_eq!(credential.name, "Test GitHub");
        assert!(credential.is_active);

        // Get credential metadata
        let retrieved = service.get_credential(credential.id).await.unwrap();
        assert_eq!(retrieved.id, credential.id);
        assert_eq!(retrieved.name, "Test GitHub");

        // Get credential data
        let data = service.get_credential_data(credential.id).await.unwrap();
        assert_eq!(data, github_data);

        // Update credential
        let updated = service
            .update_credential(credential.id, Some("Updated GitHub".to_string()), None, None)
            .await
            .unwrap();
        assert_eq!(updated.name, "Updated GitHub");

        // List credentials
        let credentials = service.list_credentials(None, true).await.unwrap();
        assert_eq!(credentials.len(), 1);
        assert_eq!(credentials[0].id, credential.id);

        // Deactivate credential
        let deactivated = service.deactivate_credential(credential.id).await.unwrap();
        assert!(!deactivated.is_active);

        // Delete credential
        service.delete_credential(credential.id).await.unwrap();
        
        // Verify it's gone
        let result = service.get_credential(credential.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_credential_service_by_type() {
        let service = create_test_service().await;

        // Create GitHub credential
        let github_data = CredentialData::GitHub {
            access_token: "ghp_github".to_string(),
            default_repo: None,
            webhook_secret: None,
        };
        service
            .create_credential(IntegrationType::GitHub, "GitHub".to_string(), github_data.clone())
            .await
            .unwrap();

        // Create Slack credential
        let slack_data = CredentialData::Slack {
            bot_token: "xoxb-slack".to_string(),
            user_token: None,
            workspace: "test".to_string(),
        };
        service
            .create_credential(IntegrationType::Slack, "Slack".to_string(), slack_data.clone())
            .await
            .unwrap();

        // Test getting by type
        let github_creds = service.get_active_credentials_for_type(IntegrationType::GitHub).await.unwrap();
        assert_eq!(github_creds.len(), 1);
        assert_eq!(github_creds[0].integration_type, IntegrationType::GitHub);

        let slack_creds = service.get_active_credentials_for_type(IntegrationType::Slack).await.unwrap();
        assert_eq!(slack_creds.len(), 1);
        assert_eq!(slack_creds[0].integration_type, IntegrationType::Slack);

        // Test first credential for type
        let first_github = service.get_first_active_credential_for_type(IntegrationType::GitHub).await.unwrap();
        assert!(first_github.is_some());
        assert_eq!(first_github.unwrap().integration_type, IntegrationType::GitHub);

        // Test first credential data for type
        let first_github_data = service.get_first_credential_data_for_type(IntegrationType::GitHub).await.unwrap();
        assert!(first_github_data.is_some());
        assert_eq!(first_github_data.unwrap(), github_data);

        // Test has credentials check
        let has_github = service.has_active_credentials_for_type(IntegrationType::GitHub).await.unwrap();
        assert!(has_github);

        let has_linear = service.has_active_credentials_for_type(IntegrationType::Linear).await.unwrap();
        assert!(!has_linear);
    }

    #[tokio::test]
    async fn test_credential_stats() {
        let service = create_test_service().await;

        // Create multiple credentials
        let github_data = CredentialData::GitHub {
            access_token: "ghp_1".to_string(),
            default_repo: None,
            webhook_secret: None,
        };
        let github_cred = service
            .create_credential(IntegrationType::GitHub, "GitHub 1".to_string(), github_data)
            .await
            .unwrap();

        let slack_data = CredentialData::Slack {
            bot_token: "xoxb-1".to_string(),
            user_token: None,
            workspace: "workspace1".to_string(),
        };
        service
            .create_credential(IntegrationType::Slack, "Slack 1".to_string(), slack_data)
            .await
            .unwrap();

        // Deactivate GitHub credential
        service.deactivate_credential(github_cred.id).await.unwrap();

        let stats = service.get_credential_stats().await.unwrap();
        
        assert_eq!(stats.total_count, 2);
        assert_eq!(stats.active_count, 1);
        assert_eq!(stats.inactive_count, 1);
        assert_eq!(stats.github_count, 1);
        assert_eq!(stats.slack_count, 1);
        assert_eq!(stats.linear_count, 0);
    }
}