use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    pub user: Option<UserConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UserConfig {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl GlobalConfig {
    pub fn config_path() -> PathBuf {
        dirs::home_dir().unwrap().join(".helixconfig")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(GlobalConfig::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn set_user_name(&mut self, name: String) {
        if self.user.is_none() {
            self.user = Some(UserConfig::default());
        }
        self.user.as_mut().unwrap().name = Some(name);
    }

    pub fn set_user_email(&mut self, email: String) {
        if self.user.is_none() {
            self.user = Some(UserConfig::default());
        }
        self.user.as_mut().unwrap().email = Some(email);
    }

    pub fn get_user_name(&self) -> Option<&str> {
        self.user.as_ref()?.name.as_deref()
    }

    pub fn get_user_email(&self) -> Option<&str> {
        self.user.as_ref()?.email.as_deref()
    }
} 