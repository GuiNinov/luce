use crate::repositories::CredentialRepository;
use crate::usecases::use_case::UseCase;
use async_trait::async_trait;
use luce_shared::{CredentialId, IntegrationCredential, LuceError};

pub struct GetCredentialInput {
    pub id: CredentialId,
}

impl GetCredentialInput {
    pub fn new(id: CredentialId) -> Self {
        Self { id }
    }
}

pub struct GetCredentialUseCase<R: CredentialRepository> {
    repository: R,
}

impl<R: CredentialRepository> GetCredentialUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: CredentialRepository + Send + Sync> UseCase<GetCredentialInput, IntegrationCredential> for GetCredentialUseCase<R> {
    async fn execute(&self, input: GetCredentialInput) -> Result<IntegrationCredential, LuceError> {
        self.repository.get_credential(input.id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::SqliteCredentialRepository;
    use luce_shared::{CredentialData, CreateCredentialInput, IntegrationType};

    async fn create_test_usecase() -> (GetCredentialUseCase<SqliteCredentialRepository>, CredentialId) {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        
        // Create a test credential first
        let create_input = CreateCredentialInput {
            integration_type: IntegrationType::GitHub,
            name: "Test GitHub".to_string(),
            credentials: CredentialData::GitHub {
                access_token: "ghp_test".to_string(),
                default_repo: Some("test/repo".to_string()),
                webhook_secret: Some("secret".to_string()),
            },
        };
        
        let credential = repo.create_credential(create_input).await.unwrap();
        let usecase = GetCredentialUseCase::new(repo);
        
        (usecase, credential.id)
    }

    #[tokio::test]
    async fn test_get_existing_credential() {
        let (usecase, credential_id) = create_test_usecase().await;
        let input = GetCredentialInput::new(credential_id);

        let credential = usecase.execute(input).await.unwrap();

        assert_eq!(credential.id, credential_id);
        assert_eq!(credential.integration_type, IntegrationType::GitHub);
        assert_eq!(credential.name, "Test GitHub");
    }

    #[tokio::test]
    async fn test_get_nonexistent_credential() {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        let usecase = GetCredentialUseCase::new(repo);
        let non_existent_id = uuid::Uuid::new_v4();
        let input = GetCredentialInput::new(non_existent_id);

        let result = usecase.execute(input).await;
        assert!(result.is_err());
    }
}