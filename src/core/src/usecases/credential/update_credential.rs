use crate::repositories::CredentialRepository;
use crate::usecases::use_case::UseCase;
use async_trait::async_trait;
use luce_shared::{CredentialData, CredentialId, IntegrationCredential, LuceError, UpdateCredentialInput as SharedUpdateCredentialInput};

pub struct UpdateCredentialInput {
    pub id: CredentialId,
    pub name: Option<String>,
    pub credentials: Option<CredentialData>,
    pub is_active: Option<bool>,
}

impl UpdateCredentialInput {
    pub fn new(id: CredentialId) -> Self {
        Self {
            id,
            name: None,
            credentials: None,
            is_active: None,
        }
    }

    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn with_credentials(mut self, credentials: CredentialData) -> Self {
        self.credentials = Some(credentials);
        self
    }

    pub fn with_active_status(mut self, is_active: bool) -> Self {
        self.is_active = Some(is_active);
        self
    }

    pub fn activate(self) -> Self {
        self.with_active_status(true)
    }

    pub fn deactivate(self) -> Self {
        self.with_active_status(false)
    }
}

pub struct UpdateCredentialUseCase<R: CredentialRepository> {
    repository: R,
}

impl<R: CredentialRepository> UpdateCredentialUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: CredentialRepository + Send + Sync> UseCase<UpdateCredentialInput, IntegrationCredential> for UpdateCredentialUseCase<R> {
    async fn execute(&self, input: UpdateCredentialInput) -> Result<IntegrationCredential, LuceError> {
        let update_input = SharedUpdateCredentialInput {
            name: input.name,
            credentials: input.credentials,
            is_active: input.is_active,
        };

        self.repository.update_credential(input.id, update_input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::SqliteCredentialRepository;
    use luce_shared::{CredentialData, CreateCredentialInput, IntegrationType};

    async fn create_test_usecase() -> (UpdateCredentialUseCase<SqliteCredentialRepository>, CredentialId) {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        
        // Create a test credential
        let create_input = CreateCredentialInput {
            integration_type: IntegrationType::GitHub,
            name: "Original GitHub".to_string(),
            credentials: CredentialData::GitHub {
                access_token: "ghp_original".to_string(),
                default_repo: Some("original/repo".to_string()),
                webhook_secret: None,
            },
        };
        
        let credential = repo.create_credential(create_input).await.unwrap();
        let usecase = UpdateCredentialUseCase::new(repo);
        
        (usecase, credential.id)
    }

    #[tokio::test]
    async fn test_update_credential_name() {
        let (usecase, credential_id) = create_test_usecase().await;
        let input = UpdateCredentialInput::new(credential_id)
            .with_name("Updated GitHub".to_string());

        let updated_credential = usecase.execute(input).await.unwrap();

        assert_eq!(updated_credential.id, credential_id);
        assert_eq!(updated_credential.name, "Updated GitHub");
        assert!(updated_credential.is_active);
    }

    #[tokio::test]
    async fn test_update_credential_credentials() {
        let (usecase, credential_id) = create_test_usecase().await;
        let new_credentials = CredentialData::GitHub {
            access_token: "ghp_new_token".to_string(),
            default_repo: Some("new/repo".to_string()),
            webhook_secret: Some("new_secret".to_string()),
        };
        
        let input = UpdateCredentialInput::new(credential_id)
            .with_credentials(new_credentials);

        let updated_credential = usecase.execute(input).await.unwrap();

        assert_eq!(updated_credential.id, credential_id);
        // Note: We can't easily verify the encrypted credentials changed without decryption
        assert!(updated_credential.is_active);
    }

    #[tokio::test]
    async fn test_deactivate_credential() {
        let (usecase, credential_id) = create_test_usecase().await;
        let input = UpdateCredentialInput::new(credential_id).deactivate();

        let updated_credential = usecase.execute(input).await.unwrap();

        assert_eq!(updated_credential.id, credential_id);
        assert!(!updated_credential.is_active);
    }

    #[tokio::test]
    async fn test_update_multiple_fields() {
        let (usecase, credential_id) = create_test_usecase().await;
        let new_credentials = CredentialData::GitHub {
            access_token: "ghp_multi_update".to_string(),
            default_repo: None,
            webhook_secret: Some("multi_secret".to_string()),
        };
        
        let input = UpdateCredentialInput::new(credential_id)
            .with_name("Multi-Updated GitHub".to_string())
            .with_credentials(new_credentials)
            .deactivate();

        let updated_credential = usecase.execute(input).await.unwrap();

        assert_eq!(updated_credential.id, credential_id);
        assert_eq!(updated_credential.name, "Multi-Updated GitHub");
        assert!(!updated_credential.is_active);
    }

    #[tokio::test]
    async fn test_update_nonexistent_credential() {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        let usecase = UpdateCredentialUseCase::new(repo);
        let non_existent_id = uuid::Uuid::new_v4();
        let input = UpdateCredentialInput::new(non_existent_id)
            .with_name("Should Fail".to_string());

        let result = usecase.execute(input).await;
        assert!(result.is_err());
    }
}