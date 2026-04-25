//! Handles `Commands::Target` — target a specific repository.

use colored::Colorize;

use crate::cli::{
    create_github, create_llm, create_memory, load_config, parse_github_url, print_banner,
    print_result, print_result_ext,
};

pub async fn run_target(
    config_path: Option<&str>,
    url: String,
    dry_run: bool,
    self_mode: bool,
) -> anyhow::Result<()> {
    print_banner();
    let config = load_config(config_path)?;

    let mode_label = if self_mode {
        "(SELF — push to private repo)".magenta().to_string()
    } else if dry_run {
        "(DRY RUN)".yellow().to_string()
    } else {
        "(LIVE)".green().to_string()
    };

    println!("🎯 Targeting: {} {}", url.cyan().bold(), mode_label);
    println!();

    let (owner, name) = parse_github_url(&url)?;

    let github = create_github(&config)?;
    let llm = create_llm(&config)?;
    let memory = create_memory(&config)?;
    let event_bus = contribai::core::events::EventBus::default();

    let pipeline = contribai::orchestrator::pipeline::ContribPipeline::new(
        &config,
        &github,
        llm.as_ref(),
        &memory,
        &event_bus,
    );

    if self_mode {
        let result = pipeline.run_targeted_self(&owner, &name, dry_run).await?;
        print_result_ext(&result, dry_run, true);
    } else {
        let result = pipeline.run_targeted(&owner, &name, dry_run).await?;
        print_result(&result, dry_run);
    }
    Ok(())
}
