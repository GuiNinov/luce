use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use luce_core::{CredentialService};
use luce_shared::{CredentialData, IntegrationType};
use uuid::Uuid;

use crate::LuceService;

#[derive(Args)]
pub struct CredentialArgs {
    #[command(subcommand)]
    pub action: CredentialAction,
}

#[derive(Subcommand)]
pub enum CredentialAction {
    /// List credentials
    List {
        /// Filter by integration type (github, slack, linear)
        #[arg(long)]
        integration_type: Option<String>,
        
        /// Include inactive credentials
        #[arg(long)]
        include_inactive: bool,
    },
    /// Create a new credential
    Create {
        /// Integration type (github, slack, linear)
        #[arg(long)]
        integration_type: String,
        
        /// Name for the credential
        #[arg(long)]
        name: String,
        
        /// GitHub access token
        #[arg(long)]
        github_token: Option<String>,
        
        /// GitHub default repository (owner/repo)
        #[arg(long)]
        github_repo: Option<String>,
        
        /// GitHub webhook secret
        #[arg(long)]
        github_webhook_secret: Option<String>,
        
        /// Slack bot token
        #[arg(long)]
        slack_bot_token: Option<String>,
        
        /// Slack user token
        #[arg(long)]
        slack_user_token: Option<String>,
        
        /// Slack workspace
        #[arg(long)]
        slack_workspace: Option<String>,
        
        /// Linear API key
        #[arg(long)]
        linear_api_key: Option<String>,
        
        /// Linear workspace
        #[arg(long)]
        linear_workspace: Option<String>,
    },
    /// Get credential details
    Get {
        /// Credential ID
        id: String,
    },
    /// Update a credential
    Update {
        /// Credential ID
        id: String,
        
        /// New name for the credential
        #[arg(long)]
        name: Option<String>,
        
        /// Activate/deactivate the credential
        #[arg(long)]
        active: Option<bool>,
    },
    /// Delete a credential
    Delete {
        /// Credential ID
        id: String,
        
        /// Force deletion without confirmation
        #[arg(long)]
        force: bool,
    },
    /// Test a credential by retrieving its data
    Test {
        /// Credential ID
        id: String,
    },
}

pub async fn handle_credential_commands(args: CredentialArgs, service: &LuceService) -> Result<()> {
    let db_url = std::env::var("LUCE_DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./luce.db".to_string());
    
    let credential_service = CredentialService::new(&db_url).await
        .context("Failed to initialize credential service")?;

    match args.action {
        CredentialAction::List { integration_type, include_inactive } => {
            handle_list_credentials(&credential_service, integration_type, include_inactive).await
        }
        CredentialAction::Create { 
            integration_type, name, github_token, github_repo, github_webhook_secret,
            slack_bot_token, slack_user_token, slack_workspace,
            linear_api_key, linear_workspace 
        } => {
            handle_create_credential(
                &credential_service, integration_type, name,
                github_token, github_repo, github_webhook_secret,
                slack_bot_token, slack_user_token, slack_workspace,
                linear_api_key, linear_workspace
            ).await
        }
        CredentialAction::Get { id } => {
            handle_get_credential(&credential_service, id).await
        }
        CredentialAction::Update { id, name, active } => {
            handle_update_credential(&credential_service, id, name, active).await
        }
        CredentialAction::Delete { id, force } => {
            handle_delete_credential(&credential_service, id, force).await
        }
        CredentialAction::Test { id } => {
            handle_test_credential(&credential_service, id).await
        }
    }
}

async fn handle_list_credentials(
    service: &CredentialService,
    integration_type: Option<String>,
    include_inactive: bool,
) -> Result<()> {
    let integration_type = if let Some(type_str) = integration_type {
        Some(parse_integration_type(&type_str)?)
    } else {
        None
    };
    
    let credentials = service.list_credentials(integration_type, !include_inactive).await
        .context("Failed to list credentials")?;

    if credentials.is_empty() {
        println!("No credentials found.");
        return Ok(());
    }

    println!("┌────────────────────────────────────────┬─────────────────┬──────────────────────┬────────┐");
    println!("│ ID                                     │ Type            │ Name                 │ Active │");
    println!("├────────────────────────────────────────┼─────────────────┼──────────────────────┼────────┤");
    
    let count = credentials.len();
    
    for cred in &credentials {
        let id_short = &cred.id.to_string()[..8];
        let status = if cred.is_active { "✓" } else { "✗" };
        println!("│ {}...                             │ {:15} │ {:20} │   {}    │", 
                 id_short, 
                 format!("{:?}", cred.integration_type), 
                 &cred.name[..std::cmp::min(cred.name.len(), 20)],
                 status);
    }
    
    println!("└────────────────────────────────────────┴─────────────────┴──────────────────────┴────────┘");
    println!("Found {} credential(s).", count);
    
    Ok(())
}

async fn handle_create_credential(
    service: &CredentialService,
    integration_type: String,
    name: String,
    github_token: Option<String>,
    github_repo: Option<String>,
    github_webhook_secret: Option<String>,
    slack_bot_token: Option<String>,
    slack_user_token: Option<String>,
    slack_workspace: Option<String>,
    linear_api_key: Option<String>,
    linear_workspace: Option<String>,
) -> Result<()> {
    let integration_type = parse_integration_type(&integration_type)?;
    
    let credentials = match integration_type {
        IntegrationType::GitHub => {
            let access_token = github_token.ok_or_else(|| {
                anyhow::anyhow!("GitHub credentials require --github-token")
            })?;
            
            CredentialData::GitHub {
                access_token,
                default_repo: github_repo,
                webhook_secret: github_webhook_secret,
            }
        }
        IntegrationType::Slack => {
            let bot_token = slack_bot_token.ok_or_else(|| {
                anyhow::anyhow!("Slack credentials require --slack-bot-token")
            })?;
            let workspace = slack_workspace.ok_or_else(|| {
                anyhow::anyhow!("Slack credentials require --slack-workspace")
            })?;
            
            CredentialData::Slack {
                bot_token,
                user_token: slack_user_token,
                workspace,
            }
        }
        IntegrationType::Linear => {
            let api_key = linear_api_key.ok_or_else(|| {
                anyhow::anyhow!("Linear credentials require --linear-api-key")
            })?;
            let workspace = linear_workspace.ok_or_else(|| {
                anyhow::anyhow!("Linear credentials require --linear-workspace")
            })?;
            
            CredentialData::Linear {
                api_key,
                workspace,
            }
        }
    };

    let credential = service.create_credential(integration_type, name, credentials).await
        .context("Failed to create credential")?;

    println!("✓ Created credential: {}", credential.id);
    println!("  Name: {}", credential.name);
    println!("  Type: {:?}", credential.integration_type);
    println!("  Status: {}", if credential.is_active { "Active" } else { "Inactive" });
    
    Ok(())
}

async fn handle_get_credential(
    service: &CredentialService,
    id: String,
) -> Result<()> {
    let credential_id = Uuid::parse_str(&id)
        .context("Invalid credential ID format")?;
    
    let credential = service.get_credential(credential_id).await
        .context("Failed to get credential")?;

    println!("Credential Details:");
    println!("  ID: {}", credential.id);
    println!("  Name: {}", credential.name);
    println!("  Type: {:?}", credential.integration_type);
    println!("  Status: {}", if credential.is_active { "Active" } else { "Inactive" });
    println!("  Created: {}", credential.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("  Updated: {}", credential.updated_at.format("%Y-%m-%d %H:%M:%S UTC"));
    
    if let Some(last_used) = credential.last_used_at {
        println!("  Last Used: {}", last_used.format("%Y-%m-%d %H:%M:%S UTC"));
    } else {
        println!("  Last Used: Never");
    }
    
    Ok(())
}

async fn handle_update_credential(
    service: &CredentialService,
    id: String,
    name: Option<String>,
    active: Option<bool>,
) -> Result<()> {
    let credential_id = Uuid::parse_str(&id)
        .context("Invalid credential ID format")?;
    
    let updated = service.update_credential(credential_id, name, None, active).await
        .context("Failed to update credential")?;

    println!("✓ Updated credential: {}", updated.id);
    println!("  Name: {}", updated.name);
    println!("  Type: {:?}", updated.integration_type);
    println!("  Status: {}", if updated.is_active { "Active" } else { "Inactive" });
    
    Ok(())
}

async fn handle_delete_credential(
    service: &CredentialService,
    id: String,
    force: bool,
) -> Result<()> {
    let credential_id = Uuid::parse_str(&id)
        .context("Invalid credential ID format")?;
    
    if !force {
        // Get credential details for confirmation
        let credential = service.get_credential(credential_id).await
            .context("Failed to get credential details")?;
        
        println!("Are you sure you want to delete this credential?");
        println!("  ID: {}", credential.id);
        println!("  Name: {}", credential.name);
        println!("  Type: {:?}", credential.integration_type);
        
        print!("Type 'yes' to confirm: ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).context("Failed to read input")?;
        
        if input.trim().to_lowercase() != "yes" {
            println!("Deletion cancelled.");
            return Ok(());
        }
    }
    
    service.delete_credential(credential_id).await
        .context("Failed to delete credential")?;

    println!("✓ Deleted credential: {}", credential_id);
    
    Ok(())
}

async fn handle_test_credential(
    service: &CredentialService,
    id: String,
) -> Result<()> {
    let credential_id = Uuid::parse_str(&id)
        .context("Invalid credential ID format")?;
    
    let credential_data = service.get_credential_data(credential_id).await
        .context("Failed to retrieve credential data")?;

    println!("✓ Successfully retrieved credential data:");
    
    match credential_data {
        CredentialData::GitHub { access_token, default_repo, webhook_secret } => {
            println!("  Type: GitHub");
            println!("  Access Token: {}...{}", &access_token[..8], &access_token[access_token.len()-4..]);
            if let Some(repo) = default_repo {
                println!("  Default Repo: {}", repo);
            }
            if webhook_secret.is_some() {
                println!("  Webhook Secret: [configured]");
            }
        }
        CredentialData::Slack { bot_token, user_token, workspace } => {
            println!("  Type: Slack");
            println!("  Bot Token: {}...{}", &bot_token[..8], &bot_token[bot_token.len()-4..]);
            if user_token.is_some() {
                println!("  User Token: [configured]");
            }
            println!("  Workspace: {}", workspace);
        }
        CredentialData::Linear { api_key, workspace } => {
            println!("  Type: Linear");
            println!("  API Key: {}...{}", &api_key[..8], &api_key[api_key.len()-4..]);
            println!("  Workspace: {}", workspace);
        }
    }
    
    println!("  Note: Credential has been marked as used.");
    
    Ok(())
}

fn parse_integration_type(type_str: &str) -> Result<IntegrationType> {
    match type_str.to_lowercase().as_str() {
        "github" => Ok(IntegrationType::GitHub),
        "slack" => Ok(IntegrationType::Slack),
        "linear" => Ok(IntegrationType::Linear),
        _ => Err(anyhow::anyhow!("Invalid integration type: {}. Valid types are: github, slack, linear", type_str)),
    }
}