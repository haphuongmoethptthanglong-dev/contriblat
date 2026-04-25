//! Handles `Commands::Update` — auto-update ContribAI to latest version.

use colored::Colorize;
use std::process::Command;

use crate::cli::print_banner;

const INSTALL_URL: &str =
    "https://raw.githubusercontent.com/haphuongmoethptthanglong-dev/contriblat/main/install.sh";

pub async fn run_update(yes: bool) -> anyhow::Result<()> {
    print_banner();
    println!(
        "{}",
        "🔄 Update — Fetching latest version from remote 'my'"
            .cyan()
            .bold()
    );
    println!("{}", "━".repeat(60).dimmed());
    println!("  Current version: {}", contribai::VERSION.cyan());
    println!("  Source:          {}", INSTALL_URL.dimmed());
    println!();

    if !yes {
        let proceed = dialoguer::Confirm::new()
            .with_prompt("  Run installer now?")
            .default(true)
            .interact()?;
        if !proceed {
            println!("  {}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    if cfg!(target_os = "windows") {
        anyhow::bail!(
            "Auto-update via shell installer is not supported on Windows. \
             Please run the install script manually in Git Bash / WSL."
        );
    }

    // curl -fsSL <url> | bash
    let status = Command::new("bash")
        .arg("-c")
        .arg(format!("curl -fsSL {} | bash", INSTALL_URL))
        .status()?;

    if !status.success() {
        anyhow::bail!("Update failed (installer exited with status {})", status);
    }

    println!();
    println!(
        "  {} ContribAI updated. Run `contribai version` to confirm.",
        "✅".green()
    );
    Ok(())
}
