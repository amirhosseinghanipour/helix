use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;
use crate::core::repository::Repository;
use crate::utils::remote_client::RemoteClient;
use crate::core::object::Object;
use git2::Repository as GitRepository;
use std::process::Command;

pub async fn clone_repository(url: &str, path: &Path) -> Result<()> {
    // Heuristic: detect VCS type
    let is_git = url.ends_with(".git") || url.contains("github.com") || url.contains("gitlab.com");
    let is_hg = url.contains("bitbucket.org") || url.ends_with(".hg") || url.contains("mercurial");
    let is_svn = url.contains("svn") || url.contains("subversion") || url.ends_with("/trunk") || url.ends_with("/branches");
    let is_bzr = url.contains("launchpad.net") || url.ends_with(".bzr") || url.contains("bazaar");

    if is_git {
        println!("{}", format!("Cloning Git repository from {}...", url).blue().bold());
        match GitRepository::clone(url, path) {
            Ok(_) => {
                println!("{}", "Git repository cloned successfully!".green().bold());
                println!("Location: {}", path.display().to_string().cyan());
                println!("Source: {}", url.magenta());
                return Ok(());
            }
            Err(e) => {
                println!("{}", format!("Failed to clone Git repository: {}", e).red());
            }
        }
    } else if is_hg {
        println!("{}", format!("Cloning Mercurial (hg) repository from {}...", url).blue().bold());
        let status = Command::new("hg")
            .arg("clone")
            .arg(url)
            .arg(path)
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("{}", "Mercurial repository cloned successfully!".green().bold());
                println!("Location: {}", path.display().to_string().cyan());
                println!("Source: {}", url.magenta());
                return Ok(());
            }
            Ok(_) | Err(_) => {
                println!("{}", "Failed to clone Mercurial repository. Is 'hg' installed?".red());
            }
        }
    } else if is_svn {
        println!("{}", format!("Cloning Subversion (svn) repository from {}...", url).blue().bold());
        let status = Command::new("svn")
            .arg("checkout")
            .arg(url)
            .arg(path)
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("{}", "Subversion repository cloned successfully!".green().bold());
                println!("Location: {}", path.display().to_string().cyan());
                println!("Source: {}", url.magenta());
                return Ok(());
            }
            Ok(_) | Err(_) => {
                println!("{}", "Failed to clone Subversion repository. Is 'svn' installed?".red());
            }
        }
    } else if is_bzr {
        println!("{}", format!("Cloning Bazaar (bzr) repository from {}...", url).blue().bold());
        let status = Command::new("bzr")
            .arg("branch")
            .arg(url)
            .arg(path)
            .status();
        match status {
            Ok(s) if s.success() => {
                println!("{}", "Bazaar repository cloned successfully!".green().bold());
                println!("Location: {}", path.display().to_string().cyan());
                println!("Source: {}", url.magenta());
                return Ok(());
            }
            Ok(_) | Err(_) => {
                println!("{}", "Failed to clone Bazaar repository. Is 'bzr' installed?".red());
            }
        }
    }

    // Default: try Helix
    let pb = ProgressBar::new(5);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .unwrap(),
    );

    pb.set_message("Creating repository structure...");
    fs::create_dir_all(path)?;
    let mut repo = Repository::new(path)?;
    pb.inc(1);

    pb.set_message("Connecting to remote...");
    let client = RemoteClient::new(url);
    let head = match client.get_ref("main").await {
        Ok(h) => h,
        Err(_) => {
            pb.finish_with_message("Failed: Only Helix remote repositories are supported. This is not a Helix remote.");
            return Err(anyhow::anyhow!("Remote is not a valid Helix repository or is unreachable. Only Helix remotes are supported (not Git, hg, svn, bzr)."));
        }
    };
    pb.inc(1);

    pb.set_message("Fetching objects...");
    let objects_dir = path.join(".helix/objects");
    let mut to_download = vec![head.clone()];
    let mut seen = std::collections::HashSet::new();
    while let Some(hash) = to_download.pop() {
        if seen.contains(&hash) {
            continue;
        }
        seen.insert(hash.clone());
        let data = client.download_object(&hash).await?;
        let (dir, file) = hash.split_at(2);
        let dir_path = objects_dir.join(dir);
        fs::create_dir_all(&dir_path)?;
        let file_path = dir_path.join(file);
        fs::write(&file_path, &data)?;
        // If commit or tree, queue referenced objects
        let obj: Object = serde_json::from_slice(&data).unwrap_or_else(|_| Object::new("blob".to_string(), String::new()));
        if obj.is_commit() {
            let commit: crate::core::commit::Commit = serde_json::from_str(&obj.data)?;
            to_download.extend(commit.parent_ids.clone());
            to_download.push(commit.tree_id.clone());
        } else if obj.is_tree() {
            let tree: crate::core::object::Tree = serde_json::from_str(&obj.data)?;
            for entry in tree.entries {
                to_download.push(entry.object_id);
            }
        }
    }
    pb.inc(1);

    pb.set_message("Setting up repository...");
    let hx_dir = path.join(".helix");
    fs::create_dir_all(&hx_dir)?;
    fs::write(hx_dir.join("HEAD"), "main")?;
    fs::write(
        hx_dir.join("branches.json"),
        serde_json::to_string_pretty(
            &serde_json::json!({"main": {"name": "main", "head_commit": head, "upstream": null, "created_at": chrono::Utc::now(), "last_updated": chrono::Utc::now()}}),
        )?,
    )?;
    pb.inc(1);

    pb.set_message("Checking out files...");
    // Open the repo and check out the latest commit
    let mut repo = Repository::open(path.to_str().unwrap())?;
    if let Some(branch) = repo.branches.get("main") {
        if let Some(commit_id) = branch.get_head_commit() {
            let commit = repo.get_commit_object(commit_id)?;
            let tree_obj = Object::load(&repo.get_objects_dir(), &commit.tree_id)?;
            let tree = crate::core::object::Tree::from_object(&tree_obj)?;
            for entry in tree.entries {
                if entry.object_type == "blob" {
                    let blob = Object::load(&repo.get_objects_dir(), &entry.object_id)?;
                    fs::write(path.join(&entry.name), &blob.data)?;
                }
            }
        }
    }
    pb.finish_with_message("Repository cloned successfully!");
    println!("\n{}", "Repository cloned successfully!".green().bold());
    println!("Location: {}", path.display().to_string().cyan());
    println!("Source: {}", url.magenta());
    println!("Current branch: {}", "main".yellow().bold());
    Ok(())
}
