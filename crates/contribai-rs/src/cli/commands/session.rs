//! Handles `Commands::Session` — manage pipeline sessions.

use crate::cli::{load_config, print_banner};
use colored::Colorize;
use console::style;

pub fn run_session_list(config_path: Option<&str>) -> anyhow::Result<()> {
    print_banner();
    let _config = load_config(config_path)?;

    println!("{}", style("📋 Active Sessions").cyan().bold());
    println!("{}", "━".repeat(50).dimmed());
    println!();

    // Sessions are in-memory, so we just show a placeholder
    // In a full implementation, this would query a session store
    println!("  {} No persistent session store configured", style("⚪").dimmed());
    println!("  {} Sessions are per-run (stateless)", style("💡").bold());
    println!();
    Ok(())
}

pub fn run_session_create(_name: String, _mode: String) -> anyhow::Result<()> {
    print_banner();
    println!("{}", style("📋 Session Created").green().bold());
    println!();
    // Session management is in-memory for now
    println!("  {} Session management is in-memory (per-run)", style("ℹ️").dimmed());
    println!();
    Ok(())
}
