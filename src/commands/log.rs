use crate::core::commit::Commit;
use crate::core::repository::Repository;
use anyhow::Result;
use colored::*;
use std::collections::{HashSet, VecDeque};

pub async fn show_log(repo: &Repository, limit: usize) -> Result<()> {
    println!("{}", "📜 Commit History".bold().blue());
    println!("{}", "=".repeat(40).blue());

    if let Some(current_branch) = repo.get_current_branch() {
        if let Some(head_commit) = current_branch.get_head_commit() {
            let mut queue = VecDeque::new();
            let mut visited = HashSet::new();
            queue.push_back((head_commit.clone(), 0));
            let mut commit_count = 0;
            while let Some((commit_id, depth)) = queue.pop_front() {
                if visited.contains(&commit_id) || commit_count >= limit {
                    continue;
                }
                if let Ok(commit_object) =
                    crate::core::object::Object::load(&repo.get_objects_dir(), &commit_id)
                {
                    if let Ok(commit) = Commit::from_object(&commit_object) {
                        let is_head = commit_count == 0;
                        let valid = commit.verify();
                        display_commit_dag(&commit, is_head, depth, valid);
                        for parent in &commit.parent_ids {
                            queue.push_back((parent.clone(), depth + 1));
                        }
                        visited.insert(commit_id);
                        commit_count += 1;
                    }
                }
            }
        } else {
            println!("{}", "No commits yet".yellow());
        }
    } else {
        println!("{}", "No commits yet".yellow());
    }
    Ok(())
}

#[allow(dead_code)]
pub async fn verify_history(repo: &Repository, commit_id: Option<&str>) -> Result<()> {
    let target_commit = if let Some(cid) = commit_id {
        cid.to_string()
    } else if let Some(current_branch) = repo.get_current_branch() {
        if let Some(head_commit) = current_branch.get_head_commit() {
            head_commit.clone()
        } else {
            println!("No commits yet");
            return Ok(());
        }
    } else {
        println!("No commits yet");
        return Ok(());
    };
    println!(
        "{}",
        format!("Verifying ancestry for commit: {}", target_commit)
            .bold()
            .blue()
    );
    let all_valid = Commit::verify_ancestry(repo, &target_commit, |commit, valid| {
        let commit_id = crate::utils::hash_utils::get_short_hash(&commit.id);
        let validity = if valid {
            "VALID".green()
        } else {
            "INVALID".red()
        };
        println!(
            "{} {} {}",
            commit_id.cyan(),
            validity,
            commit.message.bold()
        );
    });
    if all_valid {
        println!("{}", "All commits in ancestry are valid!".green().bold());
    } else {
        println!("{}", "Some commits failed verification!".red().bold());
    }
    Ok(())
}

pub async fn show_dag(repo: &Repository) -> Result<()> {
    use std::collections::{HashSet, VecDeque};
    println!("{}", "Commit DAG Visualization".bold().blue());
    println!("{}", "=".repeat(40).blue());
    if let Some(current_branch) = repo.get_current_branch() {
        if let Some(head_commit) = current_branch.get_head_commit() {
            let mut queue = VecDeque::new();
            let mut visited = HashSet::new();
            queue.push_back((head_commit.clone(), 0));
            while let Some((commit_id, depth)) = queue.pop_front() {
                if visited.contains(&commit_id) {
                    continue;
                }
                if let Ok(commit_object) =
                    crate::core::object::Object::load(&repo.get_objects_dir(), &commit_id)
                {
                    if let Ok(commit) = crate::core::commit::Commit::from_object(&commit_object) {
                        let indent = "  ".repeat(depth);
                        let parents = if commit.parent_ids.is_empty() {
                            "(root)".to_string()
                        } else {
                            commit
                                .parent_ids
                                .iter()
                                .map(|p| crate::utils::hash_utils::get_short_hash(p))
                                .collect::<Vec<_>>()
                                .join(", ")
                        };
                        println!(
                            "{}{} -> {}",
                            indent,
                            crate::utils::hash_utils::get_short_hash(&commit.id).cyan(),
                            parents
                        );
                        for parent in &commit.parent_ids {
                            queue.push_back((parent.clone(), depth + 1));
                        }
                        visited.insert(commit_id);
                    }
                }
            }
        } else {
            println!("{}", "No commits yet".yellow());
        }
    } else {
        println!("{}", "No commits yet".yellow());
    }
    Ok(())
}

fn display_commit_dag(
    commit: &crate::core::commit::Commit,
    is_head: bool,
    _depth: usize,
    valid: bool,
) {
    let branch_indicator = if is_head { "HEAD -> " } else { "     " };
    let commit_id = crate::utils::hash_utils::get_short_hash(&commit.id);
    let parents = if commit.parent_ids.is_empty() {
        "(root)".to_string()
    } else {
        commit
            .parent_ids
            .iter()
            .map(|p| crate::utils::hash_utils::get_short_hash(p))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let validity = if valid {
        "VALID".green()
    } else {
        "INVALID".red()
    };
    println!(
        "{}{} {} {}",
        branch_indicator,
        commit_id.cyan(),
        validity,
        commit.message.bold()
    );
    println!("{}", format!("    Parents: {}", parents).dimmed());
    println!(
        "{}",
        format!("    Author: {} <{}>", commit.author, commit.email).dimmed()
    );
    println!(
        "{}",
        format!(
            "    Date:   {}",
            commit.timestamp.format("%Y-%m-%d %H:%M:%S")
        )
        .dimmed()
    );
    println!(
        "{}",
        format!("    Files:  {} files changed", commit.files.len()).dimmed()
    );
    println!();
}
