//! Handles `Commands::Hunt` — aggressive multi-round discovery mode.

use colored::Colorize;

use crate::cli::{
    create_github, create_llm, create_memory, load_config, print_banner, print_config_summary,
    print_result, print_result_ext,
};

pub async fn run_hunt(
    config_path: Option<&str>,
    rounds: u32,
    delay: u32,
    language: Option<String>,
    dry_run: bool,
    approve: bool,
    self_mode: bool,
) -> anyhow::Result<()> {
    print_banner();
    let config = load_config(config_path)?;
    print_config_summary(&config, dry_run);

    if self_mode {
        println!(
            "   {}: {}",
            "Self mode".yellow().bold(),
            "clone & push to private repos (no PRs)".dimmed()
        );
    }
    println!(
        "   {}: {} rounds",
        "Hunt mode".yellow().bold(),
        rounds.to_string().cyan()
    );
    if let Some(lang) = &language {
        println!("   {}: {}", "Language".dimmed(), lang.cyan());
    }
    if approve {
        println!(
            "   {}: {}",
            "Approve".dimmed(),
            "HIGH risk enabled".yellow()
        );
    }
    println!();

    let github = create_github(&config)?;
    let llm = create_llm(&config)?;
    let memory = create_memory(&config)?;
    let event_bus = contribai::core::events::EventBus::default();

    // ── v5.4: JSONL event logger ─────────────────────────────────
    let log_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".contribai")
        .join("events.jsonl");
    let _log_handle =
        contribai::core::events::FileEventLogger::new(&log_path).spawn_logger(&event_bus);

    let mut pipeline = contribai::orchestrator::pipeline::ContribPipeline::new(
        &config,
        &github,
        llm.as_ref(),
        &memory,
        &event_bus,
    );
    pipeline.set_approve_high_risk(approve);

    if self_mode {
        let total = pipeline.hunt_self(rounds, delay as u64, dry_run).await?;
        print_result_ext(&total, dry_run, true);
    } else {
        let total = pipeline.hunt(rounds, delay as u64, dry_run, "both").await?;
        print_result(&total, dry_run);
    }

    Ok(())
}
