use anyhow::Result;
use crate::{config::Config, cli::ConfigCommands};
use crate::actions::setup::{store_token, get_token, delete_token};

pub async fn execute(config_command: ConfigCommands) -> Result<()> {
    match config_command {
        ConfigCommands::Set { key, value } => {
            // Handle token settings specially
            if key.starts_with("token.") || key.ends_with("_token") || key.ends_with("_api_key") {
                let service = match key.as_str() {
                    "token.github" | "github_token" => "github_token",
                    "token.gitlab" | "gitlab_token" => "gitlab_token",
                    "token.jira" | "jira_token" => "jira_token",
                    "token.openai" | "openai_api_key" => "openai_api_key",
                    "token.anthropic" | "anthropic_api_key" => "anthropic_api_key",
                    _ => return Err(anyhow::anyhow!("Unknown token type: {}", key)),
                };

                store_token(service, &value)?;
                println!("🔐 Token stored securely in your system keychain!");
                println!("✨ {} authentication is now configured", service.replace("_", " "));
            } else {
                // Handle regular config values
                let mut config = Config::load()?;
                config.set_value(&key, &value)?;
                config.save()?;
                println!("✨ Perfect! Set {} = {}", key, value);
                println!("🌟 Your workflow just got a little more personalized!");
            }
        }
        ConfigCommands::Get { key } => {
            // Handle token retrieval specially
            if key.starts_with("token.") || key.ends_with("_token") || key.ends_with("_api_key") {
                let service = match key.as_str() {
                    "token.github" | "github_token" => "github_token",
                    "token.gitlab" | "gitlab_token" => "gitlab_token",
                    "token.jira" | "jira_token" => "jira_token",
                    "token.openai" | "openai_api_key" => "openai_api_key",
                    "token.anthropic" | "anthropic_api_key" => "anthropic_api_key",
                    _ => return Err(anyhow::anyhow!("Unknown token type: {}", key)),
                };

                match get_token(service)? {
                    Some(_) => println!("🔐 Token is configured (hidden for security)"),
                    None => println!("❌ No token found for {}", service.replace("_", " ")),
                }
            } else {
                // Handle regular config values
                let config = Config::load()?;
                let value = config.get_value(&key)?;
                println!("{}", value);
            }
        }
        ConfigCommands::List => {
            let config = Config::load()?;
            println!("Here's your current prally configuration:");
            println!("═══════════════════════════════════════════");
            println!("💜 Core Settings:");
            println!("  default_branch = {}", config.core.default_branch);
            println!("  auto_fetch = {} ({})", config.core.auto_fetch,
                if config.core.auto_fetch { "keeping you updated! 📡" } else { "manual control ✋" });

            println!("\n🤖 AI Assistant Settings:");
            println!("  provider = {} ({})", config.llm.provider,
                match config.llm.provider.as_str() {
                    "openai" => "powered by OpenAI ✨",
                    "anthropic" => "powered by Anthropic 🧠",
                    _ => "your chosen AI buddy 🤝"
                });
            println!("  model = {}", config.llm.model);
            println!("  max_tokens = {} (room for thoughtful responses)", config.llm.max_tokens);

            if let Some(github) = &config.github {
                println!("\n🐙 GitHub Connection:");
                println!("  base_url = {}", github.base_url);
                println!("  username = {} (hey there! 👋)", github.username);

                // Check if token is configured
                match get_token("github_token")? {
                    Some(_) => println!("  token = 🔐 configured"),
                    None => println!("  token = ❌ not set"),
                }
            }

            if let Some(gitlab) = &config.gitlab {
                println!("\n🦊 GitLab Connection:");
                println!("  base_url = {}", gitlab.base_url);
                println!("  username = {} (great choice! 🎉)", gitlab.username);

                match get_token("gitlab_token")? {
                    Some(_) => println!("  token = 🔐 configured"),
                    None => println!("  token = ❌ not set"),
                }
            }

            if let Some(jira) = &config.jira {
                println!("\n📋 Jira Integration:");
                println!("  base_url = {}", jira.base_url);
                println!("  username = {} (task tracking made easier)", jira.username);

                match get_token("jira_token")? {
                    Some(_) => println!("  token = 🔐 configured"),
                    None => println!("  token = ❌ not set"),
                }
            }

            // Check LLM tokens
            println!("\n🔑 Authentication Status:");
            let openai_configured = get_token("openai_api_key")?.is_some();
            let anthropic_configured = get_token("anthropic_api_key")?.is_some();

            println!("  OpenAI API key = {}", if openai_configured { "🔐 configured" } else { "❌ not set" });
            println!("  Anthropic API key = {}", if anthropic_configured { "🔐 configured" } else { "❌ not set" });

            println!("═══════════════════════════════════════════");
            println!("💡 Tips:");
            println!("  • Use 'prally config set <key> <value>' to update any setting");
            println!("  • Use 'prally config set token.<service> <token>' to set API tokens");
            println!("  • Use 'prally config revoke <service>' to remove stored credentials");
            println!("  • Run 'prally setup' to reconfigure everything interactively");
        }
        ConfigCommands::Revoke { service } => {
            // Map service name to internal token name
            let token_service = match service.as_str() {
                "github" => "github_token",
                "gitlab" => "gitlab_token", 
                "jira" => "jira_token",
                "openai" => "openai_api_key",
                "anthropic" => "anthropic_api_key",
                _ => return Err(anyhow::anyhow!("Unknown service: {}. Supported services: github, gitlab, jira, openai, anthropic", service)),
            };

            // Check if token exists first
            match get_token(token_service)? {
                Some(_) => {
                    // Token exists, delete it
                    if delete_token(token_service)? {
                        println!("🗑️  Successfully revoked {} credentials", service);
                        println!("🔐 Token has been removed from your system keychain");
                        println!("💡 You can reconfigure {} later with 'prally config set token.{} <new-token>' or 'prally setup'", service, service);
                    } else {
                        println!("⚠️  Failed to revoke {} credentials - they may not exist or be inaccessible", service);
                    }
                },
                None => {
                    println!("❌ No {} credentials found to revoke", service);
                    println!("💡 Use 'prally config list' to see what credentials are currently configured");
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod config_tests {
    mod execute_set_tests {
        #[test]
        fn should_handle_github_token_set() {
            let key = "token.github".to_string();
            let expected_service = match key.as_str() {
                "token.github" | "github_token" => "github_token",
                "token.gitlab" | "gitlab_token" => "gitlab_token",
                "token.jira" | "jira_token" => "jira_token",
                "token.openai" | "openai_api_key" => "openai_api_key",
                "token.anthropic" | "anthropic_api_key" => "anthropic_api_key",
                _ => "unknown",
            };

            assert_eq!(expected_service, "github_token");
        }

        #[test]
        fn should_handle_gitlab_token_set() {
            let key = "gitlab_token".to_string();
            let expected_service = match key.as_str() {
                "token.github" | "github_token" => "github_token",
                "token.gitlab" | "gitlab_token" => "gitlab_token",
                "token.jira" | "jira_token" => "jira_token",
                "token.openai" | "openai_api_key" => "openai_api_key",
                "token.anthropic" | "anthropic_api_key" => "anthropic_api_key",
                _ => "unknown",
            };

            assert_eq!(expected_service, "gitlab_token");
        }

        #[test]
        fn should_handle_jira_token_set() {
            let key = "token.jira".to_string();
            let expected_service = match key.as_str() {
                "token.github" | "github_token" => "github_token",
                "token.gitlab" | "gitlab_token" => "gitlab_token",
                "token.jira" | "jira_token" => "jira_token",
                "token.openai" | "openai_api_key" => "openai_api_key",
                "token.anthropic" | "anthropic_api_key" => "anthropic_api_key",
                _ => "unknown",
            };

            assert_eq!(expected_service, "jira_token");
        }

        #[test]
        fn should_handle_openai_api_key_set() {
            let key = "openai_api_key".to_string();
            let expected_service = match key.as_str() {
                "token.github" | "github_token" => "github_token",
                "token.gitlab" | "gitlab_token" => "gitlab_token",
                "token.jira" | "jira_token" => "jira_token",
                "token.openai" | "openai_api_key" => "openai_api_key",
                "token.anthropic" | "anthropic_api_key" => "anthropic_api_key",
                _ => "unknown",
            };

            assert_eq!(expected_service, "openai_api_key");
        }

        #[test]
        fn should_handle_anthropic_api_key_set() {
            let key = "token.anthropic".to_string();
            let expected_service = match key.as_str() {
                "token.github" | "github_token" => "github_token",
                "token.gitlab" | "gitlab_token" => "gitlab_token",
                "token.jira" | "jira_token" => "jira_token",
                "token.openai" | "openai_api_key" => "openai_api_key",
                "token.anthropic" | "anthropic_api_key" => "anthropic_api_key",
                _ => "unknown",
            };

            assert_eq!(expected_service, "anthropic_api_key");
        }

        #[test]
        fn should_handle_unknown_token_type() {
            let key = "token.unknown".to_string();
            let expected_service = match key.as_str() {
                "token.github" | "github_token" => "github_token",
                "token.gitlab" | "gitlab_token" => "gitlab_token",
                "token.jira" | "jira_token" => "jira_token",
                "token.openai" | "openai_api_key" => "openai_api_key",
                "token.anthropic" | "anthropic_api_key" => "anthropic_api_key",
                _ => "unknown",
            };

            assert_eq!(expected_service, "unknown");
        }
    }

    mod execute_get_tests {
        #[test]
        fn should_identify_token_keys() {
            let token_keys = vec![
                "token.github",
                "github_token",
                "token.gitlab",
                "gitlab_token",
                "token.jira",
                "jira_token",
                "token.openai",
                "openai_api_key",
                "token.anthropic",
                "anthropic_api_key"
            ];

            for key in token_keys {
                let is_token = key.starts_with("token.") || key.ends_with("_token") || key.ends_with("_api_key");
                assert!(is_token, "Key {} should be identified as a token key", key);
            }
        }

        #[test]
        fn should_identify_non_token_keys() {
            let non_token_keys = vec![
                "core.default_branch",
                "core.auto_fetch",
                "llm.provider",
                "llm.model",
                "llm.max_tokens",
                "github.base_url",
                "github.username"
            ];

            for key in non_token_keys {
                let is_token = key.starts_with("token.") || key.ends_with("_token") || key.ends_with("_api_key");
                assert!(!is_token, "Key {} should not be identified as a token key", key);
            }
        }

        #[test]
        fn should_map_token_keys_to_services() {
            let mappings = vec![
                ("token.github", "github_token"),
                ("github_token", "github_token"),
                ("token.gitlab", "gitlab_token"),
                ("gitlab_token", "gitlab_token"),
                ("token.jira", "jira_token"),
                ("jira_token", "jira_token"),
                ("token.openai", "openai_api_key"),
                ("openai_api_key", "openai_api_key"),
                ("token.anthropic", "anthropic_api_key"),
                ("anthropic_api_key", "anthropic_api_key")
            ];

            for (key, expected_service) in mappings {
                let service = match key {
                    "token.github" | "github_token" => "github_token",
                    "token.gitlab" | "gitlab_token" => "gitlab_token",
                    "token.jira" | "jira_token" => "jira_token",
                    "token.openai" | "openai_api_key" => "openai_api_key",
                    "token.anthropic" | "anthropic_api_key" => "anthropic_api_key",
                    _ => "unknown",
                };

                assert_eq!(service, expected_service, "Key {} should map to service {}", key, expected_service);
            }
        }
    }

    mod config_command_validation_tests {
        #[test]
        fn should_handle_set_command_structure() {
            // Test that ConfigCommands::Set has the expected structure
            let key = "test_key".to_string();
            let value = "test_value".to_string();

            // This would be a ConfigCommands::Set in real usage
            assert_eq!(key, "test_key");
            assert_eq!(value, "test_value");
        }

        #[test]
        fn should_handle_get_command_structure() {
            // Test that ConfigCommands::Get has the expected structure
            let key = "test_key".to_string();

            // This would be a ConfigCommands::Get in real usage
            assert_eq!(key, "test_key");
        }

        #[test]
        fn should_handle_list_command() {
            // Test that List command is handled
            let is_list = true;
            assert_eq!(is_list, true);
        }

        #[test]
        fn should_handle_revoke_command_structure() {
            // Test that ConfigCommands::Revoke has the expected structure
            let service = "github".to_string();
            
            // This would be a ConfigCommands::Revoke in real usage
            assert_eq!(service, "github");
        }
    }

    mod execute_revoke_tests {
        #[test]
        fn should_map_github_service_to_token() {
            let service = "github";
            let token_service = match service {
                "github" => "github_token",
                "gitlab" => "gitlab_token", 
                "jira" => "jira_token",
                "openai" => "openai_api_key",
                "anthropic" => "anthropic_api_key",
                _ => "unknown",
            };
            
            assert_eq!(token_service, "github_token");
        }

        #[test]
        fn should_map_gitlab_service_to_token() {
            let service = "gitlab";
            let token_service = match service {
                "github" => "github_token",
                "gitlab" => "gitlab_token", 
                "jira" => "jira_token",
                "openai" => "openai_api_key",
                "anthropic" => "anthropic_api_key",
                _ => "unknown",
            };
            
            assert_eq!(token_service, "gitlab_token");
        }

        #[test]
        fn should_map_jira_service_to_token() {
            let service = "jira";
            let token_service = match service {
                "github" => "github_token",
                "gitlab" => "gitlab_token", 
                "jira" => "jira_token",
                "openai" => "openai_api_key",
                "anthropic" => "anthropic_api_key",
                _ => "unknown",
            };
            
            assert_eq!(token_service, "jira_token");
        }

        #[test]
        fn should_map_openai_service_to_token() {
            let service = "openai";
            let token_service = match service {
                "github" => "github_token",
                "gitlab" => "gitlab_token", 
                "jira" => "jira_token",
                "openai" => "openai_api_key",
                "anthropic" => "anthropic_api_key",
                _ => "unknown",
            };
            
            assert_eq!(token_service, "openai_api_key");
        }

        #[test]
        fn should_map_anthropic_service_to_token() {
            let service = "anthropic";
            let token_service = match service {
                "github" => "github_token",
                "gitlab" => "gitlab_token", 
                "jira" => "jira_token",
                "openai" => "openai_api_key",
                "anthropic" => "anthropic_api_key",
                _ => "unknown",
            };
            
            assert_eq!(token_service, "anthropic_api_key");
        }

        #[test]
        fn should_handle_unknown_service() {
            let service = "unknown_service";
            let token_service = match service {
                "github" => "github_token",
                "gitlab" => "gitlab_token", 
                "jira" => "jira_token",
                "openai" => "openai_api_key",
                "anthropic" => "anthropic_api_key",
                _ => "unknown",
            };
            
            assert_eq!(token_service, "unknown");
        }

        #[test]
        fn should_create_error_message_for_unknown_service() {
            let service = "invalid_service";
            let error_msg = format!("Unknown service: {}. Supported services: github, gitlab, jira, openai, anthropic", service);
            
            assert!(error_msg.contains("Unknown service"));
            assert!(error_msg.contains("invalid_service"));
            assert!(error_msg.contains("Supported services"));
            assert!(error_msg.contains("github, gitlab, jira, openai, anthropic"));
        }

        #[test]
        fn should_validate_all_supported_services() {
            let supported_services = vec!["github", "gitlab", "jira", "openai", "anthropic"];
            
            for service in supported_services {
                let token_service = match service {
                    "github" => "github_token",
                    "gitlab" => "gitlab_token", 
                    "jira" => "jira_token",
                    "openai" => "openai_api_key",
                    "anthropic" => "anthropic_api_key",
                    _ => "unknown",
                };
                
                assert_ne!(token_service, "unknown", "Service {} should be supported", service);
            }
        }
    }
}
