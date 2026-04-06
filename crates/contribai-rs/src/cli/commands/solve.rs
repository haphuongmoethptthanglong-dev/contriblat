//! Handles `Commands::Solve` — solve open issues in a repository.

use colored::Colorize;

use crate::cli::{
    create_github, create_llm, create_memory, load_config, parse_github_url, print_banner,
};

pub async fn run_solve(
    config_path: Option<&str>,
    url: String,
    dry_run: bool,
) -> anyhow::Result<()> {
    print_banner();
    let config = load_config(config_path)?;

    println!(
        "🧩 Solving issues in: {} {}",
        url.cyan().bold(),
        if dry_run {
            "(DRY RUN)".yellow().to_string()
        } else {
            "(LIVE)".green().to_string()
        }
    );
    println!();

    let (owner, name) = parse_github_url(&url)?;
    let full_name = format!("{}/{}", owner, name);

    let github = create_github(&config)?;
    let llm = create_llm(&config)?;

    let repo = contribai::core::models::Repository {
        owner: owner.clone(),
        name: name.clone(),
        full_name: full_name.clone(),
        description: None,
        language: None,
        languages: std::collections::HashMap::new(),
        stars: 0,
        forks: 0,
        open_issues: 0,
        default_branch: "main".to_string(),
        topics: vec![],
        html_url: url.clone(),
        clone_url: format!("https://github.com/{}.git", full_name),
        has_contributing: false,
        has_license: false,
        last_push_at: None,
        created_at: None,
    };

    let solver = contribai::issues::solver::IssueSolver::new(llm.as_ref(), &github);
    let issues = solver.fetch_solvable_issues(&repo, 10, 3).await;

    if issues.is_empty() {
        println!(
            "  {} No solvable issues found in {}",
            "⚠️".bold(),
            full_name.cyan()
        );
        return Ok(());
    }

    println!(
        "  {} Found {} solvable issue(s):\n",
        "📋".bold(),
        issues.len().to_string().cyan()
    );
    println!(
        "  {:>6}  {:<45}  {:<12}  {}",
        "Issue#".dimmed(),
        "Title".dimmed(),
        "Category".dimmed(),
        "URL".dimmed()
    );
    println!("  {}", "─".repeat(80).dimmed());

    for issue in &issues {
        let category = solver.classify_issue(issue);
        let cat_str = format!("{:?}", category);
        let title: String = issue.title.chars().take(43).collect();
        println!(
            "  {:>6}  {:<45}  {:<12}  {}",
            format!("#{}", issue.number).cyan(),
            title,
            cat_str.yellow(),
            issue.html_url.dimmed(),
        );
    }

    // v5.5: Actually solve issues and create PRs
    println!();

    let memory = create_memory(&config)?;
    let file_tree = github
        .get_file_tree(&owner, &name, None)
        .await
        .unwrap_or_default();

    let repo_context = contribai::core::models::RepoContext {
        repo: repo.clone(),
        file_tree,
        readme_content: None,
        contributing_guide: None,
        relevant_files: std::collections::HashMap::new(),
        open_issues: Vec::new(),
        coding_style: None,
        symbol_map: std::collections::HashMap::new(),
        resolved_imports: std::collections::HashMap::new(),
        file_ranks: std::collections::HashMap::new(),
    };

    let generator = contribai::generator::engine::ContributionGenerator::new(
        llm.as_ref(),
        &config.contribution,
    );

    let mut prs_created = 0u32;
    for issue in &issues {
        println!(
            "  {} Solving issue #{}...",
            "🔧".bold(),
            issue.number.to_string().cyan()
        );

        // Solve: issue → findings
        let findings = solver.solve_issue_deep(issue, &repo, &repo_context).await;
        if findings.is_empty() {
            println!("    {} No actionable findings", "⚠️".dimmed());
            continue;
        }

        // Fetch file contents for identified files
        let mut ctx = repo_context.clone();
        for f in &findings {
            if !f.file_path.is_empty() && !ctx.relevant_files.contains_key(&f.file_path) {
                if let Ok(content) = github
                    .get_file_content(&owner, &name, &f.file_path, None)
                    .await
                {
                    ctx.relevant_files.insert(f.file_path.clone(), content);
                }
            }
        }

        // Generate code for each finding
        let mut valid = Vec::new();
        for finding in &findings {
            if let Ok(Some(mut contrib)) = generator.generate(finding, &ctx).await {
                contrib.description = format!("Fixes #{}\n\n{}", issue.number, contrib.description);
                valid.push(contrib);
            }
        }

        if valid.is_empty() {
            println!("    {} Generation failed", "❌".dimmed());
            continue;
        }

        // Merge into single PR
        let file_count = valid.iter().map(|c| c.changes.len()).sum::<usize>();
        let mut merged = contribai::orchestrator::pipeline::merge_contributions_pub(valid);
        merged.title = format!("fix: resolve #{} — {}", issue.number, issue.title);
        merged.commit_message = format!(
            "fix: resolve #{} — {}\n\nFixes #{}",
            issue.number, issue.title, issue.number
        );

        if dry_run {
            println!(
                "    {} Would create PR ({} files)",
                "[DRY RUN]".yellow(),
                file_count
            );
            continue;
        }

        let mut pr_mgr = contribai::pr::manager::PrManager::new(&github);
        match pr_mgr.create_pr(&merged, &repo).await {
            Ok(pr_result) => {
                prs_created += 1;
                let _ = memory.record_pr(
                    &full_name,
                    pr_result.pr_number,
                    &pr_result.pr_url,
                    &merged.title,
                    &merged.contribution_type.to_string(),
                    &pr_result.branch_name,
                    &pr_result.fork_full_name,
                );
                println!(
                    "    {} PR #{} created → {}",
                    "✅".bold(),
                    pr_result.pr_number.to_string().green(),
                    pr_result.pr_url.dimmed()
                );
            }
            Err(e) => {
                println!("    {} PR failed: {}", "❌".bold(), format!("{}", e).red());
            }
        }
    }

    println!();
    if prs_created > 0 {
        println!(
            "  {} {} PR(s) created from {} issues",
            "🎉".bold(),
            prs_created.to_string().green(),
            issues.len()
        );
    } else if dry_run {
        println!("  {} Dry run — no PRs submitted.", "[DRY RUN]".yellow());
    } else {
        println!("  {} No PRs could be generated.", "⚠️".bold());
    }
    Ok(())
}
