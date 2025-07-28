use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remote {
    pub name: String,
    pub url: String,
    pub fetch_url: Option<String>,
    pub push_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_fetch: Option<chrono::DateTime<chrono::Utc>>,
    pub last_push: Option<chrono::DateTime<chrono::Utc>>,
}

impl Remote {
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            fetch_url: None,
            push_url: None,
            created_at: chrono::Utc::now(),
            last_fetch: None,
            last_push: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_urls(name: &str, fetch_url: &str, push_url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: fetch_url.to_string(),
            fetch_url: Some(fetch_url.to_string()),
            push_url: Some(push_url.to_string()),
            created_at: chrono::Utc::now(),
            last_fetch: None,
            last_push: None,
        }
    }

    #[allow(dead_code)]
    pub fn get_fetch_url(&self) -> &str {
        self.fetch_url.as_deref().unwrap_or(&self.url)
    }

    #[allow(dead_code)]
    pub fn get_push_url(&self) -> &str {
        self.push_url.as_deref().unwrap_or(&self.url)
    }

    #[allow(dead_code)]
    pub fn update_last_fetch(&mut self) {
        self.last_fetch = Some(chrono::Utc::now());
    }

    #[allow(dead_code)]
    pub fn update_last_push(&mut self) {
        self.last_push = Some(chrono::Utc::now());
    }

    #[allow(dead_code)]
    pub fn is_origin(&self) -> bool {
        self.name == "origin"
    }

    #[allow(dead_code)]
    pub fn get_age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.created_at
    }

    #[allow(dead_code)]
    pub fn get_last_fetch_age(&self) -> Option<chrono::Duration> {
        self.last_fetch.map(|last| chrono::Utc::now() - last)
    }

    #[allow(dead_code)]
    pub fn get_last_push_age(&self) -> Option<chrono::Duration> {
        self.last_push.map(|last| chrono::Utc::now() - last)
    }
}
