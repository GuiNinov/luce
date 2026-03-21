use crate::repositories::CredentialRepository;
use crate::usecases::use_case::UseCase;
use async_trait::async_trait;
use luce_shared::{IntegrationCredential, IntegrationType, LuceError};

#[derive(Debug, Clone)]
pub struct CredentialFilter {
    pub integration_type: Option<IntegrationType>,
    pub active_only: bool,
}

impl Default for CredentialFilter {
    fn default() -> Self {
        Self {
            integration_type: None,
            active_only: true,
        }
    }
}

pub struct ListCredentialsInput {
    pub filter: Option<CredentialFilter>,
}

impl ListCredentialsInput {
    pub fn new() -> Self {
        Self { filter: None }
    }

    pub fn with_filter(mut self, filter: CredentialFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn with_integration_type(mut self, integration_type: IntegrationType) -> Self {
        let mut filter = self.filter.unwrap_or_default();
        filter.integration_type = Some(integration_type);
        self.filter = Some(filter);
        self
    }

    pub fn include_inactive(mut self) -> Self {
        let mut filter = self.filter.unwrap_or_default();
        filter.active_only = false;
        self.filter = Some(filter);
        self
    }
}

pub struct ListCredentialsUseCase<R: CredentialRepository> {
    repository: R,
}

impl<R: CredentialRepository> ListCredentialsUseCase<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: CredentialRepository + Send + Sync> UseCase<ListCredentialsInput, Vec<IntegrationCredential>> for ListCredentialsUseCase<R> {
    async fn execute(&self, input: ListCredentialsInput) -> Result<Vec<IntegrationCredential>, LuceError> {
        let filter = input.filter.unwrap_or_default();
        
        self.repository.list_credentials(
            filter.integration_type,
            filter.active_only,
        ).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::SqliteCredentialRepository;
    use luce_shared::{CredentialData, CreateCredentialInput, IntegrationType};

    async fn create_test_usecase_with_data() -> ListCredentialsUseCase<SqliteCredentialRepository> {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        
        // Create GitHub credential
        let github_input = CreateCredentialInput {
            integration_type: IntegrationType::GitHub,
            name: "GitHub Work".to_string(),
            credentials: CredentialData::GitHub {
                access_token: "ghp_github".to_string(),
                default_repo: Some("work/repo".to_string()),
                webhook_secret: None,
            },
        };
        repo.create_credential(github_input).await.unwrap();

        // Create Slack credential
        let slack_input = CreateCredentialInput {
            integration_type: IntegrationType::Slack,
            name: "Company Slack".to_string(),
            credentials: CredentialData::Slack {
                bot_token: "xoxb-slack".to_string(),
                user_token: None,
                workspace: "company".to_string(),
            },
        };
        repo.create_credential(slack_input).await.unwrap();

        // Create inactive GitHub credential
        let mut inactive_github = CreateCredentialInput {
            integration_type: IntegrationType::GitHub,
            name: "Old GitHub".to_string(),
            credentials: CredentialData::GitHub {
                access_token: "ghp_old".to_string(),
                default_repo: None,
                webhook_secret: None,
            },
        };
        let inactive_cred = repo.create_credential(inactive_github).await.unwrap();
        
        // Deactivate it
        use luce_shared::UpdateCredentialInput;
        repo.update_credential(
            inactive_cred.id,
            UpdateCredentialInput {
                name: None,
                credentials: None,
                is_active: Some(false),
            },
        ).await.unwrap();

        ListCredentialsUseCase::new(repo)
    }

    #[tokio::test]
    async fn test_list_all_active_credentials() {
        let usecase = create_test_usecase_with_data().await;
        let input = ListCredentialsInput::new();

        let credentials = usecase.execute(input).await.unwrap();

        assert_eq!(credentials.len(), 2); // Only active ones
        let github_count = credentials.iter().filter(|c| c.integration_type == IntegrationType::GitHub).count();
        let slack_count = credentials.iter().filter(|c| c.integration_type == IntegrationType::Slack).count();
        assert_eq!(github_count, 1);
        assert_eq!(slack_count, 1);
    }

    #[tokio::test]
    async fn test_list_github_credentials_only() {
        let usecase = create_test_usecase_with_data().await;
        let input = ListCredentialsInput::new().with_integration_type(IntegrationType::GitHub);

        let credentials = usecase.execute(input).await.unwrap();

        assert_eq!(credentials.len(), 1); // Only active GitHub
        assert_eq!(credentials[0].integration_type, IntegrationType::GitHub);
        assert_eq!(credentials[0].name, "GitHub Work");
    }

    #[tokio::test]
    async fn test_list_including_inactive() {
        let usecase = create_test_usecase_with_data().await;
        let input = ListCredentialsInput::new()
            .with_integration_type(IntegrationType::GitHub)
            .include_inactive();

        let credentials = usecase.execute(input).await.unwrap();

        assert_eq!(credentials.len(), 2); // Active + inactive GitHub
        let active_count = credentials.iter().filter(|c| c.is_active).count();
        let inactive_count = credentials.iter().filter(|c| !c.is_active).count();
        assert_eq!(active_count, 1);
        assert_eq!(inactive_count, 1);
    }

    #[tokio::test]
    async fn test_list_empty_result() {
        let repo = SqliteCredentialRepository::new("sqlite::memory:").await.unwrap();
        let usecase = ListCredentialsUseCase::new(repo);
        let input = ListCredentialsInput::new();

        let credentials = usecase.execute(input).await.unwrap();

        assert!(credentials.is_empty());
    }
}