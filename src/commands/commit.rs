use crate::core::{commit::Commit, object::Tree, repository::Repository};
use anyhow::Result;
use colored::*;
use ed25519_dalek::SigningKey;
use indicatif::{ProgressBar, ProgressStyle};

pub async fn commit_changes(
    repo: &mut Repository,
    message: &str,
    keypair: &SigningKey,
) -> Result<()> {
    if repo.index.is_empty() {
        println!("{}", "No changes to commit".yellow());
        println!("Use 'hx add' to stage files first");
        return Ok(());
    }

    let pb = ProgressBar::new(3);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .unwrap(),
    );

    pb.set_message("Creating commit object...");

    // Get parent commit ID
    let parent_ids = if let Some(current_branch) = repo.get_current_branch() {
        if let Some(head_commit) = current_branch.get_head_commit() {
            vec![head_commit.clone()]
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    // Create tree object from staged files (use blob hashes from index)
    let mut tree = Tree::new();
    for entry in repo.index.get_all_files() {
        tree.add_entry(
            entry.path.clone(),
            entry.content_hash.clone(), // This is now the blob hash
            "blob".to_string(),
            entry.mode,
        );
    }
    let tree_object = tree.to_object();
    tree_object.save(&repo.get_objects_dir())?;
    let tree_id = tree_object.id.clone();

    // Create commit and sign it
    let commit = Commit::new(
        parent_ids,
        tree_id,
        repo.config.author.clone(),
        repo.config.email.clone(),
        message.to_string(),
        repo.index.to_file_changes(),
        Some(keypair),
    );
    // commit.sign(keypair); // Already signed in new()

    pb.inc(1);
    pb.set_message("Saving commit object...");

    // Save commit object
    let commit_object = commit.to_object();
    commit_object.save(&repo.get_objects_dir())?;

    // Verify the object was saved correctly
    if commit_object.is_commit() {
        println!(
            "Commit object saved with ID: {}",
            commit_object.get_short_id().cyan()
        );
    }

    pb.inc(1);
    pb.set_message("Updating branch...");

    // Update current branch
    if let Some(current_branch) = repo.get_current_branch_mut() {
        current_branch.update_head(commit.id.clone());
    }

    // Clear index after successful commit
    repo.index.clear();
    repo.save()?;

    pb.finish_with_message("Commit created successfully!");

    println!("\n{}", "Commit created successfully!".green().bold());
    println!("Commit ID: {}", commit.get_short_id().cyan());
    println!("Message: {}", message.blue());
    println!("Author: {} <{}>", repo.config.author, repo.config.email);
    println!(
        "Date: {}",
        commit
            .timestamp
            .format("%Y-%m-%d %H:%M:%S")
            .to_string()
            .yellow()
    );
    println!(
        "Files: {} files changed",
        commit.files.len().to_string().magenta()
    );
    println!("Branch: {}", repo.current_branch.yellow().bold());

    Ok(())
}
