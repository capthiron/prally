use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the type of context resource
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextUri {
    GitDiff,
    TaskDescription,
    CommitMessages,
    CustomInstruction,
    CustomContext,
    ContextFile(String),
    PipedInput,
    FileContent(String), // For future use with specific file paths
    Custom(String),
}

impl ContextUri {
    pub fn as_str(&self) -> String {
        match self {
            ContextUri::GitDiff => "item://git-diff".to_string(),
            ContextUri::TaskDescription => "item://task-description".to_string(),
            ContextUri::CommitMessages => "item://commit-messages".to_string(),
            ContextUri::CustomInstruction => "item://custom-instruction".to_string(),
            ContextUri::CustomContext => "item://custom-context".to_string(),
            ContextUri::ContextFile(path) => format!("file://{}", path),
            ContextUri::PipedInput => "item://piped-input".to_string(),
            ContextUri::FileContent(path) => path.clone(),
            ContextUri::Custom(uri) => uri.clone(),
        }
    }
}

impl From<String> for ContextUri {
    fn from(uri: String) -> Self {
        match uri.as_str() {
            "item://git-diff" => ContextUri::GitDiff,
            "item://task-description" => ContextUri::TaskDescription,
            _ => ContextUri::Custom(uri),
        }
    }
}

/// Represents the MIME type of context content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContextMimeType {
    TextDiff,
    TextMarkdown,
    TextPlain,
    ApplicationJson,
    Custom(String),
}

impl ContextMimeType {
    pub fn as_str(&self) -> &str {
        match self {
            ContextMimeType::TextDiff => "text/x-diff",
            ContextMimeType::TextMarkdown => "text/markdown",
            ContextMimeType::TextPlain => "text/plain",
            ContextMimeType::ApplicationJson => "application/json",
            ContextMimeType::Custom(mime) => mime,
        }
    }
}

impl From<String> for ContextMimeType {
    fn from(mime: String) -> Self {
        match mime.as_str() {
            "text/x-diff" => ContextMimeType::TextDiff,
            "text/markdown" => ContextMimeType::TextMarkdown,
            "text/plain" => ContextMimeType::TextPlain,
            "application/json" => ContextMimeType::ApplicationJson,
            _ => ContextMimeType::Custom(mime),
        }
    }
}

/// Represents a single context item in the MCP format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    pub uri: ContextUri,
    pub mime_type: ContextMimeType,
    pub content: String,
    pub metadata: Option<HashMap<String, String>>,
}

/// A group of related context items
#[derive(Debug, Serialize, Deserialize)]
pub struct ContextGroup {
    pub items: Vec<ContextItem>,
    pub metadata: Option<HashMap<String, String>>,
}

impl ContextGroup {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            metadata: None,
        }
    }

    pub fn add_item(&mut self, item: ContextItem) {
        self.items.push(item);
    }

    pub fn add_git_diff(&mut self, diff: String) {
        let item = ContextItem {
            uri: ContextUri::GitDiff,
            mime_type: ContextMimeType::TextDiff,
            content: diff,
            metadata: None,
        };
        self.add_item(item);
    }

    pub fn add_task_description(&mut self, title: String, description: String) {
        let mut metadata = HashMap::new();
        metadata.insert("title".to_string(), title);

        let item = ContextItem {
            uri: ContextUri::TaskDescription,
            mime_type: ContextMimeType::TextMarkdown,
            content: description,
            metadata: Some(metadata),
        };
        self.add_item(item);
    }

    pub fn add_commit_messages(&mut self, commit_messages: Vec<String>) {
        let content = commit_messages.join("\n\n");
        let mut metadata = HashMap::new();
        metadata.insert("count".to_string(), commit_messages.len().to_string());

        let item = ContextItem {
            uri: ContextUri::CommitMessages,
            mime_type: ContextMimeType::TextPlain,
            content,
            metadata: Some(metadata),
        };
        self.add_item(item);
    }

    pub fn add_custom_instruction(&mut self, instruction: String) {
        let item = ContextItem {
            uri: ContextUri::CustomInstruction,
            mime_type: ContextMimeType::TextPlain,
            content: instruction,
            metadata: None,
        };
        self.add_item(item);
    }

    pub fn add_custom_context(&mut self, context: String) {
        let item = ContextItem {
            uri: ContextUri::CustomContext,
            mime_type: ContextMimeType::TextPlain,
            content: context,
            metadata: None,
        };
        self.add_item(item);
    }

    pub fn add_context_file(&mut self, file_path: String, content: String) {
        let mut metadata = HashMap::new();
        metadata.insert("file_path".to_string(), file_path.clone());

        // Determine MIME type based on file extension
        let mime_type = if file_path.ends_with(".md") || file_path.ends_with(".markdown") {
            ContextMimeType::TextMarkdown
        } else if file_path.ends_with(".json") {
            ContextMimeType::ApplicationJson
        } else {
            ContextMimeType::TextPlain
        };

        let item = ContextItem {
            uri: ContextUri::ContextFile(file_path),
            mime_type,
            content,
            metadata: Some(metadata),
        };
        self.add_item(item);
    }

    pub fn add_piped_input(&mut self, content: String) {
        let item = ContextItem {
            uri: ContextUri::PipedInput,
            mime_type: ContextMimeType::TextPlain,
            content,
            metadata: None,
        };
        self.add_item(item);
    }

    pub fn find_item_by_uri(&self, target_uri: &ContextUri) -> Option<&ContextItem> {
        self.items.iter().find(|item| &item.uri == target_uri)
    }

    pub fn get_context_content(&self) -> String {
        let mut sections = Vec::new();

        // Add task description if present
        if let Some(task_item) = self.find_item_by_uri(&ContextUri::TaskDescription) {
            if let Some(title) = task_item.metadata.as_ref().and_then(|m| m.get("title")) {
                sections.push(format!("## Task: {}\n{}", title, task_item.content));
            } else {
                sections.push(format!("## Task Description\n{}", task_item.content));
            }
        }

        // Add custom context if present
        if let Some(custom_item) = self.find_item_by_uri(&ContextUri::CustomContext) {
            sections.push(format!("## Custom Context\n{}", custom_item.content));
        }

        // Add context files
        for item in &self.items {
            if let ContextUri::ContextFile(path) = &item.uri {
                sections.push(format!("## Context from {}\n{}", path, item.content));
            }
        }

        // Add piped input
        if let Some(piped_item) = self.find_item_by_uri(&ContextUri::PipedInput) {
            sections.push(format!("## Additional Context\n{}", piped_item.content));
        }

        sections.join("\n\n")
    }

    pub fn get_instruction_content(&self) -> String {
        if let Some(custom_item) = self.find_item_by_uri(&ContextUri::CustomInstruction) {
            format!("**Additional Instructions**: {}", custom_item.content)
        } else {
            String::new()
        }
    }

    pub fn to_mcp_format(&self) -> crate::error::Result<String> {
        // Convert to serializable format for MCP
        let serializable_group = SerializableContextGroup {
            items: self.items.iter().map(|item| SerializableContextItem {
                uri: item.uri.as_str(),
                mime_type: item.mime_type.as_str().to_string(),
                content: item.content.clone(),
                metadata: item.metadata.clone(),
            }).collect(),
            metadata: self.metadata.clone(),
        };
        Ok(serde_json::to_string_pretty(&serializable_group)?)
    }
}

// Helper structs for serialization to maintain string format for MCP
#[derive(Serialize)]
struct SerializableContextGroup {
    items: Vec<SerializableContextItem>,
    metadata: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
struct SerializableContextItem {
    uri: String,
    mime_type: String,
    content: String,
    metadata: Option<HashMap<String, String>>,
}

impl Default for ContextGroup {
    fn default() -> Self {
        Self::new()
    }
}
