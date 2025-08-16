use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrallyError {
    #[error("Git operation failed: {0}")]
    Git(#[from] git2::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Task tracker error: {0}")]
    TaskTracker(String),

    #[error("LLM provider error: {0}")]
    Llm(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, PrallyError>;
