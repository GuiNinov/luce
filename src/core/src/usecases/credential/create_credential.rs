use crate::repositories::CredentialRepository;
use crate::usecases::use_case::UseCase;
use async_trait::async_trait;
use luce_shared::{CredentialData, CreateCredentialInput as SharedCreateCredentialInput, IntegrationCredential, IntegrationType, LuceError};

pub struct CreateCredentialUseCaseInput {
    pub integration_type: IntegrationType,
    pub name: String,
    pub credentials: CredentialData,
}

impl CreateCredentialUseCaseInput {
    pub fn new(integration_type: IntegrationType, name: String, credentials: CredentialData) -> Self {
        Self {
            integration_type,
            name,
            credentials,
        }
    }
}

pub struct CreateCredentialUseCase<R: CredentialRepository> {
    repository: R,
}

impl<R: CredentialRepository> CreateCredentialUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: CredentialRepository + Send + Sync> UseCase<CreateCredentialUseCaseInput, IntegrationCredential> for CreateCredentialUseCase<R> {
    async fn execute(&self, input: CreateCredentialUseCaseInput) -> Result<IntegrationCredential, LuceError> {
        let create_input = SharedCreateCredentialInput {
            integration_type: input.integration_type,
            name: input.name,
            credentials: input.credentials,
        };

        self.repository.create_credential(create_input).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::SqliteCredentialRepository;
    use luce_shared::CredentialData;

    async fn create_test_usecase() -> CreateCredentialUseCase<SqliteCredentialRepository> {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        CreateCredentialUseCase::new(repo)
    }

    #[tokio::test]
    async fn test_create_github_credential() {
        let usecase = create_test_usecase().await;
        let input = CreateCredentialUseCaseInput::new(
            IntegrationType::GitHub,
            "GitHub Work Account".to_string(),
            CredentialData::GitHub {
                access_token: "ghp_test_token".to_string(),
                default_repo: Some("company/repo".to_string()),
                webhook_secret: Some("webhook_secret".to_string()),
            },
        );

        let credential = usecase.execute(input).await.unwrap();

        assert_eq!(credential.integration_type, IntegrationType::GitHub);
        assert_eq!(credential.name, "GitHub Work Account");
        assert!(credential.is_active);
    }

    #[tokio::test]
    async fn test_create_slack_credential() {
        let usecase = create_test_usecase().await;
        let input = CreateCredentialUseCaseInput::new(
            IntegrationType::Slack,
            "Company Slack".to_string(),
            CredentialData::Slack {
                bot_token: "xoxb-test-token".to_string(),
                user_token: Some("xoxp-user-token".to_string()),
                workspace: "company-workspace".to_string(),
            },
        );

        let credential = usecase.execute(input).await.unwrap();

        assert_eq!(credential.integration_type, IntegrationType::Slack);
        assert_eq!(credential.name, "Company Slack");
        assert!(credential.is_active);
    }
}