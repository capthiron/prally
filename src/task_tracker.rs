use reqwest::Client;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::config::Config;
use crate::error::{PrallyError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[async_trait]
pub trait TaskTracker {
    async fn get_task(&self, task_id: &str) -> Result<Task>;
    async fn get_api_token(&self) -> Result<String>;
}

pub struct JiraClient {
    base_url: String,
    username: String,
    client: Client,
}

impl JiraClient {
    pub fn new(base_url: String, username: String) -> Self {
        Self {
            base_url,
            username,
            client: Client::new(),
        }
    }

    pub fn from_config(config: &Config) -> Result<Option<Self>> {
        if let Some(jira_config) = &config.jira {
            Ok(Some(Self::new(
                jira_config.base_url.clone(),
                jira_config.username.clone(),
            )))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl TaskTracker for JiraClient {
    async fn get_task(&self, task_id: &str) -> Result<Task> {
        let token = self.get_api_token().await?;
        let url = format!("{}/rest/api/3/issue/{}", self.base_url, task_id);

        let response = self.client
            .get(&url)
            .basic_auth(&self.username, Some(&token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(PrallyError::TaskTracker(
                format!("Failed to fetch task {}: HTTP {}", task_id, response.status())
            ));
        }

        let json: serde_json::Value = response.json().await?;

        let task = Task {
            id: task_id.to_string(),
            title: json["fields"]["summary"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            description: json["fields"]["description"]
                .as_str()
                .unwrap_or("")
                .to_string(),
        };

        Ok(task)
    }

    async fn get_api_token(&self) -> Result<String> {
        let keyring = keyring::Entry::new("prally", "jira_token")
            .map_err(|e| PrallyError::Auth(e.to_string()))?;

        keyring
            .get_password()
            .map_err(|e| PrallyError::Auth(format!("Jira API token not found: {}", e)))
    }
}

pub async fn get_task_tracker(config: &Config) -> Result<Option<Box<dyn TaskTracker>>> {
    if let Some(jira_client) = JiraClient::from_config(config)? {
        Ok(Some(Box::new(jira_client)))
    } else {
        Ok(None)
    }
}
