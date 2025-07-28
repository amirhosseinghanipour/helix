use crate::core::object::Object;
use crate::core::repository::Repository;
use crate::utils::{file_utils, path_utils};
use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use walkdir::WalkDir;

pub async fn add_files(repo: &mut Repository, paths: &[std::path::PathBuf]) -> Result<()> {
    let mut files_to_add = Vec::new();

    // Collect all files to add
    for path in paths {
        if path.is_file() {
            if !path_utils::is_ignored(path, &repo.path) {
                files_to_add.push(path.clone());
            }
        } else if path.is_dir() {
            for entry in WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let entry_path = entry.path();
                if !path_utils::is_ignored(entry_path, &repo.path) {
                    files_to_add.push(entry_path.to_path_buf());
                }
            }
        }
    }

    if files_to_add.is_empty() {
        println!("{}", "No files to add".yellow());
        return Ok(());
    }

    let pb = ProgressBar::new(files_to_add.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    let mut added_count = 0;
    let mut skipped_count = 0;

    for file_path in files_to_add {
        let relative_path =
            path_utils::normalize_path(&file_path.strip_prefix(&repo.path).unwrap_or(&file_path))
                .to_string_lossy()
                .to_string();

        pb.set_message(format!("Adding {}", relative_path));

        if let Ok(content) = file_utils::read_file_content(&file_path) {
            let mode = file_utils::get_file_mode(&file_path)?;
            // Check if file is executable and set appropriate mode
            let mode = if file_utils::is_executable(&file_path)? {
                mode | 0o111
            } else {
                mode
            };

            // --- Blob storage logic ---
            let blob_object = Object::new(
                "blob".to_string(),
                String::from_utf8_lossy(&content).to_string(),
            );
            blob_object.save(&repo.get_objects_dir())?;
            let blob_hash = blob_object.id.clone();
            // --- End blob storage logic ---

            // Store the blob hash in the index
            let entry = crate::core::index::IndexEntry {
                path: relative_path.clone(),
                content_hash: blob_hash,
                size: content.len() as u64,
                mode,
                timestamp: chrono::Utc::now(),
                stage: 0,
            };
            repo.index.add_file(&relative_path, entry);
            added_count += 1;
        } else {
            skipped_count += 1;
        }

        pb.inc(1);
    }

    pb.finish_with_message("Files added successfully!");

    repo.save()?;

    println!("\n{}", "Files staged successfully!".green().bold());
    println!("Added: {} files", added_count.to_string().cyan());
    if skipped_count > 0 {
        println!("Skipped: {} files", skipped_count.to_string().yellow());
    }
    println!(
        "Total staged: {} files",
        repo.index.len().to_string().blue()
    );

    Ok(())
}
