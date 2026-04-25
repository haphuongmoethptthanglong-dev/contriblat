//! Self-mode handler for `hunt --self`.
//!
//! Instead of creating PRs on discovered repos, self-mode:
//! 1. Clones the target repo locally (if not already present).
//! 2. Creates a private repo on the user's GitHub (if not already present).
//! 3. Checks out a date-prefixed branch (DD-MM-YYYY).
//! 4. Applies each fix as a separate commit and pushes to the private remote.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use chrono::Utc;
use tracing::{info, warn};

use crate::core::models::Contribution;
use crate::github::client::GitHubClient;

/// Handles the clone → private-remote → branch → commit → push flow.
pub struct SelfModeHandler<'a> {
    github: &'a GitHubClient,
    token: String,
    username: String,
}

impl<'a> SelfModeHandler<'a> {
    /// Create a new handler. Fetches the authenticated username eagerly.
    pub async fn new(github: &'a GitHubClient, token: &str) -> Result<SelfModeHandler<'a>> {
        let user = github
            .get_authenticated_user()
            .await
            .context("Failed to get authenticated GitHub user")?;
        let username = user["login"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Could not determine GitHub username"))?
            .to_string();

        Ok(Self {
            github,
            token: token.to_string(),
            username,
        })
    }

    /// Authenticated username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Clone the target repo into the current working directory if the directory
    /// does not already exist. Returns the local path.
    pub fn clone_repo_locally(&self, owner: &str, repo_name: &str) -> Result<PathBuf> {
        let target_dir = PathBuf::from(repo_name);

        if target_dir.exists() {
            info!(
                repo = %format!("{}/{}", owner, repo_name),
                path = %target_dir.display(),
                "📂 Local clone already exists — reusing"
            );
            return Ok(target_dir);
        }

        let url = format!("https://github.com/{}/{}.git", owner, repo_name);
        info!(
            repo = %format!("{}/{}", owner, repo_name),
            "📥 Cloning to {}",
            target_dir.display()
        );

        let output = Command::new("git")
            .args(["clone", &url, repo_name])
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .output()
            .context("Failed to execute git clone")?;

        if !output.status.success() {
            bail!("git clone failed with exit code: {}", output.status);
        }

        Ok(target_dir)
    }

    /// Ensure a private repo exists on the user's GitHub account and configure
    /// a `self-origin` remote in the local clone pointing to it.
    pub async fn ensure_private_remote(
        &self,
        local_path: &Path,
        repo_name: &str,
    ) -> Result<()> {
        let exists = self.github.check_repo_exists(&self.username, repo_name).await;

        if exists {
            info!(
                repo = %format!("{}/{}", self.username, repo_name),
                "📦 Private repo already exists on GitHub"
            );
        } else {
            info!(
                repo = %format!("{}/{}", self.username, repo_name),
                "📦 Creating private repo on GitHub"
            );
            self.github
                .create_user_repo(repo_name, true)
                .await
                .context("Failed to create private repo on GitHub")?;
        }

        // Configure self-origin remote (token-embedded for push auth)
        let auth_url = format!(
            "https://x-access-token:{}@github.com/{}/{}.git",
            self.token, self.username, repo_name
        );

        // Remove existing self-origin if present (ignore error if it doesn't exist)
        let _ = Command::new("git")
            .args(["remote", "remove", "self-origin"])
            .current_dir(local_path)
            .output();

        let add = Command::new("git")
            .args(["remote", "add", "self-origin", &auth_url])
            .current_dir(local_path)
            .output()
            .context("Failed to add self-origin remote")?;

        if !add.status.success() {
            let err = String::from_utf8_lossy(&add.stderr);
            bail!("Failed to add self-origin remote: {}", err.trim());
        }

        // Push the default branch first so the remote is initialized
        if !exists {
            info!("🚀 Pushing default branch to initialize remote...");
            let push = Command::new("git")
                .args(["push", "-u", "self-origin", "--all"])
                .current_dir(local_path)
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .output()
                .context("Failed to push default branch")?;

            if !push.status.success() {
                warn!("Initial push to self-origin failed (non-fatal)");
            }
        }

        Ok(())
    }

    /// Create and checkout a date-prefixed branch (DD-MM-YYYY).
    /// If the branch already exists, appends `-2`, `-3`, etc.
    pub fn checkout_date_branch(&self, local_path: &Path) -> Result<String> {
        let date = Utc::now().format("%d-%m-%Y").to_string();
        let mut branch_name = date.clone();
        let mut suffix = 1u32;

        loop {
            let check = Command::new("git")
                .args(["rev-parse", "--verify", &format!("refs/heads/{}", branch_name)])
                .current_dir(local_path)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .context("Failed to check branch existence")?;

            if !check.success() {
                // Branch doesn't exist — use this name
                break;
            }

            suffix += 1;
            branch_name = format!("{}-{}", date, suffix);
        }

        info!(branch = %branch_name, "🌿 Creating branch");

        let checkout = Command::new("git")
            .args(["checkout", "-b", &branch_name])
            .current_dir(local_path)
            .output()
            .context("Failed to create branch")?;

        if !checkout.status.success() {
            let err = String::from_utf8_lossy(&checkout.stderr);
            bail!("Failed to checkout branch {}: {}", branch_name, err.trim());
        }

        Ok(branch_name)
    }

    /// Apply a contribution's file changes, commit each change separately,
    /// and push to `self-origin`.
    pub fn commit_and_push_contribution(
        &self,
        local_path: &Path,
        contribution: &Contribution,
        branch_name: &str,
    ) -> Result<usize> {
        let mut commits = 0usize;

        for change in &contribution.changes {
            let file_path = local_path.join(&change.path);

            // Ensure parent directory exists
            if let Some(parent) = file_path.parent() {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create dir for {}", change.path))?;
            }

            if change.is_deleted {
                if file_path.exists() {
                    std::fs::remove_file(&file_path)
                        .with_context(|| format!("Failed to delete {}", change.path))?;
                }
            } else {
                std::fs::write(&file_path, &change.new_content)
                    .with_context(|| format!("Failed to write {}", change.path))?;
            }

            // Stage the file
            let add = Command::new("git")
                .args(["add", &change.path])
                .current_dir(local_path)
                .output()
                .context("git add failed")?;

            if !add.status.success() {
                warn!(file = %change.path, "git add failed — skipping");
                continue;
            }

            // Commit with a descriptive message
            let msg = if contribution.changes.len() == 1 {
                contribution.commit_message.clone()
            } else {
                format!("{}: {}", contribution.title, change.path)
            };

            let commit = Command::new("git")
                .args(["commit", "-m", &msg])
                .current_dir(local_path)
                .output()
                .context("git commit failed")?;

            if !commit.status.success() {
                let err = String::from_utf8_lossy(&commit.stderr);
                // "nothing to commit" is not an error
                if err.contains("nothing to commit") {
                    info!(file = %change.path, "No changes to commit");
                    continue;
                }
                warn!(file = %change.path, err = %err.trim(), "git commit failed");
                continue;
            }

            commits += 1;
            info!(file = %change.path, "✅ Committed");
        }

        if commits > 0 {
            info!(commits, branch = %branch_name, "🚀 Pushing to self-origin");

            let push = Command::new("git")
                .args(["push", "self-origin", branch_name])
                .current_dir(local_path)
                .stdout(std::process::Stdio::inherit())
                .stderr(std::process::Stdio::inherit())
                .output()
                .context("git push failed")?;

            if !push.status.success() {
                bail!("Failed to push branch {} to self-origin", branch_name);
            }
        }

        // Clean up: replace token URL with clean URL for safety
        let clean_url = format!(
            "https://github.com/{}/{}.git",
            self.username,
            local_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("repo")
        );
        let _ = Command::new("git")
            .args(["remote", "set-url", "self-origin", &clean_url])
            .current_dir(local_path)
            .output();

        Ok(commits)
    }
}
