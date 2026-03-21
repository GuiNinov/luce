use crate::repositories::CredentialRepository;
use crate::usecases::use_case::UseCase;
use async_trait::async_trait;
use luce_shared::{CredentialData, CredentialId, LuceError};

pub struct GetCredentialDataInput {
    pub id: CredentialId,
}

impl GetCredentialDataInput {
    pub fn new(id: CredentialId) -> Self {
        Self { id }
    }
}

pub struct GetCredentialDataUseCase<R: CredentialRepository> {
    repository: R,
}

impl<R: CredentialRepository> GetCredentialDataUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: CredentialRepository + Send + Sync> UseCase<GetCredentialDataInput, CredentialData> for GetCredentialDataUseCase<R> {
    async fn execute(&self, input: GetCredentialDataInput) -> Result<CredentialData, LuceError> {
        // This will mark the credential as used and return the decrypted data
        self.repository.get_credential_data(input.id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::SqliteCredentialRepository;
    use luce_shared::{CredentialData, CreateCredentialInput, IntegrationType};

    async fn create_test_usecase() -> (GetCredentialDataUseCase<SqliteCredentialRepository>, CredentialId, CredentialData) {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        
        let credentials = CredentialData::GitHub {
            access_token: "ghp_secret_token".to_string(),
            default_repo: Some("secret/repo".to_string()),
            webhook_secret: Some("webhook_secret_123".to_string()),
        };
        
        // Create a test credential
        let create_input = CreateCredentialInput {
            integration_type: IntegrationType::GitHub,
            name: "Secret GitHub".to_string(),
            credentials: credentials.clone(),
        };
        
        let credential = repo.create_credential(create_input).await.unwrap();
        let usecase = GetCredentialDataUseCase::new(repo);
        
        (usecase, credential.id, credentials)
    }

    #[tokio::test]
    async fn test_get_credential_data_github() {
        let (usecase, credential_id, expected_data) = create_test_usecase().await;
        let input = GetCredentialDataInput::new(credential_id);

        let credential_data = usecase.execute(input).await.unwrap();

        assert_eq!(credential_data, expected_data);

        // Verify it's the correct GitHub data
        if let CredentialData::GitHub { access_token, default_repo, webhook_secret } = credential_data {
            assert_eq!(access_token, "ghp_secret_token");
            assert_eq!(default_repo, Some("secret/repo".to_string()));
            assert_eq!(webhook_secret, Some("webhook_secret_123".to_string()));
        } else {
            panic!("Expected GitHub credential data");
        }
    }

    #[tokio::test]
    async fn test_get_credential_data_slack() {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        
        let credentials = CredentialData::Slack {
            bot_token: "xoxb-secret-bot-token".to_string(),
            user_token: Some("xoxp-secret-user-token".to_string()),
            workspace: "secret-workspace".to_string(),
        };
        
        let create_input = CreateCredentialInput {
            integration_type: IntegrationType::Slack,
            name: "Secret Slack".to_string(),
            credentials: credentials.clone(),
        };
        
        let credential = repo.create_credential(create_input).await.unwrap();
        let usecase = GetCredentialDataUseCase::new(repo);
        let input = GetCredentialDataInput::new(credential.id);

        let credential_data = usecase.execute(input).await.unwrap();

        assert_eq!(credential_data, credentials);

        // Verify it's the correct Slack data
        if let CredentialData::Slack { bot_token, user_token, workspace } = credential_data {
            assert_eq!(bot_token, "xoxb-secret-bot-token");
            assert_eq!(user_token, Some("xoxp-secret-user-token".to_string()));
            assert_eq!(workspace, "secret-workspace");
        } else {
            panic!("Expected Slack credential data");
        }
    }

    #[tokio::test]
    async fn test_get_credential_data_marks_as_used() {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        
        let credentials = CredentialData::GitHub {
            access_token: "ghp_usage_test".to_string(),
            default_repo: None,
            webhook_secret: None,
        };
        
        let create_input = CreateCredentialInput {
            integration_type: IntegrationType::GitHub,
            name: "Usage Test".to_string(),
            credentials: credentials.clone(),
        };
        
        let credential = repo.create_credential(create_input).await.unwrap();
        
        // Initially, last_used_at should be None
        assert!(credential.last_used_at.is_none());
        
        let usecase = GetCredentialDataUseCase::new(repo);
        let input = GetCredentialDataInput::new(credential.id);

        let _credential_data = usecase.execute(input).await.unwrap();

        // After getting data, the credential should be marked as used
        // We need to get the credential again to check the updated timestamp
        use crate::usecases::credential::get_credential::{GetCredentialInput, GetCredentialUseCase};
        let get_usecase = GetCredentialUseCase::new(usecase.repository);
        let get_input = GetCredentialInput::new(credential.id);
        let updated_credential = get_usecase.execute(get_input).await.unwrap();

        assert!(updated_credential.last_used_at.is_some());
    }

    #[tokio::test]
    async fn test_get_nonexistent_credential_data() {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        let usecase = GetCredentialDataUseCase::new(repo);
        let non_existent_id = uuid::Uuid::new_v4();
        let input = GetCredentialDataInput::new(non_existent_id);

        let result = usecase.execute(input).await;
        assert!(result.is_err());
    }
}