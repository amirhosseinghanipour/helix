use crate::core::repository::Repository;
use crate::utils::path_utils;
use anyhow::Result;
use colored::*;
use std::collections::HashMap;
use walkdir::WalkDir;

pub async fn show_status(repo: &Repository) -> Result<()> {
    println!("{}", "Repository Status".bold().blue());
    println!("{}", "=".repeat(40).blue());

    // Show current branch
    println!("On branch: {}", repo.current_branch.yellow().bold());

    if let Some(current_branch) = repo.get_current_branch() {
        if let Some(head_commit) = current_branch.get_head_commit() {
            println!("HEAD: {}", head_commit[..8].cyan());
        } else {
            println!("HEAD: {}", "No commits yet".red());
        }
    }

    println!();

    // Get working directory files
    let working_files = get_working_directory_files(&repo.path)?;

    // Get staged files
    let staged_files: Vec<_> = repo.index.get_file_paths();
    let staged_count = repo.index.get_staged_files().len();

    // Get last commit files (if any)
    let last_commit_files = if let Some(current_branch) = repo.get_current_branch() {
        if let Some(head_commit) = current_branch.get_head_commit() {
            if let Ok(commit_object) =
                crate::core::object::Object::load(&repo.get_objects_dir(), head_commit)
            {
                if let Ok(commit) = crate::core::commit::Commit::from_object(&commit_object) {
                    commit.get_files().keys().cloned().collect()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Analyze changes
    let mut changes = HashMap::new();

    // Check for new files
    for file in &working_files {
        if !last_commit_files.contains(file) && !staged_files.contains(file) {
            changes.insert(file.clone(), "untracked".to_string());
        }
    }

    // Check for modified files
    for file in &working_files {
        if last_commit_files.contains(file) && !staged_files.contains(file) {
            changes.insert(file.clone(), "modified".to_string());
        }
    }

    // Check for staged files
    for file in &staged_files {
        if !last_commit_files.contains(file) {
            changes.insert(file.clone(), "staged".to_string());
        } else {
            changes.insert(file.clone(), "staged".to_string());
        }
    }

    // Group changes by status
    let mut untracked = Vec::new();
    let mut modified = Vec::new();
    let mut staged = Vec::new();

    for (file, status) in changes {
        match status.as_str() {
            "untracked" => untracked.push(file),
            "modified" => modified.push(file),
            "staged" => staged.push(file),
            _ => {}
        }
    }

    // Show file change types if we have staged files
    if !staged.is_empty() {
        let mut added = 0;
        let mut modified = 0;
        let mut deleted = 0;

        for entry in repo.index.get_staged_files() {
            // Check if file exists in working directory to determine change type
            let file_path = repo.path.join(&entry.path);
            if file_path.exists() {
                if let Ok(current_hash) = crate::utils::hash_utils::calculate_file_hash(&file_path)
                {
                    if current_hash != entry.content_hash {
                        modified += 1;
                    } else {
                        added += 1;
                    }
                } else {
                    added += 1;
                }
            } else {
                deleted += 1;
            }
        }

        if added > 0 {
            println!("  📈 Added: {} files", added.to_string().green());
        }
        if modified > 0 {
            println!("  Modified: {} files", modified.to_string().yellow());
        }
        if deleted > 0 {
            println!("  Deleted: {} files", deleted.to_string().red());
        }
    }

    // Display changes
    if !staged.is_empty() {
        println!("{}", "Changes to be committed:".green().bold());
        for file in &staged {
            println!("  {}", format!("  + {}", file).green());
        }
        println!();
    }

    if !modified.is_empty() {
        println!("{}", "Changes not staged for commit:".yellow().bold());
        for file in &modified {
            println!("  {}", format!("  ~ {}", file).yellow());
        }
        println!();
    }

    if !untracked.is_empty() {
        println!("{}", "❓ Untracked files:".red().bold());
        for file in &untracked {
            println!("  {}", format!("  ? {}", file).red());
        }
        println!();
    }

    if staged.is_empty() && modified.is_empty() && untracked.is_empty() {
        println!("{}", "Working tree clean".green().bold());
    } else {
        println!("Summary:");
        println!("  Staged: {} files", staged_count.to_string().green());
        println!("  Modified: {} files", modified.len().to_string().yellow());
        println!("  Untracked: {} files", untracked.len().to_string().red());
    }

    Ok(())
}

fn get_working_directory_files(repo_path: &std::path::Path) -> Result<Vec<String>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(repo_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let entry_path = entry.path();
        if !path_utils::is_ignored(entry_path, repo_path) {
            if let Some(relative_path) = path_utils::get_relative_path(repo_path, entry_path) {
                files.push(relative_path);
            }
        }
    }

    Ok(files)
}
