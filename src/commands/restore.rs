use crate::core::object::Object;
use crate::core::repository::Repository;
use crate::utils::file_utils;
use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

pub async fn restore_files(repo: &Repository, paths: Vec<std::path::PathBuf>) -> Result<()> {
    let pb = ProgressBar::new(paths.len() as u64);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .unwrap(),
    );

    pb.set_message("Restoring files from last commit...");

    // Get the last commit
    let current_branch = repo
        .get_current_branch()
        .ok_or_else(|| anyhow::anyhow!("No current branch found"))?;

    let head_commit_id = current_branch
        .get_head_commit()
        .ok_or_else(|| anyhow::anyhow!("No commits found"))?;

    // Load the commit object
    let commit_object = crate::core::object::Object::load(&repo.get_objects_dir(), head_commit_id)?;
    let commit = crate::core::commit::Commit::from_object(&commit_object)?;

    let mut restored_count = 0;
    let mut skipped_count = 0;

    for path in paths {
        let relative_path = path
            .strip_prefix(&repo.path)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        pb.set_message(format!("Restoring {}", relative_path));

        // Check if file exists in the commit
        if let Some(file_change) = commit.get_file_change(&relative_path) {
            // Load the blob object and restore the content
            let blob_object = Object::load(&repo.get_objects_dir(), &file_change.content_hash)?;
            let content = blob_object.data.as_bytes();
            if let Ok(_) = file_utils::write_file_content(&path, content) {
                restored_count += 1;
            } else {
                skipped_count += 1;
            }
        } else {
            skipped_count += 1;
        }

        pb.inc(1);
    }

    pb.finish_with_message("Files restored successfully!");

    println!("\n{}", "Files restored successfully!".green().bold());
    println!("Restored: {} files", restored_count.to_string().cyan());
    if skipped_count > 0 {
        println!("Skipped: {} files", skipped_count.to_string().yellow());
    }

    Ok(())
}
