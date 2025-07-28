use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use url::Url;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    None,
    Token(String),
    Basic { username: String, password: String },
    SSH { key_path: Option<PathBuf> },
    OAuth2 { token: String, refresh_token: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub method: AuthMethod,
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
}

impl AuthConfig {
    pub fn new(host: &str) -> Self {
        Self {
            method: AuthMethod::None,
            host: host.to_string(),
            port: None,
            username: None,
        }
    }

    pub fn with_token(mut self, token: &str) -> Self {
        self.method = AuthMethod::Token(token.to_string());
        self
    }

    pub fn with_basic_auth(mut self, username: &str, password: &str) -> Self {
        self.method = AuthMethod::Basic {
            username: username.to_string(),
            password: password.to_string(),
        };
        self.username = Some(username.to_string());
        self
    }

    pub fn with_ssh(mut self, key_path: Option<PathBuf>) -> Self {
        self.method = AuthMethod::SSH { key_path };
        self
    }

    pub fn with_oauth2(mut self, token: &str, refresh_token: Option<&str>) -> Self {
        self.method = AuthMethod::OAuth2 {
            token: token.to_string(),
            refresh_token: refresh_token.map(|t| t.to_string()),
        };
        self
    }
}

pub struct AuthManager {
    configs: HashMap<String, AuthConfig>,
    config_file: PathBuf,
}

impl AuthManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("helix");
        std::fs::create_dir_all(&config_dir)?;
        
        let config_file = config_dir.join("auth.json");
        
        let configs = if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            HashMap::new()
        };

        Ok(Self {
            configs,
            config_file,
        })
    }

    pub fn add_config(&mut self, host: &str, config: AuthConfig) -> Result<()> {
        self.configs.insert(host.to_string(), config);
        self.save_configs()?;
        Ok(())
    }

    pub fn get_config(&self, host: &str) -> Option<&AuthConfig> {
        self.configs.get(host)
    }

    pub fn remove_config(&mut self, host: &str) -> Result<()> {
        self.configs.remove(host);
        self.save_configs()?;
        Ok(())
    }

    fn save_configs(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.configs)?;
        std::fs::write(&self.config_file, content)?;
        Ok(())
    }

    pub fn get_auth_headers(&self, url: &str) -> Result<HashMap<String, String>> {
        let host = extract_host_from_url(url)?;
        
        if let Some(config) = self.get_config(&host) {
            match &config.method {
                AuthMethod::None => Ok(HashMap::new()),
                AuthMethod::Token(token) => {
                    let mut headers = HashMap::new();
                    headers.insert("Authorization".to_string(), format!("Bearer {}", token));
                    Ok(headers)
                }
                AuthMethod::Basic { username, password } => {
                    let mut headers = HashMap::new();
                    let credentials = BASE64.encode(format!("{}:{}", username, password));
                    headers.insert("Authorization".to_string(), format!("Basic {}", credentials));
                    Ok(headers)
                }
                AuthMethod::SSH { .. } => {
                    // SSH auth is handled differently, not via HTTP headers
                    Ok(HashMap::new())
                }
                AuthMethod::OAuth2 { token, .. } => {
                    let mut headers = HashMap::new();
                    headers.insert("Authorization".to_string(), format!("Bearer {}", token));
                    Ok(headers)
                }
            }
        } else {
            Ok(HashMap::new())
        }
    }

    pub fn is_ssh_url(&self, url: &str) -> bool {
        url.starts_with("ssh://") || url.starts_with("git@")
    }

    pub fn setup_ssh_connection(&self, url: &str) -> Result<()> {
        let host = extract_host_from_url(url)?;
        
        if let Some(config) = self.get_config(&host) {
            if let AuthMethod::SSH { key_path } = &config.method {
                // Test SSH connection
                let mut cmd = Command::new("ssh");
                cmd.arg("-T");
                cmd.arg("-o");
                cmd.arg("BatchMode=yes");
                
                if let Some(path) = key_path {
                    cmd.arg("-i");
                    cmd.arg(path);
                }
                
                cmd.arg(&host);
                
                let output = cmd.output()
                    .with_context(|| format!("Failed to test SSH connection to {}", host))?;
                
                if !output.status.success() {
                    return Err(anyhow::anyhow!(
                        "SSH connection failed: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
        }
        
        Ok(())
    }

    pub fn refresh_oauth_token(&mut self, host: &str) -> Result<()> {
        if let Some(config) = self.configs.get_mut(host) {
            if let AuthMethod::OAuth2 { token: _token, refresh_token } = &mut config.method {
                if let Some(_refresh) = refresh_token {
                    // TODO: Implement OAuth2 token refresh logic
                    // This would typically involve making a request to the OAuth2 provider
                    println!("Refreshing OAuth2 token for {}", host);
                }
            }
        }
        Ok(())
    }
}

fn extract_host_from_url(url: &str) -> Result<String> {
    if url.starts_with("http://") || url.starts_with("https://") {
        let url = Url::parse(url)
            .with_context(|| format!("Invalid URL: {}", url))?;
        Ok(url.host_str().unwrap_or("").to_string())
    } else if url.starts_with("ssh://") {
        let url = Url::parse(url)
            .with_context(|| format!("Invalid SSH URL: {}", url))?;
        Ok(url.host_str().unwrap_or("").to_string())
    } else if url.starts_with("git@") {
        // Parse git@host:path format
        let parts: Vec<&str> = url.split(':').collect();
        if parts.len() >= 2 {
            let host_part = parts[0];
            if host_part.starts_with("git@") {
                return Ok(host_part[4..].to_string());
            }
        }
        Err(anyhow::anyhow!("Invalid git SSH URL format: {}", url))
    } else {
        Err(anyhow::anyhow!("Unsupported URL format: {}", url))
    }
}

pub fn detect_auth_method_from_url(url: &str) -> AuthMethod {
    if url.starts_with("ssh://") || url.starts_with("git@") {
        AuthMethod::SSH { key_path: None }
    } else if url.starts_with("https://") {
        AuthMethod::None // Will be configured separately
    } else {
        AuthMethod::None
    }
}

pub fn get_default_ssh_key_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let key_path = home.join(".ssh").join("id_ed25519");
    if key_path.exists() {
        Some(key_path)
    } else {
        let rsa_key_path = home.join(".ssh").join("id_rsa");
        if rsa_key_path.exists() {
            Some(rsa_key_path)
        } else {
            None
        }
    }
} 