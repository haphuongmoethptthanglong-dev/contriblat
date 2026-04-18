//! Handles `Commands::Clone` — clone a target repository to local machine.
//! With `--fork`, also creates a new repo on the user's GitHub and pushes to it.

use colored::Colorize;

use crate::cli::{create_github, load_config, print_banner};

pub async fn run_clone(
    config_path: Option<&str>,
    url: &str,
    path: Option<&str>,
    fork: bool,
) -> anyhow::Result<()> {
    print_banner();

    let title = if fork {
        "📥 Clone + Fork — Clone and push to your remote"
    } else {
        "📥 Clone — Clone target repository"
    };
    println!("{}", title.cyan().bold());
    println!("{}", "━".repeat(60).dimmed());
    println!();

    let repo_url = url.trim().trim_end_matches('/');
    if repo_url.is_empty() {
        anyhow::bail!("Repository URL cannot be empty");
    }

    // Extract repo name from URL
    let repo_name = repo_url
        .rsplit('/')
        .next()
        .unwrap_or("repo")
        .trim_end_matches(".git");

    let target_dir = match path {
        Some(p) => p.to_string(),
        None => repo_name.to_string(),
    };

    println!("  {} {}", "Repository:".dimmed(), repo_url.cyan());
    println!("  {} {}", "Target dir:".dimmed(), target_dir.cyan());
    println!();

    // Step 1: Clone
    println!(
        "  {} git clone {} {}",
        "Running:".dimmed(),
        repo_url,
        target_dir
    );
    println!();

    let output = std::process::Command::new("git")
        .args(["clone", repo_url, &target_dir])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()?;

    if !output.status.success() {
        anyhow::bail!("git clone failed with exit code: {}", output.status);
    }

    println!();
    println!(
        "  {} Repository cloned to {}",
        "✅".green(),
        target_dir.cyan().bold()
    );

    if !fork {
        return Ok(());
    }

    // Step 2: Load config and create GitHub client
    let config = load_config(config_path)?;
    let github = create_github(&config)?;

    // Step 3: Get authenticated GitHub username
    println!();
    println!("  {} Fetching GitHub user info...", "🔑".dimmed());

    let user = github
        .get_authenticated_user()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get GitHub user: {}", e))?;

    let username = user["login"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Could not determine GitHub username from API response"))?;

    println!("  {} {}", "GitHub user:".dimmed(), username.cyan());

    // Step 4: Create new repo on user's account
    println!(
        "  {} Creating repo {}/{}...",
        "📦".dimmed(),
        username,
        repo_name
    );

    github
        .create_user_repo(repo_name, true)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create repo on GitHub: {}", e))?;

    println!(
        "  {} Repo {}/{} created",
        "✅".green(),
        username,
        repo_name
    );

    // Step 5: Set user's repo as origin, keep source as upstream
    // Use token-authenticated URL so push works without gh CLI
    let new_origin = format!(
        "https://x-access-token:{}@github.com/{}/{}.git",
        config.github.token, username, repo_name
    );

    println!("  {} Setting remotes...", "🔗".dimmed());

    // Rename current origin → upstream
    let rename = std::process::Command::new("git")
        .args(["remote", "rename", "origin", "upstream"])
        .current_dir(&target_dir)
        .output()?;

    if !rename.status.success() {
        let err = String::from_utf8_lossy(&rename.stderr);
        anyhow::bail!("Failed to rename origin to upstream: {}", err.trim());
    }

    // Add user's repo as new origin (with embedded token for auth)
    let add = std::process::Command::new("git")
        .args(["remote", "add", "origin", &new_origin])
        .current_dir(&target_dir)
        .output()?;

    if !add.status.success() {
        let err = String::from_utf8_lossy(&add.stderr);
        anyhow::bail!("Failed to add new origin: {}", err.trim());
    }

    // Display clean URLs (without token)
    let display_origin = format!("https://github.com/{}/{}.git", username, repo_name);
    println!(
        "  {} {} → {}",
        "upstream:".dimmed(),
        "upstream".yellow(),
        repo_url
    );
    println!(
        "  {} {} → {}",
        "origin:  ".dimmed(),
        "origin".green(),
        display_origin
    );

    // Step 6: Push all branches and tags to new origin
    println!();
    println!("  {} Pushing to origin...", "🚀".dimmed());

    let push = std::process::Command::new("git")
        .args(["push", "-u", "origin", "--all"])
        .current_dir(&target_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()?;

    if !push.status.success() {
        anyhow::bail!("Failed to push to new origin");
    }

    let push_tags = std::process::Command::new("git")
        .args(["push", "origin", "--tags"])
        .current_dir(&target_dir)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()?;

    if !push_tags.status.success() {
        println!("  {} Failed to push tags (non-fatal)", "⚠️".yellow());
    }

    // Step 7: Replace token URL with clean URL for safety
    let clean_origin = format!("https://github.com/{}/{}.git", username, repo_name);
    let set_url = std::process::Command::new("git")
        .args(["remote", "set-url", "origin", &clean_origin])
        .current_dir(&target_dir)
        .output()?;

    if !set_url.status.success() {
        println!(
            "  {} Could not clean origin URL (non-fatal)",
            "⚠️".yellow()
        );
    }

    println!();
    println!("{}", "━".repeat(60).dimmed());
    println!(
        "  {} Forked to {}/{} — cloned in {}",
        "🎉".green(),
        username.cyan(),
        repo_name.cyan().bold(),
        target_dir.cyan()
    );

    Ok(())
}
