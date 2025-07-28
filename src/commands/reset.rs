use crate::core::index::IndexEntry;
use crate::core::index::IndexNode;
use crate::core::repository::Repository;
use anyhow::Result;
use chrono::Utc;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;

pub async fn reset_repository(repo: &mut Repository, target: &str, mode: &str) -> Result<()> {
    let pb = ProgressBar::new(3);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .unwrap(),
    );

    pb.set_message("Resetting repository...");

    // Find the target commit
    let current_branch_name = repo.current_branch.clone();
    let commit_id = if target == "HEAD" {
        // Limit the scope of the branch reference to this block
        let branch = repo
            .branches
            .get(&current_branch_name)
            .ok_or_else(|| anyhow::anyhow!("No current branch"))?;
        branch
            .get_head_commit()
            .ok_or_else(|| anyhow::anyhow!("No HEAD commit"))?
            .to_string()
    } else {
        target.to_string()
    };
    let commit = repo.get_commit_object(&commit_id)?;

    match mode {
        "soft" => {
            // Move HEAD only
            let _ = repo.set_head(&commit_id);
            pb.inc(1);
            pb.set_message("HEAD moved (soft reset)...");
        }
        "mixed" => {
            // Move HEAD and reset index
            let _ = repo.set_head(&commit_id);
            repo.index.clear();
            for (path, file_change) in commit.get_files() {
                let entry = IndexEntry {
                    path: path.clone(),
                    content_hash: file_change.content_hash.clone(),
                    size: file_change.size,
                    mode: file_change.mode,
                    timestamp: Utc::now(),
                    stage: 0,
                };
                repo.index
                    .entries
                    .insert(path.clone(), IndexNode::File(entry));
            }
            pb.inc(1);
            pb.set_message("Index reset (mixed reset)...");
        }
        "hard" => {
            // Move HEAD, reset index, and update working directory
            let _ = repo.set_head(&commit_id);
            repo.index.clear();
            for (path, file_change) in commit.get_files() {
                let entry = IndexEntry {
                    path: path.clone(),
                    content_hash: file_change.content_hash.clone(),
                    size: file_change.size,
                    mode: file_change.mode,
                    timestamp: Utc::now(),
                    stage: 0,
                };
                repo.index
                    .entries
                    .insert(path.clone(), IndexNode::File(entry));
                // Overwrite working directory file
                let blob_obj = crate::core::object::Object::load(
                    &repo.get_objects_dir(),
                    &file_change.content_hash,
                )?;
                fs::write(path, blob_obj.data)?;
            }
            pb.inc(1);
            pb.set_message("Index and working directory reset (hard reset)...");
        }
        _ => {
            println!(
                "{}",
                format!("Unknown reset mode: {}. Use soft, mixed, or hard.", mode).red()
            );
            return Ok(());
        }
    }

    pb.inc(1);
    pb.set_message("Updating repository state...");
    repo.save()?;
    pb.finish_with_message("Repository reset successfully!");

    println!("\n{}", "Repository reset successfully!".green().bold());
    println!("Target: {}", target.cyan());
    println!("Current branch: {}", repo.current_branch.yellow().bold());
    println!("Reset mode: {}", mode.cyan());
    Ok(())
}
