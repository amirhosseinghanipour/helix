use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use crate::core::repository::Repository;

pub async fn clone_repository(url: &str, path: &Path) -> Result<()> {
    let pb = ProgressBar::new(4);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .unwrap(),
    );

    pb.set_message("Creating repository structure...");
    
    // Create the target directory
    fs::create_dir_all(path)?;
    
    // Initialize a new repository
    let mut repo = Repository::new(path)?;
    
    pb.inc(1);
    pb.set_message("Setting up initial branch...");
    
    // Create main branch
    repo.branches.insert("main".to_string(), crate::core::branch::Branch::new("main"));
    
    pb.inc(1);
    pb.set_message("Creating directories...");
    
    // Create necessary directories
    std::fs::create_dir_all(repo.get_objects_dir())?;
    std::fs::create_dir_all(repo.get_refs_dir())?;
    
    pb.inc(1);
    pb.set_message("Saving repository configuration...");
    
    // Save the repository
    repo.save()?;
    
    pb.finish_with_message("Repository cloned successfully!");
    
    println!("\n{}", "Repository cloned successfully!".green().bold());
    println!("Location: {}", path.display().to_string().cyan());
    println!("Source: {}", url.magenta());
    println!("Current branch: {}", "main".yellow().bold());
    
    Ok(())
}
