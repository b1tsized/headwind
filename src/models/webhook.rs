use serde::{Deserialize, Serialize};

/// Generic webhook payload for container registry notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryWebhook {
    pub events: Vec<RegistryEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEvent {
    pub action: String,
    pub target: Target,
    pub request: Option<Request>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Target {
    #[serde(rename = "mediaType")]
    pub media_type: Option<String>,
    pub digest: String,
    pub repository: String,
    pub tag: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub method: String,
    pub useragent: String,
}

/// Docker Hub webhook format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerHubWebhook {
    pub push_data: PushData,
    pub repository: Repository,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushData {
    pub tag: String,
    pub pushed_at: Option<i64>,
    pub pusher: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub repo_name: String,
    pub namespace: String,
    pub name: String,
}

/// Normalized webhook event after parsing (for container images)
#[derive(Debug, Clone)]
pub struct ImagePushEvent {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    #[allow(dead_code)]
    pub digest: Option<String>,
}

impl ImagePushEvent {
    pub fn full_image(&self) -> String {
        if self.registry.is_empty() || self.registry == "docker.io" {
            format!("{}:{}", self.repository, self.tag)
        } else {
            format!("{}/{}:{}", self.registry, self.repository, self.tag)
        }
    }
}

/// Normalized webhook event for Helm chart pushes
#[derive(Debug, Clone)]
pub struct ChartPushEvent {
    /// OCI registry (e.g., "ghcr.io", "registry.example.com")
    pub registry: String,
    /// Chart repository path (e.g., "myorg/charts/mychart")
    pub repository: String,
    /// Chart version (e.g., "1.2.3")
    pub version: String,
    /// SHA256 digest
    pub digest: Option<String>,
}

impl ChartPushEvent {
    /// Returns the full OCI URL for this chart
    /// Example: oci://ghcr.io/myorg/charts/mychart:1.2.3
    pub fn full_oci_url(&self) -> String {
        format!(
            "oci://{}/{}:{}",
            self.registry, self.repository, self.version
        )
    }

    /// Returns just the base OCI URL without version
    /// Example: oci://ghcr.io/myorg/charts
    pub fn base_oci_url(&self) -> String {
        format!("oci://{}/{}", self.registry, self.repository)
    }
}
