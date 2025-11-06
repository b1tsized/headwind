use crate::metrics::{POLLING_CYCLES_TOTAL, POLLING_IMAGES_CHECKED, POLLING_NEW_TAGS_FOUND};
use crate::models::policy::annotations;
use crate::models::webhook::ImagePushEvent;
use anyhow::Result;
use k8s_openapi::api::apps::v1::Deployment;
use kube::{Api, Client};
use oci_distribution::{Client as OciClient, Reference};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

/// Configuration for registry polling
#[derive(Clone, Debug)]
pub struct PollingConfig {
    /// How often to poll registries (in seconds)
    pub interval: u64,
    /// Enable/disable polling
    pub enabled: bool,
}

impl Default for PollingConfig {
    fn default() -> Self {
        Self {
            interval: 300,  // 5 minutes
            enabled: false, // Disabled by default, webhooks preferred
        }
    }
}

/// Tracks the last seen tag for each image
type ImageTagCache = Arc<RwLock<HashMap<String, String>>>;

pub struct RegistryPoller {
    config: PollingConfig,
    cache: ImageTagCache,
    event_sender: crate::webhook::EventSender,
    client: Client,
}

impl RegistryPoller {
    pub async fn new(
        config: PollingConfig,
        event_sender: crate::webhook::EventSender,
    ) -> Result<Self> {
        let client = Client::try_default().await?;
        Ok(Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            event_sender,
            client,
        })
    }

    pub async fn start(self) -> JoinHandle<()> {
        info!(
            "Starting registry poller (enabled: {}, interval: {}s)",
            self.config.enabled, self.config.interval
        );

        tokio::spawn(async move {
            if !self.config.enabled {
                info!("Registry polling is disabled");
                // Keep the event sender alive by moving it into an infinite loop
                // This prevents the webhook event channel from closing
                let _sender = self.event_sender;
                loop {
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                }
            }

            loop {
                if let Err(e) = self.poll_registries().await {
                    error!("Error polling registries: {}", e);
                }

                tokio::time::sleep(Duration::from_secs(self.config.interval)).await;
            }
        })
    }

    async fn poll_registries(&self) -> Result<()> {
        debug!("Starting registry poll cycle");
        POLLING_CYCLES_TOTAL.inc();

        // Get list of images to track from Kubernetes
        let images = self.get_tracked_images().await?;
        info!("Found {} unique images to track", images.len());

        // Poll each image for new tags
        for image in images {
            if let Err(e) = self.poll_image(&image).await {
                error!("Failed to poll image {}: {}", image, e);
            }
        }

        info!("Registry poll cycle completed");
        Ok(())
    }

    /// Get the list of images to track from Kubernetes Deployments
    async fn get_tracked_images(&self) -> Result<HashSet<String>> {
        let deployments: Api<Deployment> = Api::all(self.client.clone());
        let deployment_list = deployments.list(&Default::default()).await?;

        let mut images = HashSet::new();

        for deployment in deployment_list.items {
            let metadata = &deployment.metadata;
            let annotations = match &metadata.annotations {
                Some(ann) => ann,
                None => continue,
            };

            // Skip deployments without headwind policy annotation
            let policy = match annotations.get(annotations::POLICY) {
                Some(p) if p != "none" => p,
                _ => continue,
            };

            debug!(
                "Processing deployment {}/{} with policy {}",
                metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                metadata.name.as_ref().unwrap_or(&"unknown".to_string()),
                policy
            );

            // Extract images from pod template
            if let Some(spec) = &deployment.spec
                && let Some(template) = &spec.template.spec
            {
                for container in &template.containers {
                    if let Some(image) = &container.image {
                        debug!("  Adding image to track: {}", image);
                        images.insert(image.clone());
                    }
                }
            }
        }

        Ok(images)
    }

    /// Poll a specific image for new tags
    #[allow(dead_code)]
    pub async fn poll_image(&self, image: &str) -> Result<Option<String>> {
        let reference = Reference::try_from(image)?;

        debug!("Polling image: {}", image);
        POLLING_IMAGES_CHECKED.inc();

        // Create OCI client
        let mut client = OciClient::new(Default::default());

        // Get list of tags
        // Note: Not all registries support tag listing
        let tags = match self.list_tags(&mut client, &reference).await {
            Ok(tags) => tags,
            Err(e) => {
                warn!("Failed to list tags for {}: {}", image, e);
                return Ok(None);
            },
        };

        if tags.is_empty() {
            return Ok(None);
        }

        // Get the latest tag (you might want to sort by semver here)
        let latest_tag = tags.first().unwrap();

        // Check cache
        let cache = self.cache.read().await;
        let cached_tag = cache.get(image);

        if let Some(cached) = cached_tag
            && cached == latest_tag
        {
            // No change
            return Ok(None);
        }
        drop(cache);

        // New tag found
        info!("New tag found for {}: {}", image, latest_tag);
        POLLING_NEW_TAGS_FOUND.inc();

        // Update cache
        let mut cache = self.cache.write().await;
        cache.insert(image.to_string(), latest_tag.clone());
        drop(cache);

        // Send event
        let event = ImagePushEvent {
            registry: extract_registry(reference.registry()),
            repository: reference.repository().to_string(),
            tag: latest_tag.clone(),
            digest: None,
        };

        if let Err(e) = self.event_sender.send(event) {
            error!("Failed to send polling event: {}", e);
        }

        Ok(Some(latest_tag.clone()))
    }

    /// List tags for a given image reference
    async fn list_tags(
        &self,
        _client: &mut OciClient,
        _reference: &Reference,
    ) -> Result<Vec<String>> {
        // Note: This is a simplified implementation
        // Full implementation would need to handle:
        // - Authentication
        // - Pagination
        // - Different registry APIs
        // - Rate limiting

        // For now, return empty as this requires registry-specific implementation
        warn!("Tag listing not fully implemented yet");
        Ok(Vec::new())
    }
}

fn extract_registry(registry: &str) -> String {
    if registry.is_empty() {
        "docker.io".to_string()
    } else {
        registry.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_polling_config_default() {
        let config = PollingConfig::default();
        assert_eq!(config.interval, 300);
        assert!(!config.enabled);
    }

    #[test]
    fn test_extract_registry() {
        assert_eq!(extract_registry(""), "docker.io");
        assert_eq!(extract_registry("gcr.io"), "gcr.io");
        assert_eq!(
            extract_registry("registry.example.com"),
            "registry.example.com"
        );
    }
}
