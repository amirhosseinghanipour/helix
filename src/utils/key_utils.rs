use dirs;
use ed25519_dalek::{SigningKey, SECRET_KEY_LENGTH};
use rand::rngs::OsRng;
use rand::RngCore;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn get_key_dir() -> PathBuf {
    if let Some(home) = dirs::home_dir() {
        home.join(".helix/keys/")
    } else {
        PathBuf::from(".helix/keys/")
    }
}

pub fn keypair_path() -> PathBuf {
    get_key_dir().join("ed25519.key")
}

pub fn generate_and_save_keypair() -> std::io::Result<SigningKey> {
    let mut csprng = OsRng;
    let mut secret_bytes = [0u8; SECRET_KEY_LENGTH];
    csprng.fill_bytes(&mut secret_bytes);
    let keypair = SigningKey::from_bytes(&secret_bytes);
    let key_dir = get_key_dir();
    fs::create_dir_all(&key_dir)?;
    let mut file = fs::File::create(keypair_path())?;
    file.write_all(&keypair.to_bytes())?;
    Ok(keypair)
}

pub fn load_keypair() -> std::io::Result<SigningKey> {
    let mut file = fs::File::open(keypair_path())?;
    let mut buf = [0u8; SECRET_KEY_LENGTH];
    file.read_exact(&mut buf)?;
    let keypair = SigningKey::from_bytes(&buf);
    Ok(keypair)
}

pub fn keypair_exists() -> bool {
    keypair_path().exists()
}

pub fn export_keypair(path: &str) -> std::io::Result<()> {
    fs::copy(keypair_path(), path)?;
    Ok(())
}

pub fn import_keypair(path: &str) -> std::io::Result<()> {
    let key_dir = get_key_dir();
    fs::create_dir_all(&key_dir)?;
    fs::copy(path, keypair_path())?;
    Ok(())
}
