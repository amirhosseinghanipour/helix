use crate::core::repository::Repository;
use anyhow::Result;
use chrono::Duration;
use colored::*;

pub async fn list_branches(repo: &Repository) -> Result<()> {
    println!("{}", "Branches".bold().blue());
    println!("{}", "=".repeat(40).blue());

    for (name, branch) in &repo.branches {
        let indicator = if name == &repo.current_branch {
            "* "
        } else {
            "  "
        };
        let name_display = if name == &repo.current_branch {
            name.yellow().bold()
        } else {
            name.normal()
        };

        println!("{}{}", indicator, name_display);

        if let Some(head_commit) = branch.get_head_commit() {
            println!("    HEAD: {}", head_commit[..8].cyan());
        }

        if let Some(upstream) = branch.get_upstream() {
            println!("    Upstream: {}", upstream.magenta());
        }

        // Show branch age
        let age = branch.get_age();
        if age > Duration::hours(1) {
            println!("    Age: {} old", format_duration(age));
        }

        // Show if it's the main branch
        if branch.is_main() {
            println!("    🌟 Main branch");
        }

        // Show last update age
        let last_update = branch.get_last_update_age();
        if last_update > Duration::minutes(5) {
            println!("    Last update: {} ago", format_duration(last_update));
        }
    }

    Ok(())
}

fn format_duration(duration: Duration) -> String {
    if duration.num_days() > 0 {
        format!("{} days", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("{} hours", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("{} minutes", duration.num_minutes())
    } else {
        "just now".to_string()
    }
}

pub async fn create_branch(repo: &mut Repository, name: &str) -> Result<()> {
    if repo.branches.contains_key(name) {
        println!("{}", format!("Branch '{}' already exists", name).red());
        return Ok(());
    }

    repo.create_branch(name)?;

    println!("{}", format!("Created branch '{}'", name).green().bold());
    println!("Current branch: {}", repo.current_branch.yellow().bold());

    Ok(())
}
