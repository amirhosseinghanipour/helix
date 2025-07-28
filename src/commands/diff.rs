use crate::core::repository::Repository;
use anyhow::Result;
use colored::*;
use similar::{ChangeTag, TextDiff};
use std::fs;
use std::path::Path;

pub async fn show_diff(repo: &Repository, path: Option<&Path>) -> Result<()> {
    println!("{}", "Diff View".bold().blue());
    println!("{}", "=".repeat(40).blue());

    // Helper to get last committed content for a file
    fn get_last_commit_content(repo: &Repository, file_path: &Path) -> Option<String> {
        let branch = repo.get_current_branch()?;
        let head_commit = branch.get_head_commit()?;
        let commit = repo.get_commit_object(head_commit).ok()?;
        let file_change = commit.get_file_change(file_path.to_str()?)?;
        let blob_hash = &file_change.content_hash;
        let blob_obj =
            crate::core::object::Object::load(&repo.get_objects_dir(), blob_hash).ok()?;
        Some(blob_obj.data)
    }

    let files_to_diff: Vec<std::path::PathBuf> = if let Some(file_path) = path {
        vec![file_path.to_path_buf()]
    } else {
        let branch = match repo.get_current_branch() {
            Some(b) => b,
            None => {
                println!("{}", "No current branch found".red());
                return Ok(());
            }
        };
        let head_commit = match branch.get_head_commit() {
            Some(h) => h,
            None => {
                println!("{}", "No HEAD commit found".red());
                return Ok(());
            }
        };
        let commit = match repo.get_commit_object(head_commit) {
            Ok(obj) => obj,
            Err(_) => {
                println!("{}", "Failed to load HEAD commit object".red());
                return Ok(());
            }
        };
        commit
            .get_files()
            .keys()
            .map(|p| std::path::PathBuf::from(p))
            .collect()
    };

    let mut any_diff = false;
    for file_path in files_to_diff {
        let wd_content = fs::read_to_string(&file_path).unwrap_or_else(|_| String::new());
        let last_commit_content =
            get_last_commit_content(repo, &file_path).unwrap_or_else(|| String::new());
        if wd_content == last_commit_content {
            continue;
        }
        any_diff = true;
        println!("\nFile: {}", file_path.display().to_string().cyan());
        let diff = TextDiff::from_lines(&last_commit_content, &wd_content);
        for change in diff.iter_all_changes() {
            let (sign, color) = match change.tag() {
                ChangeTag::Delete => ("-", "red"),
                ChangeTag::Insert => ("+", "green"),
                ChangeTag::Equal => (" ", "white"),
            };
            let line = change.to_string();
            match color {
                "red" => print!("{}", format!("{}{}", sign, line).red()),
                "green" => print!("{}", format!("{}{}", sign, line).green()),
                _ => print!("{}{}", sign, line),
            }
        }
    }
    if !any_diff {
        println!("\n{}", "No differences found".green());
        println!("Working directory is clean");
    }
    Ok(())
}
