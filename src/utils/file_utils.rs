use anyhow::Result;
use std::{fs, os::unix::fs::PermissionsExt, path::Path};

pub fn read_file_content(path: &Path) -> Result<Vec<u8>> {
    Ok(fs::read(path)?)
}

pub fn write_file_content(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(fs::write(path, content)?)
}

pub fn get_file_mode(path: &Path) -> Result<u32> {
    let metadata = fs::metadata(path)?;
    let permissions = metadata.permissions();

    let mut mode = 0o644;
    if permissions.readonly() {
        mode = 0o444;
    }

    Ok(mode)
}

pub fn is_executable(path: &Path) -> Result<bool> {
    let metadata = fs::metadata(path)?;
    let permissions = metadata.permissions();
    Ok(permissions.mode() & 0o111 != 0)
}
