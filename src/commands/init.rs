use crate::core::repository::Repository;
use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;

pub async fn init_repository(path: &Path) -> Result<()> {
    let pb = ProgressBar::new(4);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .unwrap(),
    );

    pb.set_message("Creating repository structure...");
    let mut repo = Repository::new(path)?;

    pb.inc(1);
    pb.set_message("Setting up initial branch...");
    repo.branches
        .insert("main".to_string(), crate::core::branch::Branch::new("main"));

    pb.inc(1);
    pb.set_message("Creating directories...");
    std::fs::create_dir_all(repo.get_objects_dir())?;
    std::fs::create_dir_all(repo.get_refs_dir())?;

    pb.inc(1);
    pb.set_message("Saving repository configuration...");
    repo.save()?;

    pb.finish_with_message("Repository initialized successfully!");

    println!(
        "\n{}",
        "Helix repository initialized successfully!".green().bold()
    );
    println!("Repository location: {}", path.display().to_string().cyan());
    println!("Current branch: {}", "main".yellow().bold());
    println!("\n{}", "Next steps:".bold());
    println!("  hx add .     # Stage all files");
    println!("  hx commit -m \"Initial commit\"  # Create first commit");
    println!("  hx status    # Check repository status");

    Ok(())
}
