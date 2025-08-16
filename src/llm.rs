use reqwest::Client;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::config::Config;
use crate::context::{ContextGroup, ContextUri};
use crate::error::{PrallyError, Result};
use std::collections::HashMap;

/// Global default system instruction for all AI providers
pub fn get_default_system_instruction() -> String {
    "You are an expert developer assistant that helps generate high-quality pull request descriptions. Your responses should be helpful, inclusive, and supportive of all developers.".to_string()
}

/// Get the system instruction from config or use the global default
pub fn get_system_instruction(config: &Config, request_instruction: Option<&String>) -> String {
    // Priority: request-specific > config > global default
    if let Some(instruction) = request_instruction {
        return instruction.clone();
    }

    if let Some(instruction) = &config.llm.system_instruction {
        return instruction.clone();
    }

    get_default_system_instruction()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmRequest {
    pub prompt_template: String,
    pub context: ContextGroup,
    pub max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmResponse {
    pub content: String,
    pub usage: Option<UsageStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageStats {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[async_trait]
pub trait LlmProvider {
    async fn generate(&self, config: &Config, request: LlmRequest) -> Result<LlmResponse>;
    async fn get_api_key(&self) -> Result<String>;
}

pub struct OpenAiProvider {
    model: String,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(model: String) -> Self {
        Self {
            model,
            client: Client::new(),
        }
    }

    pub fn from_config(config: &Config) -> Self {
        Self::new(config.llm.model.clone())
    }

    fn build_input(&self, request: &LlmRequest) -> String {
        let mut template = request.prompt_template.clone();

        // Create placeholder replacements based on context
        let mut replacements = HashMap::new();

        // Process consolidated placeholders
        replacements.insert("INSTRUCTION".to_string(), request.context.get_instruction_content());
        replacements.insert("CONTEXT".to_string(), request.context.get_context_content());

        // Process git-specific context items
        if let Some(git_diff_item) = request.context.find_item_by_uri(&ContextUri::GitDiff) {
            let formatted_diff = format!("## Git Diff\n```diff\n{}\n```", git_diff_item.content);
            replacements.insert("GIT_DIFF".to_string(), formatted_diff);
        }

        if let Some(commit_item) = request.context.find_item_by_uri(&ContextUri::CommitMessages) {
            let count = commit_item.metadata
                .as_ref()
                .and_then(|m| m.get("count"))
                .map(|c| format!(" ({} commits)", c))
                .unwrap_or_default();
            let formatted_commits = format!("## Commit Messages{}\n```\n{}\n```", count, commit_item.content);
            replacements.insert("COMMIT_MESSAGES".to_string(), formatted_commits);
        }

        // Handle legacy placeholders for backward compatibility
        for item in &request.context.items {
            match &item.uri {
                ContextUri::FileContent(path) => {
                    let formatted_content = format!("## File: {}\n```\n{}\n```", path, item.content);
                    let placeholder = format!("FILE_CONTENT_{}",
                        path.replace(['/', '.', '-'], "_").to_uppercase());
                    replacements.insert(placeholder, formatted_content);
                }
                ContextUri::Custom(uri) => {
                    let formatted_content = format!("## {}\n{}", uri, item.content);
                    let placeholder = format!("CUSTOM_{}",
                        uri.replace(['/', '.', '-', ':'], "_").to_uppercase());
                    replacements.insert(placeholder, formatted_content);
                }
                _ => {} // Other types are handled by consolidated placeholders
            }
        }

        // Replace placeholders in template
        for (placeholder, content) in replacements {
            template = template.replace(&format!("{{{{{}}}}}", placeholder), &content);
        }

        // Remove any unused placeholders (optional - could leave them for debugging)
        template = self.clean_unused_placeholders(template);

        template
    }

    fn clean_unused_placeholders(&self, mut template: String) -> String {
        // Remove common unused placeholders with empty content
        let common_placeholders = vec![
            "{{GIT_DIFF}}",
            "{{TASK_DESCRIPTION}}",
            "{{FILE_CONTENT}}",
        ];

        for placeholder in common_placeholders {
            template = template.replace(placeholder, "");
        }

        // Clean up multiple consecutive newlines
        while template.contains("\n\n\n") {
            template = template.replace("\n\n\n", "\n\n");
        }

        template.trim().to_string()
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn generate(&self, config: &Config, request: LlmRequest) -> Result<LlmResponse> {
        let api_key = self.get_api_key().await?;
        let input = self.build_input(&request);

        // Use the global system instruction function
        let instructions = get_system_instruction(config, None);

        let payload = serde_json::json!({
            "model": self.model,
            "reasoning": {"effort": "low"},
            "instructions": instructions,
            "input": input
        });

        let response = self.client
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(&api_key)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PrallyError::Llm(
                format!("OpenAI API request failed: HTTP {}", response.status())
            ));
        }

        let json: serde_json::Value = response.json().await?;

        let content = json["output"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let usage = json["usage"].as_object().map(|u| UsageStats {
            prompt_tokens: u["prompt_tokens"].as_u64().unwrap_or(0) as u32,
            completion_tokens: u["completion_tokens"].as_u64().unwrap_or(0) as u32,
            total_tokens: u["total_tokens"].as_u64().unwrap_or(0) as u32,
        });

        Ok(LlmResponse { content, usage })
    }

    async fn get_api_key(&self) -> Result<String> {
        let keyring = keyring::Entry::new("prally", "openai_api_key")
            .map_err(|e| PrallyError::Auth(e.to_string()))?;

        keyring
            .get_password()
            .map_err(|e| PrallyError::Auth(format!("OpenAI API key not found: {}", e)))
    }
}

pub fn get_llm_provider(config: &Config) -> Result<Box<dyn LlmProvider>> {
    match config.llm.provider.as_str() {
        "openai" => Ok(Box::new(OpenAiProvider::from_config(config))),
        provider => Err(PrallyError::Llm(format!("Unsupported LLM provider: {}", provider))),
    }
}
