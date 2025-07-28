use crate::core::object::Object;
use crate::core::repository::Repository;
use anyhow::Result;
use colored::*;
use diffy::merge;
use crate::core::commit::ChangeType;
use std::fmt;
use chrono::Utc;

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

fn find_merge_base(repo: &Repository, commit1: &str, commit2: &str) -> Option<String> {
    use std::collections::{HashSet, VecDeque};
    // Collect all ancestors of commit1
    let mut ancestors1 = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(commit1.to_string());
    while let Some(current) = queue.pop_front() {
        if !ancestors1.insert(current.clone()) {
            continue;
        }
        if let Ok(obj) = Object::load(&repo.get_objects_dir(), &current) {
            if let Ok(commit) = crate::core::commit::Commit::from_object(&obj) {
                for parent in &commit.parent_ids {
                    queue.push_back(parent.clone());
                }
            }
        }
    }
    // Walk ancestors of commit2, return first found in ancestors1
    let mut queue = VecDeque::new();
    queue.push_back(commit2.to_string());
    while let Some(current) = queue.pop_front() {
        if ancestors1.contains(&current) {
            return Some(current);
        }
        if let Ok(obj) = Object::load(&repo.get_objects_dir(), &current) {
            if let Ok(commit) = crate::core::commit::Commit::from_object(&obj) {
                for parent in &commit.parent_ids {
                    queue.push_back(parent.clone());
                }
            }
        }
    }
    None
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
    
    if let (Some(_base), Some(ours), Some(theirs)) =
        (base_commit_id, ours_commit_id, theirs_commit_id)
    {
        // Find the true merge base
        let resolved_base_commit_id = match find_merge_base(repo, &ours, &theirs) {
            Some(base) => base,
            None => {
                println!("{}", "Warning: No common ancestor found, using root commit as base".yellow());
                // Fallback: use the root commit (first commit in ours history)
                let mut root = ours.clone();
                let mut last = ours.clone();
                while let Ok(obj) = Object::load(&repo.get_objects_dir(), &root) {
                    if let Ok(commit) = crate::core::commit::Commit::from_object(&obj) {
                        if let Some(parent) = commit.parent_ids.first() {
                            last = parent.clone();
                            root = parent.clone();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                last
            }
        };
        
        // Load commits with better error handling
        let base_commit = match Object::load(&repo.get_objects_dir(), &resolved_base_commit_id) {
            Ok(obj) => match crate::core::commit::Commit::from_object(&obj) {
                Ok(commit) => commit,
                Err(_) => {
                    println!("{}", format!("Failed to parse base commit: {}", resolved_base_commit_id).red());
                    return Ok(());
                }
            },
            Err(_) => {
                println!("{}", format!("Failed to load base commit: {}", resolved_base_commit_id).red());
                return Ok(());
            }
        };
        
        let ours_commit = match Object::load(&repo.get_objects_dir(), &ours) {
            Ok(obj) => match crate::core::commit::Commit::from_object(&obj) {
                Ok(commit) => commit,
                Err(_) => {
                    println!("{}", format!("Failed to parse our commit: {}", ours).red());
                    return Ok(());
                }
            },
            Err(_) => {
                println!("{}", format!("Failed to load our commit: {}", ours).red());
                return Ok(());
            }
        };
        
        let theirs_commit = match Object::load(&repo.get_objects_dir(), &theirs) {
            Ok(obj) => match crate::core::commit::Commit::from_object(&obj) {
                Ok(commit) => commit,
                Err(_) => {
                    println!("{}", format!("Failed to parse their commit: {}", theirs).red());
                    return Ok(());
                }
            },
            Err(_) => {
                println!("{}", format!("Failed to load their commit: {}", theirs).red());
                return Ok(());
            }
        };

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
            if let Some(_renamed_from) = ours_renamed.or(theirs_renamed) {
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
                    match Object::load(&repo.get_objects_dir(), ours_hash) {
                        Ok(ours_obj) => {
                            if let Err(e) = std::fs::write(&actual_path, &ours_obj.data) {
                                println!("{}", format!("Failed to write file {}: {}", path, e).red());
                            }
                        }
                        Err(_) => {
                            println!("{}", format!("Failed to load our blob: {}", ours_hash).red());
                        }
                    }
                    continue;
                }
                (None, Some(theirs_hash)) => {
                    match Object::load(&repo.get_objects_dir(), theirs_hash) {
                        Ok(theirs_obj) => {
                            if let Err(e) = std::fs::write(&actual_path, &theirs_obj.data) {
                                println!("{}", format!("Failed to write file {}: {}", path, e).red());
                            }
                        }
                        Err(_) => {
                            println!("{}", format!("Failed to load their blob: {}", theirs_hash).red());
                        }
                    }
                    continue;
                }
                (None, None) => continue, // deleted or missing
                _ => {}
            }

            // Both sides have the file, do a three-way merge
            let base_content = if let Some(base_hash) = base_blob {
                match Object::load(&repo.get_objects_dir(), &base_hash) {
                    Ok(obj) => obj.data,
                    Err(_) => String::new()
                }
            } else {
                String::new()
            };
            
            let ours_content = match Object::load(&repo.get_objects_dir(), ours_blob.as_ref().unwrap()) {
                Ok(obj) => obj.data,
                Err(_) => {
                    println!("{}", format!("Failed to load our content for: {}", path).red());
                    continue;
                }
            };
            
            let theirs_content = match Object::load(&repo.get_objects_dir(), theirs_blob.as_ref().unwrap()) {
                Ok(obj) => obj.data,
                Err(_) => {
                    println!("{}", format!("Failed to load their content for: {}", path).red());
                    continue;
                }
            };
            
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
                        if let Err(e) = std::fs::write(&actual_path, &ours_content) {
                            println!("{}", format!("Failed to write our version to {}: {}", path, e).red());
                        }
                        continue;
                    }
                    MergeStrategy::Theirs => {
                        if let Err(e) = std::fs::write(&actual_path, &theirs_content) {
                            println!("{}", format!("Failed to write their version to {}: {}", path, e).red());
                        }
                        continue;
                    }
                    MergeStrategy::Manual => {
                        if let Err(e) = std::fs::write(&actual_path, merged) {
                            println!("{}", format!("Failed to write conflict markers to {}: {}", path, e).red());
                        }
                    }
                }
            } else {
                if let Err(e) = std::fs::write(&actual_path, merged) {
                    println!("{}", format!("Failed to write merged content to {}: {}", path, e).red());
                }
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

        // If we performed a true merge (not fast-forward), create a merge commit
        if resolved_base_commit_id != ours && resolved_base_commit_id != theirs {
            use crate::core::commit::Commit;
            use crate::core::object::Object as CoreObject;
            use crate::core::index::{Index, IndexEntry};
            use crate::core::object::Tree;
            // Stage all merged files
            let mut index = Index::new();
            for entry in std::fs::read_dir(".")? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let data = std::fs::read_to_string(&path)?;
                    let blob = CoreObject::new("blob".to_string(), data.clone());
                    blob.save(&repo.get_objects_dir())?;
                    let index_entry = IndexEntry {
                        path: path.file_name().unwrap().to_string_lossy().to_string(),
                        content_hash: blob.id.clone(),
                        mode: 0o100644,
                        size: data.len() as u64,
                        stage: 0,
                        timestamp: Utc::now(),
                    };
                    index.add_file(&index_entry.path.clone(), index_entry);
                }
            }
            // Create tree object
            let mut tree = Tree::new();
            for entry in index.get_all_files() {
                tree.add_entry(
                    entry.path.clone(),
                    entry.content_hash.clone(),
                    "blob".to_string(),
                    entry.mode,
                );
            }
            let tree_object = tree.to_object();
            tree_object.save(&repo.get_objects_dir())?;
            let tree_id = tree_object.id.clone();
            // Create merge commit
            let author = repo.config.author.clone();
            let email = repo.config.email.clone();
            let message = format!(
                "Merge branch '{}' into '{}'",
                branch_name, repo.current_branch
            );
            let parents = vec![ours.clone(), theirs.clone()];
            let file_changes = index.to_file_changes();
            let commit = Commit::new(
                parents,
                tree_id,
                author,
                email,
                message,
                file_changes,
                None,
            );
            let commit_object = commit.to_object();
            commit_object.save(&repo.get_objects_dir())?;
            // Update branch head
            if let Some(current_branch) = repo.get_current_branch_mut() {
                current_branch.set_head_commit(commit_object.id.clone());
            }
            repo.save()?;
            println!("{}", format!("Created merge commit: {}", commit_object.id).green().bold());
        }
    } else {
        println!("{}", "Could not find merge base or commits".red());
        println!("Make sure both branches have commits and try again.");
    }
    Ok(())
}

fn diff3_merge(base: &str, ours: &str, theirs: &str, _path: &std::path::Path) -> String {
    match merge(base, ours, theirs) {
        Ok(result) => result,
        Err(conflict) => conflict,
    }
}
