use crate::utils::remote_client::RemoteClient;
use anyhow::Result;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::Path;

pub async fn clone_repository(url: &str, path: &Path) -> Result<()> {
    let pb = ProgressBar::new(3);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .unwrap(),
    );

    pb.set_message("Cloning repository...");
    // Create the target directory
    fs::create_dir_all(path)?;
    pb.inc(1);
    pb.set_message("Downloading objects and refs...");
    // Download all refs (for now, just 'main')
    let client = RemoteClient::new(url);
    let head = client.get_ref("main").await?;
    // Download all objects reachable from head (BFS)
            let objects_dir = path.join(".helix/objects");
    let mut to_download = vec![head.clone()];
    let mut downloaded = 0;
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
        downloaded += 1;
        // If commit or tree, queue referenced objects
        let obj: crate::core::object::Object = serde_json::from_slice(&data).unwrap_or_else(|_| {
            crate::core::object::Object::new("blob".to_string(), String::new())
        });
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
    // Write HEAD ref
            let hx_dir = path.join(".helix");
    fs::create_dir_all(&hx_dir)?;
    fs::write(hx_dir.join("HEAD"), "main")?;
    fs::write(
        hx_dir.join("branches.json"),
        serde_json::to_string_pretty(
            &serde_json::json!({"main": {"name": "main", "head_commit": head, "upstream": null, "created_at": chrono::Utc::now(), "last_updated": chrono::Utc::now()}}),
        )?,
    )?;
    pb.finish_with_message("Repository cloned successfully!");
    println!("\n{}", "Repository cloned successfully!".green().bold());
    println!("Location: {}", path.display().to_string().cyan());
    println!("Source: {}", url.magenta());
    println!("Downloaded: {} objects", downloaded.to_string().cyan());
    Ok(())
}
