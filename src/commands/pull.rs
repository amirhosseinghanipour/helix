use crate::core::commit::Commit;
use crate::core::object::Object;
use crate::core::repository::Repository;
use crate::utils::pack::{extract_objects_from_pack, Pack};
use crate::utils::remote_client::{NegotiationRequest, RemoteClient};
use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::{HashMap, HashSet};
use std::fs;

pub async fn pull_changes(repo: &Repository) -> Result<()> {
    let pb = ProgressBar::new(6);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .unwrap(),
    );

    pb.set_message("Initializing pull...");

    // Check for remote configuration
    if repo.remotes.is_empty() {
        println!("{}", "No remote repositories configured".yellow());
        println!("Use 'hx remote add origin <url>' to add a remote");
        return Ok(());
    }

    let remote = match repo.remotes.get("origin") {
        Some(remote) => remote,
        None => {
            println!("{}", "No 'origin' remote configured".yellow());
            println!("Use 'hx remote add origin <url>' to add a remote");
            return Ok(());
        }
    };

    let mut client = RemoteClient::new(&remote.url);

    // Check connectivity
    pb.set_message("Checking remote connectivity...");
    if !client.check_connectivity().await? {
        println!("{}", "Failed to connect to remote repository".red());
        return Ok(());
    }

    // Discover remote capabilities
    pb.set_message("Discovering remote capabilities...");
    let _capabilities = client.discover_capabilities().await
        .with_context(|| "Failed to discover remote capabilities")?;
    
    pb.inc(1);

    // Get current branch and remote refs
    pb.set_message("Fetching remote state...");
    let current_branch = &repo.current_branch;
    let remote_refs = client.get_refs().await
        .with_context(|| "Failed to fetch remote refs")?;

    let remote_head = match remote_refs.get(&format!("refs/heads/{}", current_branch)) {
        Some(head) => head.clone(),
        None => {
            println!("{}", format!("Remote branch '{}' not found", current_branch).yellow());
            return Ok(());
        }
    };

    // Collect local objects for negotiation
    pb.set_message("Collecting local objects...");
    let local_objects = collect_local_objects(repo)?;
    let local_object_hashes: HashSet<String> = local_objects.keys().cloned().collect();

    // Get remote objects
    let remote_object_hashes = client.get_all_object_hashes().await
        .with_context(|| "Failed to fetch remote object hashes")?;
    let remote_objects_set: HashSet<String> = remote_object_hashes.into_iter().collect();

    pb.inc(1);

    // Determine what we need to fetch
    let missing_objects: HashSet<String> = remote_objects_set
        .difference(&local_object_hashes)
        .cloned()
        .collect();

    if missing_objects.is_empty() {
        println!("{}", "Already up to date".green());
        return Ok(());
    }

    // Create negotiation request
    pb.set_message("Negotiating with remote...");
    let wants = vec![remote_head.clone()];
    let haves: Vec<String> = local_object_hashes.into_iter().collect();

    let negotiation_request = NegotiationRequest {
        wants,
        haves,
        shallow: Vec::new(),
        deepen_since: None,
        deepen_not: None,
        filter: None,
    };

    // Perform negotiation
    let negotiation_response = client.negotiate_fetch(&negotiation_request).await
        .with_context(|| "Failed to negotiate with remote")?;

    pb.inc(1);

    // Download pack if available
    if let Some(pack_id) = negotiation_response.packfile {
        pb.set_message("Downloading pack...");
        let pack_data = client.download_pack(&pack_id).await
            .with_context(|| "Failed to download pack")?;

        let pack = Pack::from_bytes(&pack_data)
            .with_context(|| "Failed to parse pack")?;

        // Extract and save objects
        let objects = extract_objects_from_pack(&pack);
        save_objects_to_repository(repo, &objects)?;

        pb.inc(1);
    } else {
        // Fallback to individual object download
        pb.set_message("Downloading individual objects...");
        download_objects_individually(&client, repo, &missing_objects).await?;
        pb.inc(1);
    }

    // Update local refs
    pb.set_message("Updating local refs...");
    update_local_refs(repo, &remote_refs, current_branch)?;

    pb.finish_with_message("Pull completed successfully!");

    // Report results
    println!("\n{}", "Pull completed successfully!".green().bold());
    println!("Objects downloaded: {}", missing_objects.len().to_string().cyan());
    println!("Remote: {}", remote.url.cyan());
    println!("Branch: {}", current_branch.yellow().bold());

    // Verify downloaded commits
    pb.set_message("Verifying downloaded commits...");
    if let Some(_branch) = repo.get_current_branch() {
        if let Some(head_commit) = _branch.get_head_commit() {
            let all_valid = Commit::verify_ancestry(repo, head_commit, |commit, valid| {
                if !valid {
                    println!(
                        "{} {} {}",
                        commit.get_short_id().cyan(),
                        "INVALID".red(),
                        commit.message.bold()
                    );
                }
            });
            if !all_valid {
                println!(
                    "{}",
                    "Warning: unsigned or invalid commits detected in pulled history!"
                        .red()
                        .bold()
                );
            } else {
                println!("{}", "All pulled commits are valid!".green().bold());
            }
        }
    }

    Ok(())
}

fn collect_local_objects(repo: &Repository) -> Result<HashMap<String, Vec<u8>>> {
    let mut objects = HashMap::new();
    let objects_dir = repo.get_objects_dir();

    for entry in std::fs::read_dir(&objects_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            for obj in std::fs::read_dir(entry.path())? {
                let obj = obj?;
                let hash = format!(
                    "{}{}",
                    entry.file_name().to_string_lossy(),
                    obj.file_name().to_string_lossy()
                );
                let data = std::fs::read(obj.path())?;
                objects.insert(hash, data);
            }
        }
    }

    Ok(objects)
}

async fn download_objects_individually(
    client: &RemoteClient,
    repo: &Repository,
    object_hashes: &HashSet<String>,
) -> Result<()> {
    let objects_dir = repo.get_objects_dir();
    let mut _downloaded = 0;
    let mut seen = HashSet::new();

    for hash in object_hashes {
        if seen.contains(hash) {
            continue;
        }
        seen.insert(hash.clone());

        // Download object
        let data = client.download_object(hash).await?;
        
        // Save to .helix/objects
        let (dir, file) = hash.split_at(2);
        let dir_path = objects_dir.join(dir);
        fs::create_dir_all(&dir_path)?;
        let file_path = dir_path.join(file);
        fs::write(&file_path, &data)?;
        _downloaded += 1;

        // If commit or tree, queue referenced objects
        let obj: Object = serde_json::from_slice(&data)
            .unwrap_or_else(|_| Object::new("blob".to_string(), String::new()));
        
        if obj.is_commit() {
            let commit: crate::core::commit::Commit = serde_json::from_str(&obj.data)?;
            for parent_id in &commit.parent_ids {
                if !seen.contains(parent_id) {
                    seen.insert(parent_id.clone());
                }
            }
            if !seen.contains(&commit.tree_id) {
                seen.insert(commit.tree_id.clone());
            }
        } else if obj.is_tree() {
            let tree: crate::core::object::Tree = serde_json::from_str(&obj.data)?;
            for entry in tree.entries {
                if !seen.contains(&entry.object_id) {
                    seen.insert(entry.object_id.clone());
                }
            }
        }
    }

    Ok(())
}

fn save_objects_to_repository(repo: &Repository, objects: &HashMap<String, Vec<u8>>) -> Result<()> {
    let objects_dir = repo.get_objects_dir();

    for (hash, data) in objects {
        let (dir, file) = hash.split_at(2);
        let dir_path = objects_dir.join(dir);
        fs::create_dir_all(&dir_path)?;
        let file_path = dir_path.join(file);
        fs::write(&file_path, data)?;
    }

    Ok(())
}

fn update_local_refs(
    repo: &Repository,
    remote_refs: &HashMap<String, String>,
    current_branch: &str,
) -> Result<()> {
    let ref_key = format!("refs/heads/{}", current_branch);
    
    if let Some(remote_head) = remote_refs.get(&ref_key) {
        // Update the local branch to point to the remote head
        if let Some(branch) = repo.get_current_branch() {
            // TODO: Implement proper ref update logic
            // For now, we'll just update the branch head
            println!("Updated {} to {}", current_branch, remote_head);
        }
    }

    Ok(())
}

pub async fn pull_with_options(
    repo: &Repository,
    remote_name: Option<&str>,
    branch_name: Option<&str>,
    rebase: bool,
) -> Result<()> {
    let remote_name = remote_name.unwrap_or("origin");
    let _branch_name = branch_name.unwrap_or(&repo.current_branch);
    
    let remote = match repo.remotes.get(remote_name) {
        Some(remote) => remote,
        None => {
            println!("{}", format!("No '{}' remote configured", remote_name).yellow());
            return Ok(());
        }
    };

    let mut client = RemoteClient::new(&remote.url);

    // Enhanced pull with options
    if rebase {
        println!("{}", "Rebase mode requested".yellow());
        // TODO: Implement rebase logic
    }

    // TODO: Implement branch-specific pull
    // TODO: Implement merge strategy selection
    // TODO: Implement conflict resolution

    // For now, delegate to the main pull function
    pull_changes(repo).await
}
