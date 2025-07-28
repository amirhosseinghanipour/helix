use crate::core::object::Object;
use crate::core::repository::Repository;
use anyhow::Result;
use colored::*;
use diffy::merge;

pub async fn merge_branch(repo: &mut Repository, branch_name: &str) -> Result<()> {
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
            "Merging branch '{}' into '{}'",
            branch_name, repo.current_branch
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
        // For each file, do a line-based three-way merge using diff3
        let mut conflicts = 0;
        for (path, _) in theirs_commit.get_files() {
            let base_blob = base_commit
                .get_file_change(path)
                .map(|fc| fc.content_hash.clone());
            let ours_blob = ours_commit
                .get_file_change(path)
                .map(|fc| fc.content_hash.clone());
            let theirs_blob = theirs_commit
                .get_file_change(path)
                .map(|fc| fc.content_hash.clone());
            if let (Some(ours_blob), Some(theirs_blob)) = (ours_blob, theirs_blob) {
                let ours_obj = Object::load(&repo.get_objects_dir(), &ours_blob)?;
                let theirs_obj = Object::load(&repo.get_objects_dir(), &theirs_blob)?;
                let base_obj =
                    base_blob.and_then(|b| Object::load(&repo.get_objects_dir(), &b).ok());
                let base_content = base_obj.as_ref().map(|o| o.data.as_str()).unwrap_or("");
                let ours_content = ours_obj.data.as_str();
                let theirs_content = theirs_obj.data.as_str();
                let merged = diff3_merge(
                    base_content,
                    ours_content,
                    theirs_content,
                    std::path::Path::new(path),
                );
                if merged.contains("<<<<<<<") {
                    conflicts += 1;
                }
                std::fs::write(path, merged)?;
            }
        }
        if conflicts > 0 {
            println!(
                "{}",
                format!("Merge completed with {} conflicts.", conflicts)
                    .yellow()
                    .bold()
            );
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
