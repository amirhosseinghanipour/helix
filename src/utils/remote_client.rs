use anyhow::{Context, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE},
    Client, Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use crate::utils::auth::AuthManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationRequest {
    pub wants: Vec<String>,
    pub haves: Vec<String>,
    pub shallow: Vec<String>,
    pub deepen_since: Option<i64>,
    pub deepen_not: Option<Vec<String>>,
    pub filter: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NegotiationResponse {
    pub acks: Vec<String>,
    pub nak: Vec<String>,
    pub shallow: Vec<String>,
    pub unshallow: Vec<String>,
    pub packfile: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    pub refs: HashMap<String, String>,
    pub objects: Vec<String>,
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResponse {
    pub success: bool,
    pub updated_refs: Vec<String>,
    pub rejected_refs: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capabilities {
    pub multi_ack: bool,
    pub multi_ack_detailed: bool,
    pub side_band: bool,
    pub side_band_64k: bool,
    pub ofs_delta: bool,
    pub thin_pack: bool,
    pub shallow: bool,
    pub no_progress: bool,
    pub include_tag: bool,
    pub report_status: bool,
    pub delete_refs: bool,
    pub quiet: bool,
    pub atomic: bool,
    pub push_options: bool,
}

pub struct RemoteClient {
    pub base_url: String,
    pub client: Client,
    pub capabilities: Option<Capabilities>,
    pub auth_token: Option<String>,
    pub timeout: Duration,
    pub auth_manager: Option<AuthManager>,
}

impl RemoteClient {
    pub fn new(base_url: &str) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_static("Helix/1.0"),
        );
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
            capabilities: None,
            auth_token: None,
            timeout: Duration::from_secs(30),
            auth_manager: None,
        }
    }

    pub fn with_auth(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }

    pub fn with_auth_manager(mut self, auth_manager: AuthManager) -> Self {
        self.auth_manager = Some(auth_manager);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    async fn make_request(&self, method: &str, endpoint: &str, body: Option<&[u8]>) -> Result<Response> {
        let url = format!("{}/{}", self.base_url, endpoint.trim_start_matches('/'));
        let mut request = self.client.request(
            method.parse().unwrap(),
            &url,
        );

        // Add authentication headers
        if let Some(token) = &self.auth_token {
            request = request.header("Authorization", format!("Bearer {}", token));
        } else if let Some(auth_manager) = &self.auth_manager {
            let auth_headers = auth_manager.get_auth_headers(&url)?;
            for (key, value) in auth_headers {
                request = request.header(key, value);
            }
        }

        if let Some(body_data) = body {
            request = request.body(body_data.to_vec());
        }

        let response = request.send().await
            .with_context(|| format!("Failed to connect to {}", url))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!(
                "HTTP {}: {}",
                status,
                error_text
            ));
        }

        Ok(response)
    }

    pub async fn discover_capabilities(&mut self) -> Result<Capabilities> {
        let response = self.make_request("GET", "/info/refs", None).await?;
        let text = response.text().await?;
        
        // Parse capabilities from the response
        let capabilities = Capabilities {
            multi_ack: text.contains("multi_ack"),
            multi_ack_detailed: text.contains("multi_ack_detailed"),
            side_band: text.contains("side-band"),
            side_band_64k: text.contains("side-band-64k"),
            ofs_delta: text.contains("ofs-delta"),
            thin_pack: text.contains("thin-pack"),
            shallow: text.contains("shallow"),
            no_progress: text.contains("no-progress"),
            include_tag: text.contains("include-tag"),
            report_status: text.contains("report-status"),
            delete_refs: text.contains("delete-refs"),
            quiet: text.contains("quiet"),
            atomic: text.contains("atomic"),
            push_options: text.contains("push-options"),
        };

        self.capabilities = Some(capabilities.clone());
        Ok(capabilities)
    }

    pub async fn negotiate_fetch(&self, request: &NegotiationRequest) -> Result<NegotiationResponse> {
        let body = serde_json::to_vec(request)?;
        let response = self.make_request("POST", "/fetch", Some(&body)).await?;
        let negotiation_response: NegotiationResponse = response.json().await?;
        Ok(negotiation_response)
    }

    pub async fn negotiate_push(&self, request: &PushRequest) -> Result<PushResponse> {
        let body = serde_json::to_vec(request)?;
        let response = self.make_request("POST", "/push", Some(&body)).await?;
        let push_response: PushResponse = response.json().await?;
        Ok(push_response)
    }

    pub async fn upload_pack(&self, pack_data: &[u8]) -> Result<()> {
        let response = self.make_request("POST", "/upload-pack", Some(pack_data)).await?;
        
        // Check if upload was successful
        if response.status().is_success() {
            Ok(())
        } else {
            let error_text = response.text().await.unwrap_or_else(|_| "Upload failed".to_string());
            Err(anyhow::anyhow!("Pack upload failed: {}", error_text))
        }
    }

    pub async fn download_pack(&self, pack_id: &str) -> Result<Vec<u8>> {
        let response = self.make_request("GET", &format!("/pack/{}", pack_id), None).await?;
        Ok(response.bytes().await?.to_vec())
    }

    // Legacy methods for backward compatibility
    pub async fn upload_object(&self, hash: &str, data: &[u8]) -> Result<()> {
        let response = self.make_request("POST", &format!("/objects/{}", hash), Some(data)).await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to upload object: {}", response.status()))
        }
    }

    pub async fn download_object(&self, hash: &str) -> Result<Vec<u8>> {
        let response = self.make_request("GET", &format!("/objects/{}", hash), None).await?;
        Ok(response.bytes().await?.to_vec())
    }

    pub async fn get_ref(&self, branch: &str) -> Result<String> {
        let response = self.make_request("GET", &format!("/refs/{}", branch), None).await?;
        Ok(response.text().await?)
    }

    pub async fn set_ref(&self, branch: &str, value: &str) -> Result<()> {
        let response = self.make_request("POST", &format!("/refs/{}", branch), Some(value.as_bytes())).await?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to set ref: {}", response.status()))
        }
    }

    pub async fn get_all_object_hashes(&self) -> Result<Vec<String>> {
        let response = self.make_request("GET", "/objects", None).await?;
        let text = response.text().await?;
            Ok(text
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect())
    }

    pub async fn get_refs(&self) -> Result<HashMap<String, String>> {
        let response = self.make_request("GET", "/refs", None).await?;
        let refs: HashMap<String, String> = response.json().await?;
        Ok(refs)
    }

    pub async fn check_connectivity(&self) -> Result<bool> {
        match self.make_request("GET", "/health", None).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
