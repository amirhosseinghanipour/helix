use crate::core::object::Object;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub parent_ids: Vec<String>,
    pub tree_id: String,
    pub author: String,
    pub email: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub files: HashMap<String, FileChange>,
    pub public_key: Option<Vec<u8>>, // Ed25519 public key
    pub signature: Option<Vec<u8>>,  // Ed25519 signature
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub content_hash: String,
    pub size: u64,
    pub mode: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed { old_path: String },
}

impl Commit {
    pub fn new(
        parent_ids: Vec<String>,
        tree_id: String,
        author: String,
        email: String,
        message: String,
        files: HashMap<String, FileChange>,
        keypair: Option<&SigningKey>,
    ) -> Self {
        let timestamp = chrono::Utc::now();
        let id = Self::calculate_id(&parent_ids, &tree_id, &author, &email, &message, &timestamp);
        let (public_key, signature) = if let Some(kp) = keypair {
            let sig = kp.sign(id.as_bytes());
            (
                Some(kp.verifying_key().to_bytes().to_vec()),
                Some(sig.to_bytes().to_vec()),
            )
        } else {
            (None, None)
        };
        Self {
            id,
            parent_ids,
            tree_id,
            author,
            email,
            message,
            timestamp,
            files,
            public_key,
            signature,
        }
    }

    pub fn calculate_id(
        parent_ids: &[String],
        tree_id: &str,
        author: &str,
        email: &str,
        message: &str,
        timestamp: &chrono::DateTime<chrono::Utc>,
    ) -> String {
        let mut hasher = Sha256::new();
        let commit_data = format!(
            "tree {}\nparents {}\nauthor {} <{}> {}\n\n{}",
            tree_id,
            parent_ids.join(","),
            author,
            email,
            timestamp.timestamp(),
            message
        );
        hasher.update(commit_data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    #[allow(dead_code)]
    pub fn sign(&mut self, keypair: &SigningKey) {
        let sig = keypair.sign(self.id.as_bytes());
        self.public_key = Some(keypair.verifying_key().to_bytes().to_vec());
        self.signature = Some(sig.to_bytes().to_vec());
    }

    pub fn verify(&self) -> bool {
        if let (Some(pk_bytes), Some(sig_bytes)) = (&self.public_key, &self.signature) {
            if let (Ok(pk_array), Ok(sig_array)) = (
                pk_bytes.as_slice().try_into(),
                sig_bytes.as_slice().try_into(),
            ) {
                if let Ok(pk) = VerifyingKey::from_bytes(pk_array) {
                    let sig = Signature::from_bytes(sig_array);
                    return pk.verify(self.id.as_bytes(), &sig).is_ok();
                }
            }
        }
        false
    }

    /// Recursively verify this commit and all ancestors (full ancestry).
    pub fn verify_ancestry<F>(
        repo: &crate::core::repository::Repository,
        commit_id: &str,
        mut on_commit: F,
    ) -> bool
    where
        F: FnMut(&Commit, bool),
    {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![commit_id.to_string()];
        let mut all_valid = true;
        while let Some(cid) = stack.pop() {
            if !visited.insert(cid.clone()) {
                continue;
            }
            let obj = match crate::core::object::Object::load(&repo.get_objects_dir(), &cid) {
                Ok(o) => o,
                Err(_) => {
                    all_valid = false;
                    continue;
                }
            };
            let commit = match Commit::from_object(&obj) {
                Ok(c) => c,
                Err(_) => {
                    all_valid = false;
                    continue;
                }
            };
            let valid = commit.verify();
            on_commit(&commit, valid);
            if !valid {
                all_valid = false;
            }
            for parent in &commit.parent_ids {
                stack.push(parent.clone());
            }
        }
        all_valid
    }

    pub fn to_object(&self) -> Object {
        Object::new("commit".to_string(), serde_json::to_string(self).unwrap())
    }

    pub fn from_object(object: &Object) -> Result<Self, serde_json::Error> {
        serde_json::from_str(&object.data)
    }

    pub fn get_short_id(&self) -> String {
        crate::utils::hash_utils::get_short_hash(&self.id)
    }

    pub fn get_files(&self) -> &HashMap<String, FileChange> {
        &self.files
    }

    pub fn get_file_change(&self, path: &str) -> Option<&FileChange> {
        self.files.get(path)
    }

    #[allow(dead_code)]
    pub fn has_file(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }
}

impl FileChange {
    pub fn new(
        path: String,
        change_type: ChangeType,
        content_hash: String,
        size: u64,
        mode: u32,
    ) -> Self {
        Self {
            path,
            change_type,
            content_hash,
            size,
            mode,
        }
    }

    #[allow(dead_code)]
    pub fn is_added(&self) -> bool {
        matches!(self.change_type, ChangeType::Added)
    }

    #[allow(dead_code)]
    pub fn is_modified(&self) -> bool {
        matches!(self.change_type, ChangeType::Modified)
    }

    #[allow(dead_code)]
    pub fn is_deleted(&self) -> bool {
        matches!(self.change_type, ChangeType::Deleted)
    }

    #[allow(dead_code)]
    pub fn is_renamed(&self) -> bool {
        matches!(self.change_type, ChangeType::Renamed { .. })
    }
}
