use clap::{Parser, Subcommand};
use anyhow::Result;
use crate::actions;

#[derive(Parser)]
#[command(name = "prally")]
#[command(about = "A CLI ally for modern Git pull request workflows")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate a description for the current branch's PR
    Describe {
        /// Use manual/interactive mode
        #[arg(short, long)]
        manual: bool,

        /// Manually provide task title
        #[arg(long)]
        title: Option<String>,

        /// Manually provide task description
        #[arg(long)]
        desc: Option<String>,

        /// Generate only the prompt without executing
        #[arg(long)]
        prompt_only: bool,

        /// Use commit messages from current branch as context
        #[arg(long)]
        use_commits: bool,

        /// Provide a custom instruction to guide the LLM
        #[arg(short, long)]
        instruction: Option<String>,

        /// Provide custom context information
        #[arg(short = 'c', long)]
        custom_context: Option<String>,

        /// Specify a context file to use as additional input
        #[arg(short = 'f', long)]
        context_file: Option<String>,

        /// Read context from stdin (for piped input)
        #[arg(long)]
        from_stdin: bool,
    },

    /// Create a new pull request
    Create {
        /// Create as draft PR
        #[arg(long)]
        draft: bool,

        /// Automatically generate description after creation
        #[arg(long)]
        describe: bool,
    },

    /// Configure prally settings
    Config {
        #[command(subcommand)]
        config_command: ConfigCommands,
    },

    /// Run initial setup wizard
    Setup,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., "core.default_branch")
        key: String,
        /// Configuration value
        value: String,
    },

    /// Get a configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// List all configuration values
    List,

    /// Revoke stored credentials (deletes from keychain)
    Revoke {
        /// Service to revoke credentials for (e.g., "github", "gitlab", "jira", "openai", "anthropic")
        service: String,
    },
}

impl Cli {
    pub async fn execute(self) -> Result<()> {
        match self.command {
            Commands::Describe { manual, title, desc, prompt_only, use_commits, instruction, custom_context, context_file, from_stdin } => {
                actions::describe::execute(manual, title, desc, prompt_only, use_commits, instruction, custom_context, context_file, from_stdin).await
            }
            Commands::Create { draft, describe } => {
                actions::create::execute(draft, describe).await
            }
            Commands::Config { config_command } => {
                actions::config::execute(config_command).await
            }
            Commands::Setup => {
                actions::setup::execute().await
            }
        }
    }
}
