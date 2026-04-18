//! Handles `Commands::Clone` — clone a target repository to local machine.
//! With `--fork`, also creates a new repo on the user's GitHub and pushes to it.

use colored::Colorize;

use crate::cli::print_banner;

pub async fn run_clone(url: &str, path: Option<&str>, fork: bool) -> anyhow::Result<()> {
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
    println!("  {} git clone {} {}", "Running:".dimmed(), repo_url, target_dir);
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

    // Step 2: Get authenticated GitHub username
    println!();
    println!("  {} Fetching GitHub user info...", "🔑".dimmed());

    let user_output = std::process::Command::new("gh")
        .args(["api", "/user", "--jq", ".login"])
        .output()?;

    if !user_output.status.success() {
        let err = String::from_utf8_lossy(&user_output.stderr);
        anyhow::bail!(
            "Failed to get GitHub user. Is `gh` CLI installed and authenticated?\n  {}",
            err.trim()
        );
    }

    let username = String::from_utf8_lossy(&user_output.stdout)
        .trim()
        .to_string();
    if username.is_empty() {
        anyhow::bail!("Could not determine GitHub username from `gh api /user`");
    }

    println!("  {} {}", "GitHub user:".dimmed(), username.cyan());

    // Step 3: Create new repo on user's account
    println!(
        "  {} Creating repo {}/{}...",
        "📦".dimmed(),
        username,
        repo_name
    );

    let create_output = std::process::Command::new("gh")
        .args(["repo", "create", repo_name, "--private", "--confirm"])
        .output()?;

    if !create_output.status.success() {
        let err = String::from_utf8_lossy(&create_output.stderr);
        anyhow::bail!("Failed to create repo on GitHub: {}", err.trim());
    }

    println!(
        "  {} Repo {}/{} created",
        "✅".green(),
        username,
        repo_name
    );

    // Step 4: Set user's repo as origin, keep source as upstream
    let new_origin = format!("https://github.com/{}/{}.git", username, repo_name);

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

    // Add user's repo as new origin
    let add = std::process::Command::new("git")
        .args(["remote", "add", "origin", &new_origin])
        .current_dir(&target_dir)
        .output()?;

    if !add.status.success() {
        let err = String::from_utf8_lossy(&add.stderr);
        anyhow::bail!("Failed to add new origin: {}", err.trim());
    }

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
        new_origin
    );

    // Step 5: Push all branches and tags to new origin
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
        println!(
            "  {} Failed to push tags (non-fatal)",
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
