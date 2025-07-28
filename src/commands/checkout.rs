use crate::core::repository::Repository;
use anyhow::Result;
use colored::*;

pub async fn checkout_branch(repo: &mut Repository, branch_name: &str) -> Result<()> {
    if !repo.branches.contains_key(branch_name) {
        println!(
            "{}",
            format!("Branch '{}' does not exist", branch_name).red()
        );
        return Ok(());
    }

    if branch_name == repo.current_branch {
        println!(
            "{}",
            format!("Already on branch '{}'", branch_name).yellow()
        );
        return Ok(());
    }

    repo.checkout_branch(branch_name)?;

    println!(
        "{}",
        format!("Switched to branch '{}'", branch_name)
            .green()
            .bold()
    );
    println!("Current branch: {}", repo.current_branch.yellow().bold());

    if let Some(current_branch) = repo.get_current_branch() {
        if let Some(head_commit) = current_branch.get_head_commit() {
            println!("HEAD: {}", head_commit[..8].cyan());
        }
    }

    Ok(())
}
