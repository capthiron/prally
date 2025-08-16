use git2::{Repository, DiffOptions};
use crate::error::{PrallyError, Result};

pub struct GitRepository {
    repo: Repository,
}

impl GitRepository {
    pub fn open() -> Result<Self> {
        let repo = Repository::open_from_env()?;
        Ok(Self { repo })
    }

    pub fn get_current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        let branch_name = head
            .shorthand()
            .ok_or_else(|| PrallyError::Git(git2::Error::from_str("Unable to get branch name")))?;
        Ok(branch_name.to_string())
    }

    pub fn get_diff(&self) -> Result<String> {

        if self.repo.head().is_err() {
            return Ok("No commits yet - this is a new branch".to_string());
        }

        let head_tree = self.repo.head()?.peel_to_tree()?;
        let mut diff_options = DiffOptions::new();
        diff_options.context_lines(3);

        let diff = self.repo.diff_tree_to_workdir_with_index(Some(&head_tree), Some(&mut diff_options))?;

        let mut diff_text = String::new();
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            match line.origin() {
                '+' | '-' | ' ' => diff_text.push(line.origin()),
                _ => {}
            }
            diff_text.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
            true
        })?;

        Ok(diff_text)
    }

     #[allow(dead_code)]
    pub fn get_remote_url(&self) -> Result<String> {
        let remote = self.repo.find_remote("origin")?;
        let url = remote.url()
            .ok_or_else(|| PrallyError::Git(git2::Error::from_str("No remote URL found")))?;
        Ok(url.to_string())
    }

    pub fn parse_task_id_from_branch(&self) -> Result<Option<String>> {
        let branch = self.get_current_branch()?;

        // Common patterns for task IDs in branch names
        // Examples: feature/ABC-123, ABC-123-description, ABC-123
        let patterns = [
            regex::Regex::new(r"([A-Z]+-\d+)").unwrap(),
            regex::Regex::new(r"#(\d+)").unwrap(),
        ];

        for pattern in &patterns {
            if let Some(captures) = pattern.captures(&branch) {
                if let Some(task_id) = captures.get(1) {
                    return Ok(Some(task_id.as_str().to_string()));
                }
            }
        }

        Ok(None)
    }

    pub fn get_commit_messages_from_branch(&self) -> Result<Vec<String>> {
        let head = self.repo.head()?;
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;

        // Try to find the merge base with the default branch (main/master)
        let default_branches = ["main", "master", "develop"];
        let mut merge_base_oid = None;

        for branch_name in &default_branches {
            if let Ok(branch_ref) = self.repo.find_reference(&format!("refs/heads/{}", branch_name))
                .or_else(|_| self.repo.find_reference(&format!("refs/remotes/origin/{}", branch_name))) {
                if let Ok(target_oid) = branch_ref.target().ok_or_else(|| git2::Error::from_str("No target OID")) {
                    if let Ok(base) = self.repo.merge_base(head.target().unwrap(), target_oid) {
                        merge_base_oid = Some(base);
                        break;
                    }
                }
            }
        }

        let mut commit_messages = Vec::new();

        for oid in revwalk {
            let oid = oid?;

            // Stop if we've reached the merge base
            if let Some(base) = merge_base_oid {
                if oid == base {
                    break;
                }
            }

            let commit = self.repo.find_commit(oid)?;
            let message = commit.message().unwrap_or("").trim().to_string();

            // Skip empty messages and merge commits
            if !message.is_empty() && !message.starts_with("Merge") {
                commit_messages.push(message);
            }

            // Limit to prevent excessive output
            if commit_messages.len() >= 20 {
                break;
            }
        }

        // Reverse to get chronological order (oldest first)
        commit_messages.reverse();
        Ok(commit_messages)
    }
}
