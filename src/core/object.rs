use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    pub id: String,
    pub object_type: String,
    pub data: String,
    pub size: usize,
}

impl Object {
    pub fn new(object_type: String, data: String) -> Self {
        let id = Self::calculate_id(&object_type, &data);
        let size = data.len();

        Self {
            id,
            object_type,
            data,
            size,
        }
    }

    fn calculate_id(object_type: &str, data: &str) -> String {
        let mut hasher = Sha256::new();
        let content = format!("{} {}\0{}", object_type, data.len(), data);
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn save(&self, objects_dir: &Path) -> Result<()> {
        let object_dir = objects_dir.join(&self.id[..2]);
        let object_path = object_dir.join(&self.id[2..]);

        fs::create_dir_all(&object_dir)?;

        let compressed_data = self.compress()?;
        fs::write(&object_path, compressed_data)?;

        Ok(())
    }

    pub fn load(objects_dir: &Path, object_id: &str) -> Result<Self> {
        let object_path = objects_dir.join(&object_id[..2]).join(&object_id[2..]);

        if !object_path.exists() {
            anyhow::bail!("Object {} not found", object_id);
        }

        let compressed_data = fs::read(&object_path)?;
        let data = Self::decompress(&compressed_data)?;

        // Parse the object data
        let parts: Vec<&str> = data.splitn(2, '\0').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid object format");
        }

        let header = parts[0];
        let content = parts[1];

        let header_parts: Vec<&str> = header.split_whitespace().collect();
        if header_parts.len() != 2 {
            anyhow::bail!("Invalid object header");
        }

        let object_type = header_parts[0].to_string();
        let size: usize = header_parts[1].parse().context("Invalid object size")?;

        if content.len() != size {
            anyhow::bail!("Object size mismatch");
        }

        Ok(Self {
            id: object_id.to_string(),
            object_type,
            data: content.to_string(),
            size,
        })
    }

    fn compress(&self) -> Result<Vec<u8>> {
        use flate2::write::DeflateEncoder;
        use flate2::Compression;
        use std::io::Write;

        let content = format!("{} {}\0{}", self.object_type, self.size, self.data);
        let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(content.as_bytes())?;
        Ok(encoder.finish()?)
    }

    fn decompress(data: &[u8]) -> Result<String> {
        use flate2::read::DeflateDecoder;
        use std::io::Read;

        let mut decoder = DeflateDecoder::new(data);
        let mut content = String::new();
        decoder.read_to_string(&mut content)?;
        Ok(content)
    }

    pub fn get_short_id(&self) -> String {
        crate::utils::hash_utils::get_short_hash(&self.id)
    }

    pub fn is_commit(&self) -> bool {
        self.object_type == "commit"
    }

    pub fn is_tree(&self) -> bool {
        self.object_type == "tree"
    }

    #[allow(dead_code)]
    pub fn is_blob(&self) -> bool {
        self.object_type == "blob"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tree {
    pub entries: Vec<TreeEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeEntry {
    pub name: String,
    pub object_id: String,
    pub object_type: String,
    pub mode: u32,
}

impl Tree {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, name: String, object_id: String, object_type: String, mode: u32) {
        self.entries.push(TreeEntry {
            name,
            object_id,
            object_type,
            mode,
        });
    }

    pub fn to_object(&self) -> Object {
        Object::new("tree".to_string(), serde_json::to_string(self).unwrap())
    }

    #[allow(dead_code)]
    pub fn from_object(object: &Object) -> Result<Self, serde_json::Error> {
        serde_json::from_str(&object.data)
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}
