use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LuceConfig {
    pub database_url: String,
    pub integrations: IntegrationsConfig,
    pub server: ServerConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IntegrationsConfig {
    pub github: Option<GitHubConfig>,
    pub slack: Option<SlackConfig>,
    pub linear: Option<LinearConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub access_token: String,
    pub webhook_secret: String,
    pub default_repo: String,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlackConfig {
    pub bot_token: String,
    pub app_token: Option<String>,
    pub signing_secret: String,
    pub default_channel: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearConfig {
    pub api_key: String,
    pub team_id: String,
    pub webhook_secret: Option<String>,
}

impl Default for LuceConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite:luce.db".to_string(),
            integrations: IntegrationsConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
            cors_origins: vec!["*".to_string()],
        }
    }
}

impl LuceConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let mut config = LuceConfig::default();

        // Database
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            config.database_url = database_url;
        }

        // Server
        if let Ok(host) = std::env::var("LUCE_HOST") {
            config.server.host = host;
        }
        if let Ok(port) = std::env::var("LUCE_PORT") {
            config.server.port = port.parse()?;
        }
        if let Ok(cors_origins) = std::env::var("LUCE_CORS_ORIGINS") {
            config.server.cors_origins = cors_origins.split(',').map(|s| s.to_string()).collect();
        }

        // GitHub integration
        if let (Ok(token), Ok(secret), Ok(repo)) = (
            std::env::var("GITHUB_ACCESS_TOKEN"),
            std::env::var("GITHUB_WEBHOOK_SECRET"),
            std::env::var("GITHUB_DEFAULT_REPO"),
        ) {
            config.integrations.github = Some(GitHubConfig {
                access_token: token,
                webhook_secret: secret,
                default_repo: repo,
                webhook_url: std::env::var("GITHUB_WEBHOOK_URL").ok(),
            });
        }

        // Slack integration
        if let (Ok(bot_token), Ok(signing_secret)) = (
            std::env::var("SLACK_BOT_TOKEN"),
            std::env::var("SLACK_SIGNING_SECRET"),
        ) {
            config.integrations.slack = Some(SlackConfig {
                bot_token,
                app_token: std::env::var("SLACK_APP_TOKEN").ok(),
                signing_secret,
                default_channel: std::env::var("SLACK_DEFAULT_CHANNEL").ok(),
            });
        }

        // Linear integration
        if let (Ok(api_key), Ok(team_id)) = (
            std::env::var("LINEAR_API_KEY"),
            std::env::var("LINEAR_TEAM_ID"),
        ) {
            config.integrations.linear = Some(LinearConfig {
                api_key,
                team_id,
                webhook_secret: std::env::var("LINEAR_WEBHOOK_SECRET").ok(),
            });
        }

        Ok(config)
    }

    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config = if path.ends_with(".toml") {
            toml::from_str(&contents)?
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::from_str(&contents)?
        } else {
            serde_json::from_str(&contents)?
        };
        Ok(config)
    }

    pub fn to_file(&self, path: &str) -> anyhow::Result<()> {
        let contents = if path.ends_with(".toml") {
            toml::to_string_pretty(self)?
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            serde_yaml::to_string(self)?
        } else {
            serde_json::to_string_pretty(self)?
        };
        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn has_github_integration(&self) -> bool {
        self.integrations.github.is_some()
    }

    pub fn has_slack_integration(&self) -> bool {
        self.integrations.slack.is_some()
    }

    pub fn has_linear_integration(&self) -> bool {
        self.integrations.linear.is_some()
    }

    pub fn get_enabled_integrations(&self) -> Vec<&'static str> {
        let mut enabled = Vec::new();
        if self.has_github_integration() {
            enabled.push("github");
        }
        if self.has_slack_integration() {
            enabled.push("slack");
        }
        if self.has_linear_integration() {
            enabled.push("linear");
        }
        enabled
    }
}

impl GitHubConfig {
    pub fn is_valid(&self) -> bool {
        !self.access_token.is_empty()
            && !self.webhook_secret.is_empty()
            && !self.default_repo.is_empty()
            && self.default_repo.contains('/')
    }
}

impl SlackConfig {
    pub fn is_valid(&self) -> bool {
        !self.bot_token.is_empty() && !self.signing_secret.is_empty()
    }
}

impl LinearConfig {
    pub fn is_valid(&self) -> bool {
        !self.api_key.is_empty() && !self.team_id.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = LuceConfig::default();
        assert_eq!(config.database_url, "sqlite:luce.db");
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
        assert!(config.integrations.github.is_none());
        assert!(config.integrations.slack.is_none());
        assert!(config.integrations.linear.is_none());
    }

    #[test]
    fn test_config_serialization() {
        let config = LuceConfig {
            database_url: "sqlite:test.db".to_string(),
            integrations: IntegrationsConfig {
                github: Some(GitHubConfig {
                    access_token: "token".to_string(),
                    webhook_secret: "secret".to_string(),
                    default_repo: "owner/repo".to_string(),
                    webhook_url: Some("https://example.com/webhook".to_string()),
                }),
                slack: None,
                linear: None,
            },
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                cors_origins: vec!["https://example.com".to_string()],
            },
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: LuceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_github_config_validation() {
        let valid_config = GitHubConfig {
            access_token: "token".to_string(),
            webhook_secret: "secret".to_string(),
            default_repo: "owner/repo".to_string(),
            webhook_url: None,
        };
        assert!(valid_config.is_valid());

        let invalid_config = GitHubConfig {
            access_token: "".to_string(),
            webhook_secret: "secret".to_string(),
            default_repo: "owner/repo".to_string(),
            webhook_url: None,
        };
        assert!(!invalid_config.is_valid());

        let invalid_repo_config = GitHubConfig {
            access_token: "token".to_string(),
            webhook_secret: "secret".to_string(),
            default_repo: "invalid-repo".to_string(), // Missing '/'
            webhook_url: None,
        };
        assert!(!invalid_repo_config.is_valid());
    }

    #[test]
    fn test_slack_config_validation() {
        let valid_config = SlackConfig {
            bot_token: "xoxb-token".to_string(),
            app_token: None,
            signing_secret: "secret".to_string(),
            default_channel: None,
        };
        assert!(valid_config.is_valid());

        let invalid_config = SlackConfig {
            bot_token: "".to_string(),
            app_token: None,
            signing_secret: "secret".to_string(),
            default_channel: None,
        };
        assert!(!invalid_config.is_valid());
    }

    #[test]
    fn test_linear_config_validation() {
        let valid_config = LinearConfig {
            api_key: "lin_api_key".to_string(),
            team_id: "team_id".to_string(),
            webhook_secret: None,
        };
        assert!(valid_config.is_valid());

        let invalid_config = LinearConfig {
            api_key: "lin_api_key".to_string(),
            team_id: "".to_string(),
            webhook_secret: None,
        };
        assert!(!invalid_config.is_valid());
    }

    #[test]
    fn test_enabled_integrations() {
        let mut config = LuceConfig::default();
        assert_eq!(config.get_enabled_integrations(), Vec::<&str>::new());

        config.integrations.github = Some(GitHubConfig {
            access_token: "token".to_string(),
            webhook_secret: "secret".to_string(),
            default_repo: "owner/repo".to_string(),
            webhook_url: None,
        });
        assert_eq!(config.get_enabled_integrations(), vec!["github"]);

        config.integrations.slack = Some(SlackConfig {
            bot_token: "token".to_string(),
            app_token: None,
            signing_secret: "secret".to_string(),
            default_channel: None,
        });
        assert_eq!(config.get_enabled_integrations(), vec!["github", "slack"]);
    }

    #[test]
    fn test_from_env() {
        // Set some environment variables
        env::set_var("DATABASE_URL", "postgresql://localhost/test");
        env::set_var("LUCE_HOST", "0.0.0.0");
        env::set_var("LUCE_PORT", "8080");
        env::set_var("GITHUB_ACCESS_TOKEN", "test_token");
        env::set_var("GITHUB_WEBHOOK_SECRET", "test_secret");
        env::set_var("GITHUB_DEFAULT_REPO", "test/repo");

        let config = LuceConfig::from_env().unwrap();

        assert_eq!(config.database_url, "postgresql://localhost/test");
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 8080);
        assert!(config.integrations.github.is_some());

        let github_config = config.integrations.github.unwrap();
        assert_eq!(github_config.access_token, "test_token");
        assert_eq!(github_config.webhook_secret, "test_secret");
        assert_eq!(github_config.default_repo, "test/repo");

        // Clean up
        env::remove_var("DATABASE_URL");
        env::remove_var("LUCE_HOST");
        env::remove_var("LUCE_PORT");
        env::remove_var("GITHUB_ACCESS_TOKEN");
        env::remove_var("GITHUB_WEBHOOK_SECRET");
        env::remove_var("GITHUB_DEFAULT_REPO");
    }

    #[test]
    fn test_config_file_operations() {
        let config = LuceConfig {
            database_url: "sqlite:test.db".to_string(),
            integrations: IntegrationsConfig {
                github: Some(GitHubConfig {
                    access_token: "token".to_string(),
                    webhook_secret: "secret".to_string(),
                    default_repo: "owner/repo".to_string(),
                    webhook_url: None,
                }),
                slack: None,
                linear: None,
            },
            server: ServerConfig::default(),
        };

        // Test JSON serialization
        config.to_file("/tmp/test_config.json").unwrap();
        let loaded_config = LuceConfig::from_file("/tmp/test_config.json").unwrap();
        assert_eq!(config, loaded_config);

        // Clean up
        std::fs::remove_file("/tmp/test_config.json").ok();
    }
}
