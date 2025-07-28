use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    pub head_commit: Option<String>,
    pub upstream: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Branch {
    pub fn new(name: &str) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.to_string(),
            head_commit: None,
            upstream: None,
            created_at: now,
            last_updated: now,
        }
    }

    #[allow(dead_code)]
    pub fn with_head(name: &str, head_commit: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            name: name.to_string(),
            head_commit: Some(head_commit),
            upstream: None,
            created_at: now,
            last_updated: now,
        }
    }

    pub fn update_head(&mut self, commit_id: String) {
        self.head_commit = Some(commit_id);
        self.last_updated = chrono::Utc::now();
    }

    pub fn set_head_commit(&mut self, commit_id: String) {
        self.head_commit = Some(commit_id);
        self.last_updated = chrono::Utc::now();
    }

    #[allow(dead_code)]
    pub fn set_upstream(&mut self, upstream: String) {
        self.upstream = Some(upstream);
    }

    pub fn get_head_commit(&self) -> Option<&String> {
        self.head_commit.as_ref()
    }

    #[allow(dead_code)]
    pub fn has_upstream(&self) -> bool {
        self.upstream.is_some()
    }

    pub fn get_upstream(&self) -> Option<&String> {
        self.upstream.as_ref()
    }

    pub fn is_main(&self) -> bool {
        self.name == "main" || self.name == "master"
    }

    pub fn get_age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.created_at
    }

    pub fn get_last_update_age(&self) -> chrono::Duration {
        chrono::Utc::now() - self.last_updated
    }
}
