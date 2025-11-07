use crate::metrics::{DAEMONSETS_WATCHED, RECONCILE_DURATION, RECONCILE_ERRORS};
use crate::models::{
    ResourcePolicy, TargetRef, UpdatePolicy, UpdatePolicyType, UpdateRequest, UpdateRequestSpec,
    UpdateType, annotations,
};
use crate::notifications::{self, DeploymentInfo};
use crate::policy::PolicyEngine;
use anyhow::Result;
use chrono::Utc;
use futures::StreamExt;
use k8s_openapi::api::apps::v1::DaemonSet;
use kube::{
    ResourceExt,
    api::{Api, Patch, PatchParams, PostParams},
    client::Client,
    runtime::{
        controller::{Action, Controller},
        watcher::Config,
    },
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument};

pub struct DaemonSetController {
    client: Client,
    policy_engine: Arc<PolicyEngine>,
}

impl DaemonSetController {
    pub async fn new() -> Result<Self> {
        let client = Client::try_default().await?;
        let policy_engine = Arc::new(PolicyEngine);

        Ok(Self {
            client,
            policy_engine,
        })
    }

    pub async fn run(self) {
        info!("DaemonSet controller starting...");

        // Run the controller in a loop with exponential backoff
        // This handles transient errors during startup or runtime
        let mut backoff_seconds = 1;
        const MAX_BACKOFF: u64 = 60;

        loop {
            let daemonsets: Api<DaemonSet> = Api::all(self.client.clone());

            info!("Creating controller for daemonsets");

            let result = Controller::new(daemonsets, Config::default())
                .run(
                    reconcile,
                    error_policy,
                    Arc::new(ControllerContext {
                        client: self.client.clone(),
                        policy_engine: self.policy_engine.clone(),
                    }),
                )
                .for_each(|res| async move {
                    match res {
                        Ok((obj_ref, _action)) => {
                            info!(
                                "Reconciled daemonset: {}/{}",
                                obj_ref.namespace.as_deref().unwrap_or("default"),
                                obj_ref.name
                            );
                        },
                        Err(e) => {
                            // Log reconciliation errors but continue processing
                            error!("Reconciliation error: {}", e);
                            RECONCILE_ERRORS.inc();
                        },
                    }
                })
                .await;

            // If the controller stream ends, log it and restart after backoff
            error!(
                "DaemonSet controller stream ended, restarting in {}s...",
                backoff_seconds
            );
            tokio::time::sleep(Duration::from_secs(backoff_seconds)).await;

            // Exponential backoff up to MAX_BACKOFF seconds
            backoff_seconds = (backoff_seconds * 2).min(MAX_BACKOFF);

            debug!("Controller loop result: {:?}", result);
        }
    }
}

struct ControllerContext {
    #[allow(dead_code)]
    client: Client,
    #[allow(dead_code)]
    policy_engine: Arc<PolicyEngine>,
}

#[instrument(skip(_ctx), fields(daemonset = %daemonset.name_any()))]
async fn reconcile(
    daemonset: Arc<DaemonSet>,
    _ctx: Arc<ControllerContext>,
) -> Result<Action, kube::Error> {
    let _timer = RECONCILE_DURATION.start_timer();

    let namespace = daemonset.namespace().unwrap_or_default();
    let name = daemonset.name_any();

    debug!("Reconciling daemonset {}/{} - starting", namespace, name);

    // Parse headwind annotations to get update policy
    let annotations = daemonset.metadata.annotations.as_ref();
    if annotations.is_none() {
        debug!(
            "DaemonSet {}/{} has no annotations, skipping",
            namespace, name
        );
        return Ok(Action::requeue(Duration::from_secs(300)));
    }

    let annotations = annotations.unwrap();

    // Check if this daemonset has headwind annotations
    if !annotations.contains_key(annotations::POLICY) {
        debug!(
            "DaemonSet {}/{} has no headwind policy annotation, skipping",
            namespace, name
        );
        return Ok(Action::requeue(Duration::from_secs(300)));
    }

    // Parse the policy from annotations
    let policy = match parse_policy_from_annotations(annotations) {
        Ok(p) => p,
        Err(e) => {
            error!(
                "Failed to parse policy for daemonset {}/{}: {}",
                namespace, name, e
            );
            return Err(create_error(&format!("Failed to parse policy: {}", e)));
        },
    };

    debug!(
        "DaemonSet {}/{} has policy: {:?}",
        namespace, name, policy.policy
    );

    // Update the gauge for watched daemonsets
    DAEMONSETS_WATCHED.set(1);

    // Check if there are any available updates for this daemonset
    // This would be triggered by webhook/polling events
    // For now, we just requeue to check again later
    debug!("DaemonSet {}/{} reconciliation complete", namespace, name);

    Ok(Action::requeue(Duration::from_secs(300)))
}

fn error_policy(
    _object: Arc<DaemonSet>,
    _error: &kube::Error,
    _ctx: Arc<ControllerContext>,
) -> Action {
    // Requeue after 60 seconds on errors
    Action::requeue(Duration::from_secs(60))
}

/// Helper to create a kube::Error from a string message
fn create_error(msg: &str) -> kube::Error {
    kube::Error::Api(kube::error::ErrorResponse {
        status: "Failure".to_string(),
        message: msg.to_string(),
        reason: "InvalidConfiguration".to_string(),
        code: 400,
    })
}

/// Parse an image string into (image_name, tag)
/// Example: "myregistry.com/myimage:v1.2.3" -> ("myregistry.com/myimage", "v1.2.3")
fn parse_image(image: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = image.rsplitn(2, ':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid image format: {}", image));
    }
    Ok((parts[1].to_string(), parts[0].to_string()))
}

/// Handle an available image update for a daemonset
/// This is called when we detect a new version is available (via webhook or polling)
#[allow(dead_code)]
#[instrument(skip(client, policy_engine))]
pub async fn handle_image_update(
    client: &Client,
    policy_engine: &Arc<PolicyEngine>,
    daemonset: &DaemonSet,
    image: &str,
    new_version: &str,
) -> Result<()> {
    let namespace = daemonset.namespace().unwrap_or_default();
    let name = daemonset.name_any();

    info!(
        "Handling image update for daemonset {}/{}: {} -> {}",
        namespace, name, image, new_version
    );

    // Parse policy from annotations
    let annotations = daemonset
        .metadata
        .annotations
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("DaemonSet has no annotations"))?;

    let policy = parse_policy_from_annotations(annotations)?;

    // Find the container using this image
    let spec = daemonset
        .spec
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("DaemonSet has no spec"))?;

    let template_spec = spec
        .template
        .spec
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("DaemonSet template has no spec"))?;

    let containers = &template_spec.containers;

    // Find container with matching image
    let mut current_version = None;
    for container in containers {
        let empty_image = String::new();
        let container_image = container.image.as_ref().unwrap_or(&empty_image);
        let (img_name, img_tag) = parse_image(container_image)
            .map_err(|e| anyhow::anyhow!("Failed to parse container image: {}", e))?;

        if img_name == image || container_image.starts_with(image) {
            current_version = Some(img_tag);
            break;
        }
    }

    let current_version = current_version
        .ok_or_else(|| anyhow::anyhow!("Container with image {} not found", image))?;

    debug!(
        "Current version: {}, new version: {}",
        current_version, new_version
    );

    // Check if we should update based on policy
    let should_update = policy_engine
        .should_update(&policy, &current_version, new_version)
        .map_err(|e| anyhow::anyhow!("Policy evaluation failed: {}", e))?;

    if !should_update {
        info!(
            "Update from {} to {} rejected by policy {:?}",
            current_version, new_version, policy.policy
        );
        return Ok(());
    }

    info!(
        "Update from {} to {} approved by policy {:?}",
        current_version, new_version, policy.policy
    );

    // Check minimum update interval
    if let (Some(min_interval), Some(last_update_str)) = (
        policy.min_update_interval,
        annotations.get(annotations::LAST_UPDATE),
    ) && let Ok(last_update) = chrono::DateTime::parse_from_rfc3339(last_update_str)
    {
        let elapsed = Utc::now().signed_duration_since(last_update.with_timezone(&Utc));
        let min_duration = chrono::Duration::seconds(min_interval as i64);

        if elapsed < min_duration {
            info!(
                "Skipping update for daemonset {}/{}: minimum interval not met ({} < {} seconds)",
                namespace,
                name,
                elapsed.num_seconds(),
                min_interval
            );
            return Ok(());
        }
    }

    // Check if approval is required
    if policy.require_approval {
        info!(
            "Creating UpdateRequest for daemonset {}/{}: {} -> {}",
            namespace, name, current_version, new_version
        );

        create_update_request(
            client,
            &namespace,
            &name,
            image,
            &current_version,
            new_version,
            &policy,
        )
        .await?;
    } else {
        info!(
            "Auto-updating daemonset {}/{} (no approval required): {} -> {}",
            namespace, name, current_version, new_version
        );

        // Apply update directly
        update_daemonset_image(client, &namespace, &name, image, new_version).await?;

        // Send notification
        notifications::notify_update_completed(DeploymentInfo {
            name: name.clone(),
            namespace: namespace.clone(),
            current_image: format!("{}:{}", image, current_version),
            new_image: format!("{}:{}", image, new_version),
            container: None,
            resource_kind: Some("DaemonSet".to_string()),
        });
    }

    Ok(())
}

/// Create an UpdateRequest CRD for a pending update
#[allow(dead_code)]
async fn create_update_request(
    client: &Client,
    namespace: &str,
    name: &str,
    image: &str,
    current_version: &str,
    new_version: &str,
    policy: &ResourcePolicy,
) -> Result<()> {
    let update_requests: Api<UpdateRequest> = Api::namespaced(client.clone(), namespace);

    // Create a unique name for the update request
    let request_name = format!(
        "{}-{}",
        name,
        new_version.replace([':', '.', '/'], "-").to_lowercase()
    );

    debug!(
        "Creating UpdateRequest: {}/{} for daemonset {}",
        namespace, request_name, name
    );

    let update_request = UpdateRequest {
        metadata: kube::api::ObjectMeta {
            name: Some(request_name.clone()),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec: UpdateRequestSpec {
            target_ref: TargetRef {
                api_version: "apps/v1".to_string(),
                kind: "DaemonSet".to_string(),
                name: name.to_string(),
                namespace: namespace.to_string(),
            },
            update_type: UpdateType::Image,
            container_name: None,
            current_image: format!("{}:{}", image, current_version),
            new_image: format!("{}:{}", image, new_version),
            policy: map_policy_to_crd(&policy.policy),
            reason: Some(format!(
                "Update from {} to {}",
                current_version, new_version
            )),
            require_approval: true,
            expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
        },
        status: None,
    };

    // Check if UpdateRequest already exists
    match update_requests.get(&request_name).await {
        Ok(existing) => {
            debug!(
                "UpdateRequest {}/{} already exists, skipping creation",
                namespace, request_name
            );
            // Check if it's in a terminal state (Completed, Rejected, Failed)
            if let Some(status) = &existing.status {
                use crate::models::crd::UpdatePhase;
                if status.phase == UpdatePhase::Completed
                    || status.phase == UpdatePhase::Rejected
                    || status.phase == UpdatePhase::Failed
                {
                    info!(
                        "Existing UpdateRequest is in terminal state ({:?}), creating new one",
                        status.phase
                    );
                    // Delete the old one and create a new one
                    update_requests
                        .delete(&request_name, &Default::default())
                        .await?;
                    update_requests
                        .create(&PostParams::default(), &update_request)
                        .await?;
                }
            }
        },
        Err(kube::Error::Api(err)) if err.code == 404 => {
            // Doesn't exist, create it
            update_requests
                .create(&PostParams::default(), &update_request)
                .await?;
            info!(
                "Created UpdateRequest {}/{} for daemonset {}",
                namespace, request_name, name
            );
        },
        Err(e) => {
            error!("Failed to check for existing UpdateRequest: {}", e);
            return Err(anyhow::anyhow!("Failed to check UpdateRequest: {}", e));
        },
    }

    Ok(())
}

/// Map internal UpdatePolicy to CRD UpdatePolicyType
#[allow(dead_code)]
fn map_policy_to_crd(policy: &UpdatePolicy) -> UpdatePolicyType {
    match policy {
        UpdatePolicy::Patch => UpdatePolicyType::Patch,
        UpdatePolicy::Minor => UpdatePolicyType::Minor,
        UpdatePolicy::Major => UpdatePolicyType::Major,
        UpdatePolicy::Glob => UpdatePolicyType::Glob,
        UpdatePolicy::None => UpdatePolicyType::None,
        // Map All and Force to Major since they don't exist in CRD
        UpdatePolicy::All | UpdatePolicy::Force => UpdatePolicyType::Major,
    }
}

/// Simple glob pattern matching (supports * and ?)
#[allow(dead_code)]
fn glob_match(pattern: &str, text: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    glob_match_impl(&pattern_chars, &text_chars, 0, 0)
}

#[allow(dead_code)]
fn glob_match_impl(pattern: &[char], text: &[char], pi: usize, ti: usize) -> bool {
    if pi >= pattern.len() && ti >= text.len() {
        return true;
    }
    if pi >= pattern.len() {
        return false;
    }

    if pattern[pi] == '*' {
        // Try matching zero or more characters
        glob_match_impl(pattern, text, pi + 1, ti)
            || (ti < text.len() && glob_match_impl(pattern, text, pi, ti + 1))
    } else if ti < text.len() && (pattern[pi] == '?' || pattern[pi] == text[ti]) {
        glob_match_impl(pattern, text, pi + 1, ti + 1)
    } else {
        false
    }
}

/// Parse ResourcePolicy from Deployment annotations
fn parse_policy_from_annotations(
    annotations: &std::collections::BTreeMap<String, String>,
) -> Result<ResourcePolicy> {
    let policy_str = annotations
        .get(annotations::POLICY)
        .ok_or_else(|| anyhow::anyhow!("No policy annotation found"))?;

    let policy = match policy_str.as_str() {
        "patch" => UpdatePolicy::Patch,
        "minor" => UpdatePolicy::Minor,
        "major" => UpdatePolicy::Major,
        "all" => UpdatePolicy::All,
        "glob" => UpdatePolicy::Glob,
        "force" => UpdatePolicy::Force,
        "none" => UpdatePolicy::None,
        _ => {
            return Err(anyhow::anyhow!("Invalid update policy: {}", policy_str));
        },
    };

    let pattern = annotations.get(annotations::PATTERN).cloned();

    let require_approval = annotations
        .get(annotations::REQUIRE_APPROVAL)
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(true);

    let min_update_interval = annotations
        .get(annotations::MIN_UPDATE_INTERVAL)
        .and_then(|v| v.parse::<u64>().ok());

    let images = annotations
        .get(annotations::IMAGES)
        .map(|s| s.split(',').map(|i| i.trim().to_string()).collect())
        .unwrap_or_default();

    Ok(ResourcePolicy {
        policy,
        pattern,
        require_approval,
        min_update_interval,
        images,
    })
}

/// Update a daemonset's container image - public wrapper
pub async fn update_daemonset_image(
    client: &Client,
    namespace: &str,
    name: &str,
    image: &str,
    new_version: &str,
) -> Result<()> {
    update_daemonset_image_with_tracking(client, namespace, name, image, new_version, None).await
}

/// Update a daemonset's container image with tracking
/// If approver is provided, it will be recorded in the last-update annotation
pub async fn update_daemonset_image_with_tracking(
    client: &Client,
    namespace: &str,
    name: &str,
    image: &str,
    new_version: &str,
    approver: Option<&str>,
) -> Result<()> {
    let daemonsets: Api<DaemonSet> = Api::namespaced(client.clone(), namespace);

    // Build new image string
    let new_image = format!("{}:{}", image, new_version);

    info!(
        "Updating daemonset {}/{} image to {}",
        namespace, name, new_image
    );

    // Create strategic merge patch to update the container image
    // We need to find which container to update
    let daemonset = daemonsets.get(name).await?;
    let spec = daemonset
        .spec
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("DaemonSet has no spec"))?;

    let template_spec = spec
        .template
        .spec
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("DaemonSet template has no spec"))?;

    // Find the container index
    let mut container_index = None;
    for (idx, container) in template_spec.containers.iter().enumerate() {
        if let Some(container_image) = &container.image
            && container_image.starts_with(image)
        {
            container_index = Some(idx);
            break;
        }
    }

    let container_index = container_index
        .ok_or_else(|| anyhow::anyhow!("Container with image {} not found", image))?;

    // Update last-update annotation with timestamp
    let now = Utc::now();
    let last_update_value = if let Some(approver) = approver {
        format!("{} (approved by {})", now.to_rfc3339(), approver)
    } else {
        now.to_rfc3339()
    };

    let patch = json!({
        "spec": {
            "template": {
                "spec": {
                    "containers": [{
                        "name": template_spec.containers[container_index].name,
                        "image": new_image
                    }]
                }
            }
        },
        "metadata": {
            "annotations": {
                annotations::LAST_UPDATE: last_update_value
            }
        }
    });

    daemonsets
        .patch(
            name,
            &PatchParams::apply("headwind"),
            &Patch::Strategic(patch),
        )
        .await?;

    info!(
        "Successfully updated daemonset {}/{} to version {}",
        namespace, name, new_version
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_image() {
        let (name, tag) = parse_image("myregistry.com/myimage:v1.2.3").unwrap();
        assert_eq!(name, "myregistry.com/myimage");
        assert_eq!(tag, "v1.2.3");

        let (name, tag) = parse_image("nginx:latest").unwrap();
        assert_eq!(name, "nginx");
        assert_eq!(tag, "latest");
    }

    #[test]
    fn test_parse_image_invalid() {
        assert!(parse_image("invalid-no-tag").is_err());
    }

    #[test]
    fn test_glob_match() {
        assert!(glob_match("v1.*.0", "v1.2.0"));
        assert!(glob_match("v1.*.0", "v1.99.0"));
        assert!(!glob_match("v1.*.0", "v1.2.1"));
        assert!(!glob_match("v1.*.0", "v2.0.0"));

        assert!(glob_match("v*-stable", "v1.2.3-stable"));
        assert!(glob_match("v*-stable", "v999-stable"));
        assert!(!glob_match("v*-stable", "v1.2.3-beta"));
    }

    #[test]
    fn test_parse_policy_from_annotations() {
        let mut annotations = std::collections::BTreeMap::new();
        annotations.insert(annotations::POLICY.to_string(), "minor".to_string());
        annotations.insert(
            annotations::REQUIRE_APPROVAL.to_string(),
            "false".to_string(),
        );
        annotations.insert(
            annotations::MIN_UPDATE_INTERVAL.to_string(),
            "600".to_string(),
        );

        let policy = parse_policy_from_annotations(&annotations).unwrap();
        assert_eq!(policy.policy, UpdatePolicy::Minor);
        assert!(!policy.require_approval);
        assert_eq!(policy.min_update_interval, Some(600));
    }
}
