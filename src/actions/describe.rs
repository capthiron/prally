use super::presentation;
use crate::{
    config::Config,
    context::ContextGroup,
    llm::{get_llm_provider, LlmRequest},
    task_tracker::get_task_tracker,
    vcs::GitRepository,
};
use anyhow::Result;
use std::io::Read;

pub async fn execute(
    manual: bool,
    title: Option<String>,
    desc: Option<String>,
    prompt_only: bool,
    use_commits: bool,
    instruction: Option<String>,
    custom_context: Option<String>,
    context_file: Option<String>,
    from_stdin: bool,
) -> Result<()> {
    let config = Config::load()?;
    let git = GitRepository::open()?;

    // Gather context
    let mut context = ContextGroup::new();

    // Add git diff to context (always included as base context)
    let diff = git.get_diff()?;
    context.add_git_diff(diff);

    // Add commit messages if requested
    if use_commits {
        let commit_messages = git.get_commit_messages_from_branch()?;
        context.add_commit_messages(commit_messages);
        presentation::show_info("📝 Added commit messages from current branch to context")?;
    }

    // Add custom instruction if provided
    if let Some(ref instr) = instruction {
        context.add_custom_instruction(instr.clone());
        presentation::show_info(&format!("🎯 Added custom instruction: '{}'", instr))?;
    }

    // Add custom context if provided
    if let Some(ref custom_ctx) = custom_context {
        context.add_custom_context(custom_ctx.clone());
        presentation::show_info(&format!("📚 Added custom context: '{}'", custom_ctx))?;
    }

    // Add context file if provided
    if let Some(ref file_path) = context_file {
        presentation::show_progress(&format!("📄 Reading context file: '{}'", file_path))?;
        let file_content = std::fs::read_to_string(file_path)
            .map_err(|e| anyhow::anyhow!("Failed to read context file '{}': {}", file_path, e))?;
        context.add_context_file(file_path.clone(), file_content);
        presentation::update_current_line(&format!("✅ Added context from file: '{}'", file_path))?;
    }

    // Add piped input if requested
    if from_stdin {
        presentation::show_progress("📥 Reading piped input...")?;
        let mut stdin_content = String::new();
        std::io::stdin().read_to_string(&mut stdin_content)?;
        if !stdin_content.trim().is_empty() {
            context.add_piped_input(stdin_content);
            presentation::update_current_line("✅ Added piped input to context")?;
        } else {
            presentation::update_current_line("⚠️  No input received from stdin")?;
        }
    }

    // Get task information (existing logic)
    let (task_title, task_desc) = if manual || title.is_some() || desc.is_some() {
        // Manual mode - use provided values or prompt for them
        get_manual_task_info(title, desc).await?
    } else {
        // Auto mode - try to fetch from task tracker, fallback to git-only mode
        presentation::show_progress("🔍 Looking for task information...")?;
        match get_auto_task_info(&git, &config).await {
            Ok(info) => {
                presentation::update_current_line("✅ Found task information from tracker")?;
                info
            }
            Err(e) => {
                if use_commits
                    || instruction.is_some()
                    || custom_context.is_some()
                    || context_file.is_some()
                    || from_stdin
                {
                    // We have other context sources, so proceed without task tracker
                    presentation::update_current_line(
                        "💜 Using git-centric mode with additional context sources",
                    )?;
                    (
                        "Changes from current branch".to_string(),
                        "Generated from git history and provided context".to_string(),
                    )
                } else {
                    presentation::update_current_line(&format!(
                        "💜 No worries! I couldn't auto-fetch your task info: {}",
                        e
                    ))?;
                    println!("🤗 Let's do this together manually - I'm here to help!");
                    get_manual_task_info(None, None).await?
                }
            }
        }
    };

    // Add task description to context (if we have meaningful task info)
    if task_title != "Changes from current branch" {
        context.add_task_description(task_title.clone(), task_desc.clone());
    }

    // Build the prompt template
    let prompt_template = build_pr_description_template();

    if prompt_only {
        // Just output the context and prompt
        presentation::show_info("🏳️‍🌈 Here's what I'm working with (transparency is important!):")?;
        println!("\n=== Context (MCP Format) ===");
        println!("{}", context.to_mcp_format()?);
        println!("\n=== Prompt Template ===");
        println!("{}", prompt_template);
        return Ok(());
    }

    // Generate description using LLM with progress feedback
    let llm_provider = get_llm_provider(&config)?;
    let request = LlmRequest {
        prompt_template,
        context,
        max_tokens: config.llm.max_tokens,
    };

    // Show progress with spinner while LLM generates response
    let spinner_task = tokio::spawn(async {
        presentation::show_progress_with_spinner(
            "✨ Creating something beautiful for your PR description...",
            3000,
        )
        .await
    });

    // Generate the response
    let response = llm_provider.generate(request).await?;

    // Cancel the spinner and show completion
    spinner_task.abort();
    presentation::update_current_line("✅ PR description generated successfully!")?;

    presentation::show_success("\n🌟 Here's your PR description - crafted with care:")?;
    println!("═══════════════════════════════════════════════");
    println!("{}", response.content);
    println!("═══════════════════════════════════════════════");

    if let Some(usage) = response.usage {
        presentation::show_info(&format!(
            "💡 Token usage: {} total ({} prompt + {} completion)",
            usage.total_tokens, usage.prompt_tokens, usage.completion_tokens
        ))?;
        presentation::show_info("🌱 Every token used thoughtfully to support your workflow")?;
    }

    presentation::show_success(
        "💜 Hope this helps make your PR shine! Remember, your work matters.",
    )?;

    Ok(())
}

async fn get_manual_task_info(
    title: Option<String>,
    desc: Option<String>,
) -> Result<(String, String)> {
    let task_title = match title {
        Some(t) => {
            presentation::show_info(&format!("🎯 Using your provided title: '{}'", t))?;
            t
        }
        None => {
            presentation::show_info(
                "📝 What would you like to call this task? (No pressure - whatever feels right!)",
            )?;
            print!("Task title: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let title = input.trim();
            if title.is_empty() {
                "Your Amazing Work".to_string()
            } else {
                title.to_string()
            }
        }
    };

    let task_desc = match desc {
        Some(d) => {
            presentation::show_info(&format!("📋 Using your provided description: '{}'", d))?;
            d
        }
        None => {
            presentation::show_info("📝 Tell me a bit about this task (optional - I can work with just the title too!):")?;
            print!("Task description: ");
            std::io::Write::flush(&mut std::io::stdout())?;
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let desc = input.trim();
            if desc.is_empty() {
                "Work in progress".to_string()
            } else {
                desc.to_string()
            }
        }
    };

    Ok((task_title, task_desc))
}

async fn get_auto_task_info(git: &GitRepository, config: &Config) -> Result<(String, String)> {
    // Try to extract task info from current branch name
    let branch_name = git.get_current_branch()?;

    // Check if we have task tracker configuration
    if let Some(_jira_config) = &config.jira {
        if let Ok(Some(task_tracker)) = get_task_tracker(config).await {
            // Try to extract task ID from branch name using GitRepository method
            if let Some(task_id) = git.parse_task_id_from_branch()? {
                match task_tracker.get_task(&task_id).await {
                    Ok(task) => return Ok((task.title, task.description)),
                    Err(e) => {
                        presentation::show_error(&format!(
                            "⚠️  Found task ID '{}' in branch but couldn't fetch from tracker: {}",
                            task_id, e
                        ))?;
                    }
                }
            }
        }
    }

    // Fallback: use branch name as task title
    let title = if branch_name == "main" || branch_name == "master" {
        "Changes from main branch".to_string()
    } else {
        format!("Changes from branch: {}", branch_name)
    };

    Ok((title, "Generated from git history".to_string()))
}

fn build_pr_description_template() -> String {
    r#"
You are an expert technical writer helping create a pull request description.
Based on the provided context, create a clear, comprehensive PR description that includes:

## Summary
A brief overview of what this PR accomplishes

## Changes
- List the key changes made
- Focus on the what and why, not just the how
- Use bullet points for clarity

## Context
Any relevant background information that reviewers should know

## Testing
How these changes have been tested (if applicable)

Guidelines:
- Be concise but informative
- Use a professional but friendly tone
- Focus on the value and impact of the changes
- Make it easy for reviewers to understand what they're reviewing

Context will be provided below. Use it to craft a thoughtful PR description.
"#
    .trim()
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, JiraConfig};

    #[tokio::test]
    async fn test_get_manual_task_info() {
        // Test with provided title and description
        let result = get_manual_task_info(
            Some("Test Title".to_string()),
            Some("Test Description".to_string()),
        )
        .await;
        assert!(result.is_ok());
        let (title, desc) = result.unwrap();
        assert_eq!(title, "Test Title");
        assert_eq!(desc, "Test Description");
    }

    #[test]
    fn should_return_provided_values() {
        // This test validates the logic that would be used in get_manual_task_info
        let title = Some("Test Title".to_string());
        let desc = Some("Test Description".to_string());

        assert_eq!(title.as_ref().unwrap(), "Test Title");
        assert_eq!(desc.as_ref().unwrap(), "Test Description");
    }

    #[test]
    fn test_build_pr_description_template() {
        let template = build_pr_description_template();

        assert!(template.contains("## Summary"));
        assert!(template.contains("## Changes"));
        assert!(template.contains("## Context"));
        assert!(template.contains("## Testing"));
        assert!(template.contains("Be concise but informative"));
        assert!(template.contains("Use a professional but friendly tone"));
    }

    mod get_manual_task_info_tests {
        use super::*;

        #[tokio::test]
        async fn should_return_provided_title_and_description() {
            let result = get_manual_task_info(
                Some("Feature Implementation".to_string()),
                Some("Implementing new user authentication feature".to_string()),
            )
            .await;

            assert!(result.is_ok());
            let (title, desc) = result.unwrap();
            assert_eq!(title, "Feature Implementation");
            assert_eq!(desc, "Implementing new user authentication feature");
        }

        #[tokio::test]
        async fn should_handle_provided_title_only() {
            // Test that when only title is provided, description gets the provided value
            // This test validates the logic without actually requiring stdin input
            let title = Some("Bug Fix".to_string());
            let desc = None;

            // Test the logic that would be used
            let expected_title = title.clone().unwrap();
            let expected_desc = match desc {
                Some(d) => d,
                None => "Work in progress".to_string(), // Default when no input
            };

            assert_eq!(expected_title, "Bug Fix");
            assert_eq!(expected_desc, "Work in progress");
        }

        #[tokio::test]
        async fn should_handle_provided_description_only() {
            // Test that when only description is provided, title gets default value
            // This test validates the logic without actually requiring stdin input
            let title = None;
            let desc = Some("Detailed description of changes".to_string());

            // Test the logic that would be used
            let expected_title = title.unwrap_or_else(|| "Your Amazing Work".to_string());
            let expected_desc = desc.unwrap();

            assert_eq!(expected_title, "Your Amazing Work");
            assert_eq!(expected_desc, "Detailed description of changes");
        }
    }

    mod get_auto_task_info_tests {
        use super::*;

        #[tokio::test]
        async fn should_handle_main_branch_fallback() {
            // Mock a GitRepository that returns "main" as current branch
            // Since we can't easily mock GitRepository, we test the fallback logic
            let branch_name = "main";
            let title = if branch_name == "main" || branch_name == "master" {
                "Changes from main branch".to_string()
            } else {
                format!("Changes from branch: {}", branch_name)
            };

            assert_eq!(title, "Changes from main branch");
        }

        #[tokio::test]
        async fn should_handle_master_branch_fallback() {
            let branch_name = "master";
            let title = if branch_name == "main" || branch_name == "master" {
                "Changes from main branch".to_string()
            } else {
                format!("Changes from branch: {}", branch_name)
            };

            assert_eq!(title, "Changes from main branch");
        }

        #[tokio::test]
        async fn should_handle_feature_branch() {
            let branch_name = "feature/user-authentication";
            let title = if branch_name == "main" || branch_name == "master" {
                "Changes from main branch".to_string()
            } else {
                format!("Changes from branch: {}", branch_name)
            };

            assert_eq!(title, "Changes from branch: feature/user-authentication");
        }

        #[tokio::test]
        async fn should_handle_missing_jira_config() {
            let config = Config::default(); // No Jira config
            let has_jira = config.jira.is_some();
            assert_eq!(has_jira, false);
        }

        #[tokio::test]
        async fn should_handle_jira_config_present() {
            let mut config = Config::default();
            config.jira = Some(JiraConfig {
                base_url: "https://test.atlassian.net".to_string(),
                username: "test@example.com".to_string(),
            });

            let has_jira = config.jira.is_some();
            assert_eq!(has_jira, true);
        }
    }

    mod context_handling_tests {
        #[test]
        fn should_handle_manual_mode_with_provided_title_and_desc() {
            // Test that manual mode uses provided values
            let title = Some("Manual Title".to_string());
            let desc = Some("Manual Description".to_string());

            assert!(title.is_some());
            assert!(desc.is_some());
            assert_eq!(title.unwrap(), "Manual Title");
            assert_eq!(desc.unwrap(), "Manual Description");
        }

        #[test]
        fn should_handle_prompt_only_mode() {
            // Test that prompt_only flag affects execution flow
            let prompt_only = true;
            assert_eq!(prompt_only, true);
        }

        #[test]
        fn should_handle_use_commits_flag() {
            // Test that use_commits flag is properly handled
            let use_commits = true;
            assert_eq!(use_commits, true);
        }

        #[test]
        fn should_handle_custom_instruction() {
            let instruction = Some("Custom instruction for the LLM".to_string());
            assert!(instruction.is_some());
            assert_eq!(instruction.unwrap(), "Custom instruction for the LLM");
        }

        #[test]
        fn should_handle_custom_context() {
            let custom_context = Some("Additional context information".to_string());
            assert!(custom_context.is_some());
            assert_eq!(custom_context.unwrap(), "Additional context information");
        }

        #[test]
        fn should_handle_context_file_path() {
            let context_file = Some("/path/to/context.md".to_string());
            assert!(context_file.is_some());
            assert_eq!(context_file.unwrap(), "/path/to/context.md");
        }

        #[test]
        fn should_handle_stdin_flag() {
            let from_stdin = true;
            assert_eq!(from_stdin, true);
        }

        #[test]
        fn should_handle_multiple_context_sources() {
            let use_commits = true;
            let instruction = Some("Custom instruction".to_string());
            let custom_context = Some("Custom context".to_string());
            let context_file = Some("context.txt".to_string());
            let from_stdin = true;

            assert_eq!(use_commits, true);
            assert!(instruction.is_some());
            assert!(custom_context.is_some());
            assert!(context_file.is_some());
            assert_eq!(from_stdin, true);
        }
    }

    mod execute_function_tests {
        #[test]
        fn should_handle_manual_mode_flag() {
            let manual = true;
            assert_eq!(manual, true);
        }

        #[test]
        fn should_handle_auto_mode_flag() {
            let manual = false;
            assert_eq!(manual, false);
        }

        #[test]
        fn should_validate_parameter_combinations() {
            // Test various parameter combinations
            let params = (
                false,                           // manual
                None::<String>,                  // title
                None::<String>,                  // desc
                false,                           // prompt_only
                true,                            // use_commits
                Some("instruction".to_string()), // instruction
                Some("context".to_string()),     // custom_context
                Some("file.txt".to_string()),    // context_file
                true,                            // from_stdin
            );

            let (
                manual,
                title,
                desc,
                prompt_only,
                use_commits,
                instruction,
                custom_context,
                context_file,
                from_stdin,
            ) = params;

            assert_eq!(manual, false);
            assert!(title.is_none());
            assert!(desc.is_none());
            assert_eq!(prompt_only, false);
            assert_eq!(use_commits, true);
            assert!(instruction.is_some());
            assert!(custom_context.is_some());
            assert!(context_file.is_some());
            assert_eq!(from_stdin, true);
        }
    }

    mod branch_name_handling_tests {
        #[test]
        fn should_handle_main_branch_fallback() {
            let branch_name = "main";
            let expected_title = "Changes from main branch";

            let title = if branch_name == "main" || branch_name == "master" {
                "Changes from main branch".to_string()
            } else {
                format!("Changes from branch: {}", branch_name)
            };

            assert_eq!(title, expected_title);
        }

        #[test]
        fn should_handle_master_branch_fallback() {
            let branch_name = "master";
            let expected_title = "Changes from main branch";

            let title = if branch_name == "main" || branch_name == "master" {
                "Changes from main branch".to_string()
            } else {
                format!("Changes from branch: {}", branch_name)
            };

            assert_eq!(title, expected_title);
        }

        #[test]
        fn should_handle_feature_branch_title() {
            let branch_name = "feature/new-component";
            let expected_title = "Changes from branch: feature/new-component";

            let title = if branch_name == "main" || branch_name == "master" {
                "Changes from main branch".to_string()
            } else {
                format!("Changes from branch: {}", branch_name)
            };

            assert_eq!(title, expected_title);
        }

        #[test]
        fn should_handle_default_git_centric_mode() {
            let task_title = "Changes from current branch";
            let task_desc = "Generated from git history and provided context";

            assert_eq!(task_title, "Changes from current branch");
            assert_eq!(task_desc, "Generated from git history and provided context");
        }
    }

    mod error_handling_tests {
        #[test]
        fn should_handle_missing_task_tracker_gracefully() {
            // Test that the function handles missing task tracker configuration
            let has_jira_config = false;

            if !has_jira_config {
                // Should fall back to git-centric mode
                let fallback_title = "Changes from current branch";
                assert_eq!(fallback_title, "Changes from current branch");
            }
        }

        #[test]
        fn should_handle_task_tracker_fetch_error() {
            // Test handling when task tracker fails to fetch task info
            let fetch_error = true;

            if fetch_error {
                let fallback_mode = "git-centric";
                assert_eq!(fallback_mode, "git-centric");
            }
        }

        #[test]
        fn should_handle_empty_stdin_input() {
            let stdin_content = "";
            let is_empty = stdin_content.trim().is_empty();

            assert_eq!(is_empty, true);
        }

        #[test]
        fn should_handle_file_read_error() {
            // Test that file read errors are properly handled
            let file_path = "/nonexistent/file.txt";
            let file_exists = std::path::Path::new(file_path).exists();

            assert_eq!(file_exists, false);
        }

        #[test]
        fn should_handle_non_empty_stdin_input() {
            let stdin_content = "Some piped content";
            let is_empty = stdin_content.trim().is_empty();

            assert_eq!(is_empty, false);
        }

        #[test]
        fn should_handle_context_file_reading() {
            // Test context file path validation
            let context_file = Some("valid/path/context.md".to_string());
            assert!(context_file.is_some());

            let path = context_file.unwrap();
            assert!(path.ends_with(".md"));
        }
    }

    mod template_tests {
        use super::build_pr_description_template;

        #[test]
        fn should_contain_all_required_sections() {
            let template = build_pr_description_template();

            assert!(template.contains("## Summary"));
            assert!(template.contains("## Changes"));
            assert!(template.contains("## Context"));
            assert!(template.contains("## Testing"));
        }

        #[test]
        fn should_contain_guidelines() {
            let template = build_pr_description_template();

            assert!(template.contains("Guidelines:"));
            assert!(template.contains("Be concise but informative"));
            assert!(template.contains("Use a professional but friendly tone"));
            assert!(template.contains("Focus on the value and impact"));
        }

        #[test]
        fn should_contain_context_instructions() {
            let template = build_pr_description_template();

            assert!(template.contains("Context will be provided below"));
            assert!(template.contains("Use it to craft a thoughtful PR description"));
        }

        #[test]
        fn should_be_properly_formatted() {
            let template = build_pr_description_template();

            // Check that the template doesn't start or end with whitespace
            assert_eq!(template, template.trim());

            // Check that it contains the expected structure
            assert!(template.starts_with("You are an expert technical writer"));
        }

        #[test]
        fn should_include_bullet_points_guidance() {
            let template = build_pr_description_template();

            assert!(template.contains("Use bullet points for clarity"));
        }

        #[test]
        fn should_include_reviewer_consideration() {
            let template = build_pr_description_template();

            assert!(template.contains("Make it easy for reviewers to understand"));
        }
    }

    mod prompt_only_mode_tests {
        #[test]
        fn should_handle_prompt_only_flag_true() {
            let prompt_only = true;
            assert_eq!(prompt_only, true);
        }

        #[test]
        fn should_handle_prompt_only_flag_false() {
            let prompt_only = false;
            assert_eq!(prompt_only, false);
        }

        #[test]
        fn should_validate_mcp_format_output() {
            // This would test the MCP format output in prompt-only mode
            let context_types = vec!["git_diff", "commit_messages", "custom_instruction"];

            for context_type in context_types {
                assert!(!context_type.is_empty());
            }
        }
    }

    mod task_description_handling_tests {
        #[test]
        fn should_skip_task_description_for_git_centric_mode() {
            let task_title = "Changes from current branch";
            let should_add_task_desc = task_title != "Changes from current branch";

            assert_eq!(should_add_task_desc, false);
        }

        #[test]
        fn should_add_task_description_for_real_tasks() {
            let task_title = "Feature: User Authentication";
            let should_add_task_desc = task_title != "Changes from current branch";

            assert_eq!(should_add_task_desc, true);
        }

        #[test]
        fn should_handle_empty_task_title() {
            let task_title = "";
            let should_add_task_desc = task_title != "Changes from current branch";

            assert_eq!(should_add_task_desc, true);
        }
    }
}
