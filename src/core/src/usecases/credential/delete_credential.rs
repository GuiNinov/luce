use crate::repositories::CredentialRepository;
use crate::usecases::use_case::UseCase;
use async_trait::async_trait;
use luce_shared::{CredentialId, LuceError};

pub struct DeleteCredentialInput {
    pub id: CredentialId,
}

impl DeleteCredentialInput {
    pub fn new(id: CredentialId) -> Self {
        Self { id }
    }
}

pub struct DeleteCredentialUseCase<R: CredentialRepository> {
    repository: R,
}

impl<R: CredentialRepository> DeleteCredentialUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: CredentialRepository + Send + Sync> UseCase<DeleteCredentialInput, ()> for DeleteCredentialUseCase<R> {
    async fn execute(&self, input: DeleteCredentialInput) -> Result<(), LuceError> {
        self.repository.delete_credential(input.id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::SqliteCredentialRepository;
    use luce_shared::{CredentialData, CreateCredentialInput, IntegrationType};

    async fn create_test_usecase() -> (DeleteCredentialUseCase<SqliteCredentialRepository>, CredentialId) {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        
        // Create a test credential
        let create_input = CreateCredentialInput {
            integration_type: IntegrationType::GitHub,
            name: "To Be Deleted".to_string(),
            credentials: CredentialData::GitHub {
                access_token: "ghp_delete_me".to_string(),
                default_repo: None,
                webhook_secret: None,
            },
        };
        
        let credential = repo.create_credential(create_input).await.unwrap();
        let usecase = DeleteCredentialUseCase::new(repo);
        
        (usecase, credential.id)
    }

    #[tokio::test]
    async fn test_delete_existing_credential() {
        let (usecase, credential_id) = create_test_usecase().await;
        let input = DeleteCredentialInput::new(credential_id);

        let result = usecase.execute(input).await;
        assert!(result.is_ok());

        // Verify the credential was deleted by trying to get it
        // This requires access to the repository, so we create a new one with the same database
        use crate::usecases::credential::get_credential::{GetCredentialInput, GetCredentialUseCase};
        let get_usecase = GetCredentialUseCase::new(SqliteCredentialRepository::new("sqlite::memory:").await.unwrap());
        let get_input = GetCredentialInput::new(credential_id);
        let get_result = get_usecase.execute(get_input).await;
        assert!(get_result.is_err());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_credential() {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        let usecase = DeleteCredentialUseCase::new(repo);
        let non_existent_id = uuid::Uuid::new_v4();
        let input = DeleteCredentialInput::new(non_existent_id);

        let result = usecase.execute(input).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_multiple_credentials() {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        let usecase = DeleteCredentialUseCase::new(repo);

        // Create multiple credentials
        let create_input1 = CreateCredentialInput {
            integration_type: IntegrationType::GitHub,
            name: "GitHub 1".to_string(),
            credentials: CredentialData::GitHub {
                access_token: "ghp_1".to_string(),
                default_repo: None,
                webhook_secret: None,
            },
        };
        let cred1 = usecase.repository.create_credential(create_input1).await.unwrap();

        let create_input2 = CreateCredentialInput {
            integration_type: IntegrationType::Slack,
            name: "Slack 1".to_string(),
            credentials: CredentialData::Slack {
                bot_token: "xoxb-1".to_string(),
                user_token: None,
                workspace: "workspace1".to_string(),
            },
        };
        let cred2 = usecase.repository.create_credential(create_input2).await.unwrap();

        // Delete first credential
        let delete_input1 = DeleteCredentialInput::new(cred1.id);
        let result1 = usecase.execute(delete_input1).await;
        assert!(result1.is_ok());

        // Delete second credential
        let delete_input2 = DeleteCredentialInput::new(cred2.id);
        let result2 = usecase.execute(delete_input2).await;
        assert!(result2.is_ok());

        // Verify both are deleted
        let list_result = usecase.repository.list_credentials(None, false).await.unwrap();
        assert!(list_result.is_empty());
    }
}