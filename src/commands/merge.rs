use crate::core::object::Object;
use crate::core::repository::Repository;
use anyhow::Result;
use colored::*;
use diffy::merge;
use crate::core::commit::{ChangeType, FileChange};
use std::fmt;

/// Merge conflict resolution strategy
pub enum MergeStrategy {
    Ours,
    Theirs,
    Manual,
}

impl fmt::Display for MergeStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MergeStrategy::Ours => write!(f, "ours"),
            MergeStrategy::Theirs => write!(f, "theirs"),
            MergeStrategy::Manual => write!(f, "manual"),
        }
    }
}

pub async fn merge_branch(
    repo: &mut Repository,
    branch_name: &str,
    strategy: Option<MergeStrategy>,
) -> Result<()> {
    let strategy = strategy.unwrap_or(MergeStrategy::Manual);
    if !repo.branches.contains_key(branch_name) {
        println!(
            "{}",
            format!("Branch '{}' does not exist", branch_name).red()
        );
        return Ok(());
    }
    if branch_name == repo.current_branch {
        println!("{}", "Cannot merge branch into itself".red());
        return Ok(());
    }
    println!(
        "{}",
        format!(
            "Merging branch '{}' into '{}' with strategy: {}",
            branch_name, repo.current_branch, strategy
        )
        .blue()
        .bold()
    );
    // Find merge base (for now, use main as base if exists)
    let base_commit_id = repo
        .branches
        .get("main")
        .and_then(|b| b.get_head_commit())
        .cloned();
    let ours_commit_id = repo
        .get_current_branch()
        .and_then(|b| b.get_head_commit())
        .cloned();
    let theirs_commit_id = repo
        .branches
        .get(branch_name)
        .and_then(|b| b.get_head_commit())
        .cloned();
    if let (Some(base), Some(ours), Some(theirs)) =
        (base_commit_id, ours_commit_id, theirs_commit_id)
    {
        let base_commit = Object::load(&repo.get_objects_dir(), &base)?;
        let ours_commit = Object::load(&repo.get_objects_dir(), &ours)?;
        let theirs_commit = Object::load(&repo.get_objects_dir(), &theirs)?;
        let base_commit = crate::core::commit::Commit::from_object(&base_commit)?;
        let ours_commit = crate::core::commit::Commit::from_object(&ours_commit)?;
        let theirs_commit = crate::core::commit::Commit::from_object(&theirs_commit)?;

        // Collect all file paths from base, ours, and theirs
        let mut all_paths = std::collections::HashSet::new();
        for commit in [&base_commit, &ours_commit, &theirs_commit] {
            for (path, _) in commit.get_files() {
                all_paths.insert(path.clone());
            }
        }
        // Also handle renames: add old_path for renamed files
        for commit in [&base_commit, &ours_commit, &theirs_commit] {
            for fc in commit.get_files().values() {
                if let ChangeType::Renamed { old_path } = &fc.change_type {
                    all_paths.insert(old_path.clone());
                }
            }
        }

        let mut conflicts = 0;
        let mut conflicted_files = Vec::new();
        for path in all_paths {
            let base_fc = base_commit.get_file_change(&path);
            let ours_fc = ours_commit.get_file_change(&path);
            let theirs_fc = theirs_commit.get_file_change(&path);

            // Handle deletions
            let ours_deleted = ours_fc.map_or(false, |fc| matches!(fc.change_type, ChangeType::Deleted));
            let theirs_deleted = theirs_fc.map_or(false, |fc| matches!(fc.change_type, ChangeType::Deleted));
            if ours_deleted || theirs_deleted {
                // If deleted in either, remove file if exists
                if std::path::Path::new(&path).exists() {
                    let _ = std::fs::remove_file(&path);
                }
                continue;
            }

            // Handle renames
            let ours_renamed = ours_fc.and_then(|fc| match &fc.change_type {
                ChangeType::Renamed { old_path } => Some(old_path.clone()),
                _ => None,
            });
            let theirs_renamed = theirs_fc.and_then(|fc| match &fc.change_type {
                ChangeType::Renamed { old_path } => Some(old_path.clone()),
                _ => None,
            });
            let mut actual_path = path.clone();
            if let Some(renamed_from) = ours_renamed.or(theirs_renamed) {
                // If both renamed from the same file, merge into new name
                actual_path = path.clone();
                // Optionally, handle more complex rename conflicts here
            }

            // Get blob hashes
            let base_blob = base_fc.map(|fc| fc.content_hash.clone());
            let ours_blob = ours_fc.map(|fc| fc.content_hash.clone());
            let theirs_blob = theirs_fc.map(|fc| fc.content_hash.clone());

            // If file only exists in one side, take that version
            match (ours_blob.as_ref(), theirs_blob.as_ref()) {
                (Some(ours_hash), None) => {
                    let ours_obj = Object::load(&repo.get_objects_dir(), ours_hash)?;
                    std::fs::write(&actual_path, &ours_obj.data)?;
                    continue;
                }
                (None, Some(theirs_hash)) => {
                    let theirs_obj = Object::load(&repo.get_objects_dir(), theirs_hash)?;
                    std::fs::write(&actual_path, &theirs_obj.data)?;
                    continue;
                }
                (None, None) => continue, // deleted or missing
                _ => {}
            }

            // Both sides have the file, do a three-way merge
            let base_content = if let Some(base_hash) = base_blob {
                Object::load(&repo.get_objects_dir(), &base_hash).ok().map(|o| o.data).unwrap_or_default()
            } else {
                String::new()
            };
            let ours_content = Object::load(&repo.get_objects_dir(), ours_blob.as_ref().unwrap())?.data;
            let theirs_content = Object::load(&repo.get_objects_dir(), theirs_blob.as_ref().unwrap())?.data;
            let merged = diff3_merge(
                &base_content,
                &ours_content,
                &theirs_content,
                std::path::Path::new(&actual_path),
            );
            if merged.contains("<<<<<<<") {
                // Conflict detected
                conflicts += 1;
                conflicted_files.push(actual_path.clone());
                match strategy {
                    MergeStrategy::Ours => {
                        std::fs::write(&actual_path, &ours_content)?;
                        continue;
                    }
                    MergeStrategy::Theirs => {
                        std::fs::write(&actual_path, &theirs_content)?;
                        continue;
                    }
                    MergeStrategy::Manual => {
                        std::fs::write(&actual_path, merged)?;
                    }
                }
            } else {
                std::fs::write(&actual_path, merged)?;
            }
        }
        if conflicts > 0 {
            match strategy {
                MergeStrategy::Manual => {
                    println!(
                        "{}",
                        format!("Merge completed with {} conflicts.", conflicts)
                            .yellow()
                            .bold()
                    );
                    println!("{}", "Conflicted files:");
                    for f in conflicted_files {
                        println!("  {}", f.red().bold());
                    }
                    println!("Please resolve conflicts and commit the result.");
                }
                MergeStrategy::Ours | MergeStrategy::Theirs => {
                    println!(
                        "{}",
                        format!("Merge completed with {} conflicts, resolved automatically using '{}'.", conflicts, strategy)
                            .yellow()
                            .bold()
                    );
                }
            }
        } else {
            println!("{}", "Merge completed successfully".green().bold());
        }
        println!("Current branch: {}", repo.current_branch.yellow().bold());
    } else {
        println!("{}", "Could not find merge base or commits".red());
    }
    Ok(())
}

fn diff3_merge(base: &str, ours: &str, theirs: &str, _path: &std::path::Path) -> String {
    match merge(base, ours, theirs) {
        Ok(result) => result,
        Err(conflict) => conflict,
    }
}
