use crate::core::commit::Commit;
use crate::core::repository::Repository;
use crate::utils::pack::create_thin_pack;
use crate::utils::remote_client::{NegotiationRequest, PushRequest, RemoteClient};
use anyhow::{Context, Result};
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::{HashMap, HashSet};

pub async fn push_changes(repo: &Repository) -> Result<()> {
    let pb = ProgressBar::new(5);
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {wide_msg}")
            .unwrap(),
    );

    pb.set_message("Initializing push...");

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

    // Verify local commits before push
    pb.set_message("Verifying local commits...");
    if let Some(branch) = repo.get_current_branch() {
        if let Some(head_commit) = branch.get_head_commit() {
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
                    "Push aborted: unsigned or invalid commits detected!"
                        .red()
                        .bold()
                );
                return Ok(());
            }
        }
    }

    pb.inc(1);

    // Collect local objects
    pb.set_message("Collecting local objects...");
    let local_objects = collect_local_objects(repo)?;
    let local_object_hashes: HashSet<String> = local_objects.keys().cloned().collect();

    // Get remote refs and objects
    pb.set_message("Fetching remote state...");
    let _remote_refs = client.get_refs().await
        .with_context(|| "Failed to fetch remote refs")?;
    
    let remote_object_hashes = client.get_all_object_hashes().await
        .with_context(|| "Failed to fetch remote object hashes")?;
    let remote_objects_set: HashSet<String> = remote_object_hashes.iter().cloned().collect();
    let remote_objects_map: HashMap<String, Vec<u8>> = remote_object_hashes.clone().into_iter().map(|h| (h, Vec::new())).collect();

    pb.inc(1);

    // Determine what needs to be pushed
    let missing_objects: HashSet<String> = local_object_hashes
        .difference(&remote_objects_set)
        .cloned()
        .collect();

    if missing_objects.is_empty() {
        println!("{}", "No new objects to push".green());
            return Ok(());
        }

    // Create negotiation request
    pb.set_message("Negotiating with remote...");
    let current_branch = &repo.current_branch;
    let wants = vec![current_branch.clone()];
    let haves = remote_object_hashes;

    let negotiation_request = NegotiationRequest {
        wants,
        haves,
        shallow: Vec::new(),
        deepen_since: None,
        deepen_not: None,
        filter: None,
    };

    // Perform negotiation
    let _negotiation_response = client.negotiate_fetch(&negotiation_request).await
        .with_context(|| "Failed to negotiate with remote")?;

    pb.inc(1);

    // Build and upload pack
    pb.set_message("Building and uploading pack...");
    let pack = create_thin_pack(&local_objects, &remote_objects_map);
    let pack_data = pack.to_bytes()
        .with_context(|| "Failed to serialize pack")?;

    client.upload_pack(&pack_data).await
        .with_context(|| "Failed to upload pack")?;

    // Update remote refs
    pb.set_message("Updating remote refs...");
    let mut refs_to_update = HashMap::new();
    
    if let Some(branch) = repo.get_current_branch() {
        if let Some(head_commit) = branch.get_head_commit() {
            refs_to_update.insert(format!("refs/heads/{}", current_branch), head_commit.to_string());
        }
    }

    let push_request = PushRequest {
        refs: refs_to_update,
        objects: missing_objects.into_iter().collect(),
        force: false,
    };

    let push_response = client.negotiate_push(&push_request).await
        .with_context(|| "Failed to push refs")?;

    pb.finish_with_message("Push completed successfully!");

    // Report results
    println!("\n{}", "Push completed successfully!".green().bold());
    println!("Objects uploaded: {}", pack.header.object_count.to_string().cyan());
    println!("Pack size: {} bytes", pack_data.len().to_string().cyan());
    println!("Remote: {}", remote.url.cyan());
    println!("Branch: {}", current_branch.yellow().bold());

    if !push_response.updated_refs.is_empty() {
        println!("Updated refs: {}", push_response.updated_refs.join(", ").green());
    }

    if !push_response.rejected_refs.is_empty() {
        println!("Rejected refs: {}", push_response.rejected_refs.join(", ").red());
    }

    if let Some(error) = push_response.error {
        println!("Warning: {}", error.yellow());
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

pub async fn push_with_options(
    repo: &Repository,
    force: bool,
    remote_name: Option<&str>,
    _refspec: Option<&str>,
) -> Result<()> {
    let remote_name = remote_name.unwrap_or("origin");
    
    let remote = match repo.remotes.get(remote_name) {
        Some(remote) => remote,
        None => {
            println!("{}", format!("No '{}' remote configured", remote_name).yellow());
            return Ok(());
        }
    };

    let _client = RemoteClient::new(&remote.url);

    // Enhanced push with options
    if force {
        println!("{}", "Force push requested - this may overwrite remote changes!".yellow().bold());
    }

    // TODO: Implement refspec parsing and filtering
    // TODO: Implement force push logic
    // TODO: Implement dry-run mode

    // For now, delegate to the main push function
    push_changes(repo).await
}
