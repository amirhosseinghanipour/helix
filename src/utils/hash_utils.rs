use sha2::{Digest, Sha256};

pub fn calculate_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub fn calculate_file_hash(path: &std::path::Path) -> anyhow::Result<String> {
    let content = std::fs::read(path)?;
    Ok(calculate_hash(&content))
}

pub fn get_short_hash(hash: &str) -> String {
    hash.chars().take(8).collect()
}
