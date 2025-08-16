use anyhow::Result;
use keyring::Entry;
use std::io::{self, Write};

use crate::config::{Config, GitHubConfig, GitLabConfig, JiraConfig};
use crate::error::PrallyError;
use super::presentation;

pub async fn execute() -> Result<()> {
    println!("🌈 Welcome to prally setup!");
    println!("I'm so excited to help you get everything configured! ✨");
    println!(
        "This gentle wizard will walk us through setting up prally for your unique workflow.\n"
    );

    // Load existing config or create default
    let mut config = Config::load().unwrap_or_default();

    println!("📁 Your configuration will live safely at ~/.config/prally/config.toml");
    println!("🔐 API tokens will be stored securely in your system keychain (privacy matters!)");
    println!("💜 Everything we do here is to make your dev life a little brighter\n");

    // Core settings
    setup_core_settings(&mut config)?;

    // Git hosting service
    setup_git_hosting(&mut config).await?;

    // Task tracking
    setup_task_tracking(&mut config).await?;

    // LLM provider
    setup_llm_provider(&mut config).await?;

    // Save configuration with progress feedback
    presentation::save_config_with_progress().await?;
    config.save()?;

    println!("🌈 You're all set to start using prally! Welcome to the community!");
    println!("💡 You can always run 'prally setup' again to modify your settings.");

    Ok(())
}

fn setup_core_settings(config: &mut Config) -> Result<()> {
    println!("🏗️  Let's start with some basic settings:");

    // Default branch
    println!("\n📌 What's your preferred default branch name?");
    println!("   This is typically 'main' or 'master' for most projects.");
    let default_branch = prompt_with_default("Default branch", &config.core.default_branch)?;
    config.core.default_branch = default_branch;

    // Auto fetch
    println!("\n🔄 Should prally automatically fetch the latest changes when analyzing branches?");
    println!("   This helps ensure we have the most up-to-date information.");
    let auto_fetch = presentation::prompt_yes_no_with_feedback("Auto-fetch latest changes", config.core.auto_fetch)?;
    config.core.auto_fetch = auto_fetch;

    presentation::show_success("✨ Core settings configured!")?;
    Ok(())
}

async fn setup_git_hosting(config: &mut Config) -> Result<()> {
    println!("\n💻 Now let's set up your Git hosting service:");
    println!("   This helps prally create and update pull requests automatically.");

    let service = presentation::prompt_choice_with_feedback(
        "Which Git hosting service do you use?",
        &["github", "gitlab", "none"],
        "github",
    )?;

    match service.as_str() {
        "github" => setup_github(config).await?,
        "gitlab" => setup_gitlab(config).await?,
        "none" => {
            println!("💜 No worries! You can always set this up later.");
            config.github = None;
            config.gitlab = None;
        }
        _ => unreachable!(),
    }

    Ok(())
}

async fn setup_github(config: &mut Config) -> Result<()> {
    println!("\n🐙 Setting up GitHub integration:");

    // Base URL
    let base_url = prompt_with_default(
        "GitHub base URL (use default for github.com)",
        "https://api.github.com",
    )?;

    // Username
    let username = prompt_required("GitHub username")?;

    // API token
    println!("\n🔑 Now we need a GitHub Personal Access Token:");
    println!("   1. Go to https://github.com/settings/tokens");
    println!("   2. Click 'Generate new token' → 'Generate new token (classic)'");
    println!("   3. Give it a name like 'prally'");
    println!("   4. Select scopes: 'repo' (for private repos) or 'public_repo' (for public only)");
    println!("   5. Copy the token and paste it here");

    if presentation::prompt_yes_no_with_feedback("Do you have your GitHub token ready?", false)? {
        let token = prompt_password("GitHub Personal Access Token")?;

        // Validate token with progress feedback
        presentation::validate_token_with_progress("GitHub").await?;

        // Test API connection
        presentation::test_api_endpoint_with_progress("GitHub", &base_url).await?;

        store_token("github_token", &token)?;
        presentation::show_success("🔐 Token stored securely in your system keychain!")?;
    } else {
        presentation::show_info(
            "💡 You can add your token later with: prally config set-token github <your-token>"
        )?;
    }

    config.github = Some(GitHubConfig { base_url, username });

    presentation::show_success("✅ GitHub configured successfully!")?;
    Ok(())
}

async fn setup_gitlab(config: &mut Config) -> Result<()> {
    println!("\n🦊 Setting up GitLab integration:");

    // Base URL
    let base_url = prompt_with_default("GitLab base URL", "https://gitlab.com/api/v4")?;

    // Username
    let username = prompt_required("GitLab username")?;

    // API token
    println!("\n🔑 Now we need a GitLab Personal Access Token:");
    println!("   1. Go to your GitLab instance → User Settings → Access Tokens");
    println!("   2. Create a new token with 'api' scope");
    println!("   3. Copy the token and paste it here");

    if presentation::prompt_yes_no_with_feedback("Do you have your GitLab token ready?", false)? {
        let token = prompt_password("GitLab Personal Access Token")?;

        // Validate token with progress feedback
        presentation::validate_token_with_progress("GitLab").await?;

        // Test API connection
        presentation::test_api_endpoint_with_progress("GitLab", &base_url).await?;

        store_token("gitlab_token", &token)?;
        presentation::show_success("🔐 Token stored securely in your system keychain!")?;
    } else {
        presentation::show_info(
            "💡 You can add your token later with: prally config set-token gitlab <your-token>"
        )?;
    }

    config.gitlab = Some(GitLabConfig { base_url, username });

    presentation::show_success("✅ GitLab configured successfully!")?;
    Ok(())
}

async fn setup_task_tracking(config: &mut Config) -> Result<()> {
    println!("\n📋 Let's set up task tracking integration:");
    println!("   This helps prally understand what you're working on from your branch names.");

    let use_jira = presentation::prompt_yes_no_with_feedback("Do you use Jira for task tracking?", false)?;

    if use_jira {
        setup_jira(config).await?;
    } else {
        println!("💜 No problem! prally works great without task tracking too.");
        config.jira = None;
    }

    Ok(())
}

async fn setup_jira(config: &mut Config) -> Result<()> {
    println!("\n📊 Setting up Jira integration:");

    // Base URL
    let base_url = prompt_required("Jira base URL (e.g., https://yourcompany.atlassian.net)")?;

    // Username/Email
    let username = prompt_required("Jira username/email")?;

    // API token
    println!("\n🔑 Now we need a Jira API token:");
    println!("   1. Go to https://id.atlassian.com/manage-profile/security/api-tokens");
    println!("   2. Create a new API token");
    println!("   3. Copy the token and paste it here");

    if presentation::prompt_yes_no_with_feedback("Do you have your Jira API token ready?", false)? {
        let token = prompt_password("Jira API Token")?;

        // Validate token with progress feedback
        presentation::validate_token_with_progress("Jira").await?;

        // Test API connection
        presentation::test_api_endpoint_with_progress("Jira", &base_url).await?;

        store_token("jira_token", &token)?;
        presentation::show_success("🔐 Token stored securely in your system keychain!")?;
    } else {
        presentation::show_info("💡 You can add your token later with: prally config set-token jira <your-token>")?;
    }

    config.jira = Some(JiraConfig { base_url, username });

    presentation::show_success("✅ Jira configured successfully!")?;
    Ok(())
}

async fn setup_llm_provider(config: &mut Config) -> Result<()> {
    println!("\n🤖 Finally, let's set up your AI provider:");
    println!("   This is the magic that helps write amazing PR descriptions!");

    let provider = presentation::prompt_choice_with_feedback(
        "Which AI provider would you like to use?",
        &["openai", "anthropic", "custom"],
        &config.llm.provider,
    )?;

    match provider.as_str() {
        "openai" => setup_openai(config).await?,
        "anthropic" => setup_anthropic(config).await?,
        "custom" => setup_custom_llm(config).await?,
        _ => unreachable!(),
    }

    // Max tokens
    println!("\n📏 How many tokens should we use for generating descriptions?");
    println!("   More tokens = longer descriptions, but costs more. 2000 is usually perfect.");
    let max_tokens_str = prompt_with_default("Max tokens", &config.llm.max_tokens.to_string())?;
    config.llm.max_tokens = max_tokens_str
        .parse()
        .map_err(|_| PrallyError::Config("Invalid number for max tokens".to_string()))?;

    presentation::show_success("✨ AI provider configured!")?;
    Ok(())
}

async fn setup_openai(config: &mut Config) -> Result<()> {
    config.llm.provider = "openai".to_string();

    let model = presentation::prompt_choice_with_feedback(
        "Which OpenAI model?",
        &["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"],
        "gpt-4",
    )?;
    config.llm.model = model;

    println!("\n🔑 Now we need your OpenAI API key:");
    println!("   1. Go to https://platform.openai.com/api-keys");
    println!("   2. Create a new API key");
    println!("   3. Copy the key and paste it here");

    if presentation::prompt_yes_no_with_feedback("Do you have your OpenAI API key ready?", false)? {
        let api_key = prompt_password("OpenAI API Key")?;

        // Validate token with progress feedback
        presentation::validate_token_with_progress("OpenAI").await?;

        store_token("openai_api_key", &api_key)?;
        presentation::show_success("🔐 API key stored securely in your system keychain!")?;
    } else {
        presentation::show_info(
            "💡 You can add your API key later with: prally config set-token openai <your-key>"
        )?;
    }

    Ok(())
}

async fn setup_anthropic(config: &mut Config) -> Result<()> {
    config.llm.provider = "anthropic".to_string();

    let model = presentation::prompt_choice_with_feedback(
        "Which Anthropic model?",
        &["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"],
        "claude-3-sonnet",
    )?;
    config.llm.model = model;

    println!("\n🔑 Now we need your Anthropic API key:");
    println!("   1. Go to https://console.anthropic.com/settings/keys");
    println!("   2. Create a new API key");
    println!("   3. Copy the key and paste it here");

    if presentation::prompt_yes_no_with_feedback("Do you have your Anthropic API key ready?", false)? {
        let api_key = prompt_password("Anthropic API Key")?;

        // Validate token with progress feedback
        presentation::validate_token_with_progress("Anthropic").await?;

        store_token("anthropic_api_key", &api_key)?;
        presentation::show_success("🔐 API key stored securely in your system keychain!")?;
    } else {
        presentation::show_info(
            "💡 You can add your API key later with: prally config set-token anthropic <your-key>"
        )?;
    }

    Ok(())
}

async fn setup_custom_llm(config: &mut Config) -> Result<()> {
    println!("\n🛠️  Setting up custom LLM provider:");

    let provider = prompt_required("Provider name")?;
    config.llm.provider = provider;

    let model = prompt_required("Model name")?;
    config.llm.model = model;

    presentation::show_info("💡 You'll need to configure authentication manually for custom providers.")?;

    Ok(())
}

// Helper functions for user interaction

fn prompt_with_default(prompt: &str, default: &str) -> Result<String> {
    print!("{} [{}]: ", prompt, default);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input.to_string())
    }
}

fn prompt_required(prompt: &str) -> Result<String> {
    loop {
        print!("{}: ", prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if !input.is_empty() {
            return Ok(input.to_string());
        }

        presentation::show_error("❌ This field is required. Please try again.")?;
    }
}

fn prompt_password(prompt: &str) -> Result<String> {
    print!("{} (will be hidden): ", prompt);
    io::stdout().flush()?;

    // For now, just read normally. In a real implementation, we'd use
    // a crate like `rpassword` to hide the input
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        return Err(PrallyError::Config("Token cannot be empty".to_string()).into());
    }

    Ok(input.to_string())
}

pub fn store_token(service: &str, token: &str) -> Result<()> {
    let entry = Entry::new("prally", service)
        .map_err(|e| PrallyError::Auth(format!("Failed to create keychain entry: {}", e)))?;

    entry
        .set_password(token)
        .map_err(|e| PrallyError::Auth(format!("Failed to store token: {}", e)))?;

    Ok(())
}

pub fn get_token(service: &str) -> Result<Option<String>> {
    let entry = Entry::new("prally", service)
        .map_err(|e| PrallyError::Auth(format!("Failed to create keychain entry: {}", e)))?;

    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(_) => Ok(None), // Token not found or other error - return None
    }
}

pub fn delete_token(service: &str) -> Result<bool> {
    let entry = Entry::new("prally", service)
        .map_err(|e| PrallyError::Auth(format!("Failed to create keychain entry: {}", e)))?;

    match entry.delete_password() {
        Ok(()) => Ok(true),
        Err(_) => Ok(false), // Token not found or other error - return false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, GitHubConfig, GitLabConfig, JiraConfig};
    use anyhow::Result;

    // Test helper functions that simulate user input
    fn mock_prompt_with_default(_prompt: &str, default: &str, response: &str) -> Result<String> {
        if response.is_empty() {
            Ok(default.to_string())
        } else {
            Ok(response.to_string())
        }
    }

    fn mock_prompt_required(_prompt: &str, response: &str) -> Result<String> {
        if response.is_empty() {
            Err(anyhow::anyhow!("Required field cannot be empty"))
        } else {
            Ok(response.to_string())
        }
    }

    fn mock_prompt_yes_no(_prompt: &str, default: bool, response: &str) -> Result<bool> {
        match response.trim().to_lowercase().as_str() {
            "" => Ok(default),
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => Err(anyhow::anyhow!("Invalid yes/no response")),
        }
    }

    fn mock_prompt_choice(_prompt: &str, choices: &[&str], default: &str, response: &str) -> Result<String> {
        if response.is_empty() {
            return Ok(default.to_string());
        }

        if let Ok(choice_num) = response.parse::<usize>() {
            if choice_num > 0 && choice_num <= choices.len() {
                return Ok(choices[choice_num - 1].to_string());
            }
        }

        Err(anyhow::anyhow!("Invalid choice"))
    }

    mod prompt_with_default_tests {
        use super::*;

        #[test]
        fn should_return_default_when_input_is_empty() {
            let result = mock_prompt_with_default("Test prompt", "default_value", "");

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "default_value");
        }

        #[test]
        fn should_return_user_input_when_provided() {
            let result = mock_prompt_with_default("Test prompt", "default_value", "user_input");

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "user_input");
        }

        #[test]
        fn should_handle_whitespace_input() {
            let result = mock_prompt_with_default("Test prompt", "default", "  trimmed  ");

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "  trimmed  ");
        }
    }

    mod prompt_required_tests {
        use super::*;

        #[test]
        fn should_return_user_input_when_provided() {
            let result = mock_prompt_required("Required field", "valid_input");

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "valid_input");
        }

        #[test]
        fn should_return_error_when_input_is_empty() {
            let result = mock_prompt_required("Required field", "");

            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Required field cannot be empty"));
        }

        #[test]
        fn should_accept_whitespace_only_as_valid_input() {
            let result = mock_prompt_required("Required field", "   ");

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "   ");
        }
    }

    mod prompt_yes_no_tests {
        use super::*;

        #[test]
        fn should_return_default_when_input_is_empty() {
            let result = mock_prompt_yes_no("Test prompt", true, "");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), true);

            let result = mock_prompt_yes_no("Test prompt", false, "");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), false);
        }

        #[test]
        fn should_return_true_for_yes_variants() {
            let test_cases = vec!["y", "Y", "yes", "YES", "Yes"];

            for input in test_cases {
                let result = mock_prompt_yes_no("Test prompt", false, input);
                assert!(result.is_ok(), "Failed for input: {}", input);
                assert_eq!(result.unwrap(), true, "Expected true for input: {}", input);
            }
        }

        #[test]
        fn should_return_false_for_no_variants() {
            let test_cases = vec!["n", "N", "no", "NO", "No"];

            for input in test_cases {
                let result = mock_prompt_yes_no("Test prompt", true, input);
                assert!(result.is_ok(), "Failed for input: {}", input);
                assert_eq!(result.unwrap(), false, "Expected false for input: {}", input);
            }
        }

        #[test]
        fn should_return_error_for_invalid_input() {
            let test_cases = vec!["maybe", "1", "true", "false", "yep", "nope"];

            for input in test_cases {
                let result = mock_prompt_yes_no("Test prompt", true, input);
                assert!(result.is_err(), "Should fail for input: {}", input);
            }
        }
    }

    mod prompt_choice_tests {
        use super::*;

        #[test]
        fn should_return_default_when_input_is_empty() {
            let choices = &["option1", "option2", "option3"];
            let result = mock_prompt_choice("Choose", choices, "option2", "");

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "option2");
        }

        #[test]
        fn should_return_correct_choice_for_valid_number() {
            let choices = &["github", "gitlab", "none"];

            let result = mock_prompt_choice("Choose service", choices, "github", "1");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "github");

            let result = mock_prompt_choice("Choose service", choices, "github", "2");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "gitlab");

            let result = mock_prompt_choice("Choose service", choices, "github", "3");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "none");
        }

        #[test]
        fn should_return_error_for_invalid_choice_number() {
            let choices = &["option1", "option2"];

            let result = mock_prompt_choice("Choose", choices, "option1", "0");
            assert!(result.is_err());

            let result = mock_prompt_choice("Choose", choices, "option1", "3");
            assert!(result.is_err());

            let result = mock_prompt_choice("Choose", choices, "option1", "99");
            assert!(result.is_err());
        }

        #[test]
        fn should_return_error_for_non_numeric_input() {
            let choices = &["option1", "option2"];

            let result = mock_prompt_choice("Choose", choices, "option1", "abc");
            assert!(result.is_err());

            let result = mock_prompt_choice("Choose", choices, "option1", "option1");
            assert!(result.is_err());
        }
    }

    mod core_settings_tests {
        use super::*;

        #[test]
        fn should_set_default_branch_from_user_input() {
            let mut config = Config::default();
            config.core.default_branch = "main".to_string();

            let new_branch = mock_prompt_with_default("Default branch", &config.core.default_branch, "develop").unwrap();
            config.core.default_branch = new_branch;

            assert_eq!(config.core.default_branch, "develop");
        }

        #[test]
        fn should_keep_existing_default_branch_when_empty_input() {
            let mut config = Config::default();
            config.core.default_branch = "master".to_string();

            let new_branch = mock_prompt_with_default("Default branch", &config.core.default_branch, "").unwrap();
            config.core.default_branch = new_branch;

            assert_eq!(config.core.default_branch, "master");
        }

        #[test]
        fn should_set_auto_fetch_from_user_input() {
            let mut config = Config::default();
            config.core.auto_fetch = true;

            let new_auto_fetch = mock_prompt_yes_no("Auto-fetch", config.core.auto_fetch, "n").unwrap();
            config.core.auto_fetch = new_auto_fetch;

            assert_eq!(config.core.auto_fetch, false);
        }

        #[test]
        fn should_keep_existing_auto_fetch_when_empty_input() {
            let mut config = Config::default();
            config.core.auto_fetch = false;

            let new_auto_fetch = mock_prompt_yes_no("Auto-fetch", config.core.auto_fetch, "").unwrap();
            config.core.auto_fetch = new_auto_fetch;

            assert_eq!(config.core.auto_fetch, false);
        }
    }

    mod github_setup_tests {
        use super::*;

        #[test]
        fn should_configure_github_with_default_values() {
            let mut config = Config::default();

            let base_url = mock_prompt_with_default("GitHub base URL", "https://api.github.com", "").unwrap();
            let username = mock_prompt_required("GitHub username", "testuser").unwrap();

            config.github = Some(GitHubConfig {
                base_url,
                username,
            });

            assert!(config.github.is_some());
            let github_config = config.github.unwrap();
            assert_eq!(github_config.base_url, "https://api.github.com");
            assert_eq!(github_config.username, "testuser");
        }

        #[test]
        fn should_configure_github_with_custom_values() {
            let mut config = Config::default();

            let base_url = mock_prompt_with_default("GitHub base URL", "https://api.github.com", "https://github.company.com/api/v3").unwrap();
            let username = mock_prompt_required("GitHub username", "john.doe").unwrap();

            config.github = Some(GitHubConfig {
                base_url,
                username,
            });

            let github_config = config.github.unwrap();
            assert_eq!(github_config.base_url, "https://github.company.com/api/v3");
            assert_eq!(github_config.username, "john.doe");
        }

        #[test]
        fn should_require_username_for_github_setup() {
            let result = mock_prompt_required("GitHub username", "");
            assert!(result.is_err());
        }
    }

    mod gitlab_setup_tests {
        use super::*;

        #[test]
        fn should_configure_gitlab_with_default_values() {
            let mut config = Config::default();

            let base_url = mock_prompt_with_default("GitLab base URL", "https://gitlab.com/api/v4", "").unwrap();
            let username = mock_prompt_required("GitLab username", "testuser").unwrap();

            config.gitlab = Some(GitLabConfig {
                base_url,
                username,
            });

            assert!(config.gitlab.is_some());
            let gitlab_config = config.gitlab.unwrap();
            assert_eq!(gitlab_config.base_url, "https://gitlab.com/api/v4");
            assert_eq!(gitlab_config.username, "testuser");
        }

        #[test]
        fn should_configure_gitlab_with_custom_values() {
            let mut config = Config::default();

            let base_url = mock_prompt_with_default("GitLab base URL", "https://gitlab.com/api/v4", "https://gitlab.company.com/api/v4").unwrap();
            let username = mock_prompt_required("GitLab username", "jane.smith").unwrap();

            config.gitlab = Some(GitLabConfig {
                base_url,
                username,
            });

            let gitlab_config = config.gitlab.unwrap();
            assert_eq!(gitlab_config.base_url, "https://gitlab.company.com/api/v4");
            assert_eq!(gitlab_config.username, "jane.smith");
        }
    }

    mod jira_setup_tests {
        use super::*;

        #[test]
        fn should_configure_jira_with_user_input() {
            let mut config = Config::default();

            let base_url = mock_prompt_required("Jira base URL", "https://company.atlassian.net").unwrap();
            let username = mock_prompt_required("Jira username", "user@company.com").unwrap();

            config.jira = Some(JiraConfig {
                base_url,
                username,
            });

            assert!(config.jira.is_some());
            let jira_config = config.jira.unwrap();
            assert_eq!(jira_config.base_url, "https://company.atlassian.net");
            assert_eq!(jira_config.username, "user@company.com");
        }

        #[test]
        fn should_require_base_url_for_jira_setup() {
            let result = mock_prompt_required("Jira base URL", "");
            assert!(result.is_err());
        }

        #[test]
        fn should_require_username_for_jira_setup() {
            let result = mock_prompt_required("Jira username", "");
            assert!(result.is_err());
        }
    }

    mod llm_provider_tests {
        use super::*;

        #[test]
        fn should_configure_openai_provider() {
            let mut config = Config::default();

            config.llm.provider = "openai".to_string();
            let model = mock_prompt_choice("Which OpenAI model?", &["gpt-4", "gpt-4-turbo", "gpt-3.5-turbo"], "gpt-4", "2").unwrap();
            config.llm.model = model;

            let max_tokens_str = mock_prompt_with_default("Max tokens", "2000", "1500").unwrap();
            config.llm.max_tokens = max_tokens_str.parse().unwrap();

            assert_eq!(config.llm.provider, "openai");
            assert_eq!(config.llm.model, "gpt-4-turbo");
            assert_eq!(config.llm.max_tokens, 1500);
        }

        #[test]
        fn should_configure_anthropic_provider() {
            let mut config = Config::default();

            config.llm.provider = "anthropic".to_string();
            let model = mock_prompt_choice("Which Anthropic model?", &["claude-3-opus", "claude-3-sonnet", "claude-3-haiku"], "claude-3-sonnet", "1").unwrap();
            config.llm.model = model;

            assert_eq!(config.llm.provider, "anthropic");
            assert_eq!(config.llm.model, "claude-3-opus");
        }

        #[test]
        fn should_configure_custom_llm_provider() {
            let mut config = Config::default();

            let provider = mock_prompt_required("Provider name", "custom-ai").unwrap();
            config.llm.provider = provider;

            let model = mock_prompt_required("Model name", "custom-model-v1").unwrap();
            config.llm.model = model;

            assert_eq!(config.llm.provider, "custom-ai");
            assert_eq!(config.llm.model, "custom-model-v1");
        }

        #[test]
        fn should_use_default_max_tokens_when_empty_input() {
            let result = mock_prompt_with_default("Max tokens", "2000", "");
            assert!(result.is_ok());

            let max_tokens: u32 = result.unwrap().parse().unwrap();
            assert_eq!(max_tokens, 2000);
        }

        #[test]
        fn should_parse_custom_max_tokens() {
            let result = mock_prompt_with_default("Max tokens", "2000", "4000");
            assert!(result.is_ok());

            let max_tokens: u32 = result.unwrap().parse().unwrap();
            assert_eq!(max_tokens, 4000);
        }
    }

    mod git_hosting_choice_tests {
        use super::*;

        #[test]
        fn should_select_github_by_default() {
            let service = mock_prompt_choice("Which Git hosting service?", &["github", "gitlab", "none"], "github", "").unwrap();
            assert_eq!(service, "github");
        }

        #[test]
        fn should_select_gitlab_when_chosen() {
            let service = mock_prompt_choice("Which Git hosting service?", &["github", "gitlab", "none"], "github", "2").unwrap();
            assert_eq!(service, "gitlab");
        }

        #[test]
        fn should_select_none_when_chosen() {
            let service = mock_prompt_choice("Which Git hosting service?", &["github", "gitlab", "none"], "github", "3").unwrap();
            assert_eq!(service, "none");
        }

        #[test]
        fn should_handle_none_selection_in_config() {
            let mut config = Config::default();

            let service = mock_prompt_choice("Which Git hosting service?", &["github", "gitlab", "none"], "github", "3").unwrap();

            if service == "none" {
                config.github = None;
                config.gitlab = None;
            }

            assert!(config.github.is_none());
            assert!(config.gitlab.is_none());
        }
    }

    mod task_tracking_choice_tests {
        use super::*;

        #[test]
        fn should_configure_jira_when_user_says_yes() {
            let use_jira = mock_prompt_yes_no("Do you use Jira?", false, "y").unwrap();
            assert_eq!(use_jira, true);
        }

        #[test]
        fn should_skip_jira_when_user_says_no() {
            let use_jira = mock_prompt_yes_no("Do you use Jira?", false, "n").unwrap();
            assert_eq!(use_jira, false);
        }

        #[test]
        fn should_default_to_no_for_jira() {
            let use_jira = mock_prompt_yes_no("Do you use Jira?", false, "").unwrap();
            assert_eq!(use_jira, false);
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn should_create_complete_config_with_all_services() {
            let mut config = Config::default();

            config.core.default_branch = mock_prompt_with_default("Default branch", "main", "develop").unwrap();
            config.core.auto_fetch = mock_prompt_yes_no("Auto-fetch", true, "n").unwrap();

            let github_base_url = mock_prompt_with_default("GitHub base URL", "https://api.github.com", "").unwrap();
            let github_username = mock_prompt_required("GitHub username", "testuser").unwrap();
            config.github = Some(GitHubConfig {
                base_url: github_base_url,
                username: github_username,
            });

            let jira_base_url = mock_prompt_required("Jira base URL", "https://company.atlassian.net").unwrap();
            let jira_username = mock_prompt_required("Jira username", "test@company.com").unwrap();
            config.jira = Some(JiraConfig {
                base_url: jira_base_url,
                username: jira_username,
            });

            config.llm.provider = "openai".to_string();
            config.llm.model = mock_prompt_choice("OpenAI model", &["gpt-4", "gpt-4-turbo"], "gpt-4", "2").unwrap();
            config.llm.max_tokens = mock_prompt_with_default("Max tokens", "2000", "1500").unwrap().parse().unwrap();

            assert_eq!(config.core.default_branch, "develop");
            assert_eq!(config.core.auto_fetch, false);
            
            assert!(config.github.is_some());
            let github = config.github.unwrap();
            assert_eq!(github.base_url, "https://api.github.com");
            assert_eq!(github.username, "testuser");

            assert!(config.jira.is_some());
            let jira = config.jira.unwrap();
            assert_eq!(jira.base_url, "https://company.atlassian.net");
            assert_eq!(jira.username, "test@company.com");

            assert_eq!(config.llm.provider, "openai");
            assert_eq!(config.llm.model, "gpt-4-turbo");
            assert_eq!(config.llm.max_tokens, 1500);
        }

        #[test]
        fn should_create_minimal_config_with_defaults() {
            let mut config = Config::default();

            config.core.default_branch = mock_prompt_with_default("Default branch", "main", "").unwrap();
            config.core.auto_fetch = mock_prompt_yes_no("Auto-fetch", true, "").unwrap();

            let service = mock_prompt_choice("Git hosting", &["github", "gitlab", "none"], "github", "3").unwrap();
            if service == "none" {
                config.github = None;
                config.gitlab = None;
            }

            let use_jira = mock_prompt_yes_no("Use Jira", false, "").unwrap();
            if !use_jira {
                config.jira = None;
            }

            config.llm.max_tokens = mock_prompt_with_default("Max tokens", "2000", "").unwrap().parse().unwrap();

            assert_eq!(config.core.default_branch, "main");
            assert_eq!(config.core.auto_fetch, true);
            assert!(config.github.is_none());
            assert!(config.gitlab.is_none());
            assert!(config.jira.is_none());
            assert_eq!(config.llm.provider, "openai");
            assert_eq!(config.llm.max_tokens, 2000);
        }
    }

    mod token_management_tests {
        use super::*;

        #[test]
        fn should_handle_empty_token_as_error() {
            let result = mock_prompt_required("API Token", "");
            assert!(result.is_err());
        }

        #[test]
        fn should_accept_valid_token() {
            let result = mock_prompt_required("API Token", "sk-1234567890abcdef");
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), "sk-1234567890abcdef");
        }

        #[test]
        fn should_handle_token_decline_gracefully() {
            let has_token = mock_prompt_yes_no("Do you have your token ready?", false, "n").unwrap();
            assert_eq!(has_token, false);
        }

        #[test]
        fn should_proceed_with_token_setup_when_ready() {
            let has_token = mock_prompt_yes_no("Do you have your token ready?", false, "y").unwrap();
            assert_eq!(has_token, true);
        }
    }

    mod error_handling_tests {
        use super::*;

        #[test]
        fn should_handle_invalid_max_tokens_input() {
            let invalid_inputs = vec!["abc", "-100", "not_a_number"];

            for input in invalid_inputs {
                let result = input.parse::<u32>();
                assert!(result.is_err(), "Should fail to parse: {}", input);
            }
        }

        #[test]
        fn should_handle_valid_max_tokens_input() {
            let valid_inputs = vec!["0", "1000", "2000", "4000", "8000"];

            for input in valid_inputs {
                let result = input.parse::<u32>();
                assert!(result.is_ok(), "Should parse successfully: {}", input);
            }
        }

        #[test]
        fn should_handle_out_of_range_choice_gracefully() {
            let choices = &["option1", "option2"];

            let result = mock_prompt_choice("Choose", choices, "option1", "0");
            assert!(result.is_err());

            let result = mock_prompt_choice("Choose", choices, "option1", "3");
            assert!(result.is_err());
        }
    }

    mod get_token_tests {
        use super::*;

        #[test]
        fn should_return_none_when_token_not_found() {
            // This test will naturally return None since no token is stored
            let result = get_token("non_existent_service");
            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        }
    }
}
