mod cli;
mod config;
mod vcs;
mod task_tracker;
mod llm;
mod actions;
mod context;
mod error;

use anyhow::Result;
use cli::Cli;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute the command
    cli.execute().await
}
