use crate::core::commit::Commit;
use crate::core::object::Object;
use crate::core::{branch::Branch, index::Index, remote::Remote};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub path: PathBuf,
    pub git_dir: PathBuf,
    pub config: RepositoryConfig,
    pub index: Index,
    pub branches: HashMap<String, Branch>,
    pub current_branch: String,
    pub remotes: HashMap<String, Remote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryConfig {
    pub name: String,
    pub description: Option<String>,
    pub author: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Repository {
    pub fn new(path: &Path) -> Result<Self> {
        let git_dir = path.join(".helix");
        let config = RepositoryConfig {
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            description: None,
            author: std::env::var("HX_AUTHOR").unwrap_or_else(|_| "Unknown".to_string()),
            email: std::env::var("HX_EMAIL").unwrap_or_else(|_| "unknown@example.com".to_string()),
            created_at: chrono::Utc::now(),
        };

        Ok(Self {
            path: path.to_path_buf(),
            git_dir,
            config,
            index: Index::new(),
            branches: HashMap::new(),
            current_branch: "main".to_string(),
            remotes: HashMap::new(),
        })
    }

    pub fn open(path: &str) -> Result<Self> {
        let path = Path::new(path);
        let git_dir = path.join(".helix");

        if !git_dir.exists() {
            anyhow::bail!("Not a Helix repository");
        }

        let config_path = git_dir.join("config.json");
        let config_data = fs::read_to_string(&config_path)?;
        let config: RepositoryConfig = serde_json::from_str(&config_data)?;

        let index_path = git_dir.join("index.json");
        let index = if index_path.exists() {
            serde_json::from_str(&fs::read_to_string(&index_path).context("Failed to read index")?)?
        } else {
            Index::new()
        };

        let branches_path = git_dir.join("branches.json");
        let branches: HashMap<String, Branch> = if branches_path.exists() {
            serde_json::from_str(
                &fs::read_to_string(&branches_path).context("Failed to read branches")?,
            )?
        } else {
            let mut map = HashMap::new();
            map.insert("main".to_string(), Branch::new("main"));
            map
        };

        let current_branch_path = git_dir.join("HEAD");
        let current_branch = if current_branch_path.exists() {
            fs::read_to_string(&current_branch_path)
                .context("Failed to read HEAD")?
                .trim()
                .to_string()
        } else {
            "main".to_string()
        };

        let remotes_path = git_dir.join("remotes.json");
        let remotes: HashMap<String, Remote> = if remotes_path.exists() {
            serde_json::from_str(
                &fs::read_to_string(&remotes_path).context("Failed to read remotes")?,
            )?
        } else {
            HashMap::new()
        };

        Ok(Self {
            path: path.to_path_buf(),
            git_dir,
            config,
            index,
            branches,
            current_branch,
            remotes,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        // Create .helix directory if it doesn't exist
        fs::create_dir_all(&self.git_dir)?;

        // Save config
        let config_path = self.git_dir.join("config.json");
        fs::write(&config_path, serde_json::to_string_pretty(&self.config)?)?;

        // Save index
        let index_path = self.git_dir.join("index.json");
        fs::write(&index_path, serde_json::to_string_pretty(&self.index)?)?;

        // Save branches
        let branches_path = self.git_dir.join("branches.json");
        fs::write(
            &branches_path,
            serde_json::to_string_pretty(&self.branches)?,
        )?;

        // Save current branch
        let head_path = self.git_dir.join("HEAD");
        fs::write(&head_path, &self.current_branch)?;

        // Save remotes
        let remotes_path = self.git_dir.join("remotes.json");
        fs::write(&remotes_path, serde_json::to_string_pretty(&self.remotes)?)?;

        Ok(())
    }

    pub fn get_current_branch(&self) -> Option<&Branch> {
        self.branches.get(&self.current_branch)
    }

    pub fn get_current_branch_mut(&mut self) -> Option<&mut Branch> {
        self.branches.get_mut(&self.current_branch)
    }

    pub fn create_branch(&mut self, name: &str) -> Result<()> {
        if self.branches.contains_key(name) {
            anyhow::bail!("Branch '{}' already exists", name);
        }

        let new_branch = Branch::new(name);
        self.branches.insert(name.to_string(), new_branch);
        self.save()?;

        Ok(())
    }

    pub fn checkout_branch(&mut self, name: &str) -> Result<()> {
        if !self.branches.contains_key(name) {
            anyhow::bail!("Branch '{}' does not exist", name);
        }

        self.current_branch = name.to_string();
        self.save()?;

        Ok(())
    }

    pub fn add_remote(&mut self, name: &str, url: &str) -> Result<()> {
        let remote = Remote::new(name, url);
        self.remotes.insert(name.to_string(), remote);
        self.save()?;

        Ok(())
    }

    pub fn get_objects_dir(&self) -> PathBuf {
        self.git_dir.join("objects")
    }

    pub fn get_refs_dir(&self) -> PathBuf {
        self.git_dir.join("refs")
    }

    pub fn get_commit_object(&self, commit_id: &str) -> anyhow::Result<Commit> {
        let obj = Object::load(&self.get_objects_dir(), commit_id)?;
        Commit::from_object(&obj).map_err(|e| anyhow::anyhow!(e))
    }

    pub fn set_head(&mut self, commit_id: &str) -> anyhow::Result<()> {
        if let Some(branch) = self.branches.get_mut(&self.current_branch) {
            branch.set_head_commit(commit_id.to_string());
            self.save()?;
            Ok(())
        } else {
            anyhow::bail!("Current branch not found")
        }
    }
}
