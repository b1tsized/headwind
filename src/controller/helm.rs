use crate::metrics::{
    HELM_CHART_VERSIONS_CHECKED, HELM_RELEASES_WATCHED, HELM_UPDATES_APPROVED, HELM_UPDATES_FOUND,
    HELM_UPDATES_REJECTED, RECONCILE_DURATION, RECONCILE_ERRORS,
};
use crate::models::crd::{
    TargetRef, UpdatePhase, UpdatePolicyType, UpdateRequest, UpdateRequestSpec,
    UpdateRequestStatus, UpdateType,
};
use crate::models::policy::annotations;
use crate::models::{HelmRelease, ResourcePolicy, UpdatePolicy};
use crate::policy::PolicyEngine;
use anyhow::Result;
use futures::StreamExt;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{
    Api, Client, ResourceExt,
    api::ListParams,
    runtime::{Controller, controller::Action, watcher::Config},
};
use std::{collections::BTreeMap, sync::Arc, time::Duration};
use tracing::{debug, error, info, warn};

pub struct HelmController {
    client: Client,
    policy_engine: Arc<PolicyEngine>,
}

impl HelmController {
    pub async fn new(policy_engine: Arc<PolicyEngine>) -> Result<Self> {
        let client = Client::try_default().await?;
        info!("Helm controller initialized");
        Ok(Self {
            client,
            policy_engine,
        })
    }

    pub async fn run(self) {
        let api: Api<HelmRelease> = Api::all(self.client.clone());

        // Create context to pass to reconcile function
        let context = Arc::new(ControllerContext {
            client: self.client.clone(),
            policy_engine: self.policy_engine.clone(),
        });

        // Set up controller with exponential backoff
        Controller::new(api, Config::default())
            .shutdown_on_signal()
            .run(reconcile, error_policy, context)
            .filter_map(|x| async move { std::result::Result::ok(x) })
            .for_each(|_| futures::future::ready(()))
            .await;
    }
}

struct ControllerContext {
    client: Client,
    policy_engine: Arc<PolicyEngine>,
}

async fn reconcile(
    helm_release: Arc<HelmRelease>,
    ctx: Arc<ControllerContext>,
) -> Result<Action, kube::Error> {
    let _timer = RECONCILE_DURATION.start_timer();

    let namespace = helm_release.namespace().ok_or_else(|| {
        kube::Error::Api(kube::error::ErrorResponse {
            status: "Failure".to_string(),
            message: "HelmRelease must be namespaced".to_string(),
            reason: "BadRequest".to_string(),
            code: 400,
        })
    })?;
    let name = helm_release.name_any();

    debug!(
        "Reconciling HelmRelease {}/{} (generation {})",
        namespace,
        name,
        helm_release.metadata.generation.unwrap_or_default()
    );

    // Parse policy from annotations
    let policy = parse_policy_from_annotations(helm_release.metadata.annotations.as_ref());

    if policy == UpdatePolicy::None {
        debug!(
            "HelmRelease {}/{} has policy=none, skipping",
            namespace, name
        );
        return Ok(Action::requeue(Duration::from_secs(3600)));
    }

    // Extract chart information
    let chart_name = &helm_release.spec.chart.spec.chart;
    let current_version = helm_release
        .spec
        .chart
        .spec
        .version
        .as_deref()
        .unwrap_or("*");

    debug!(
        "HelmRelease {}/{} - Chart: {}, Current version: {}, Policy: {:?}",
        namespace, name, chart_name, current_version, policy
    );

    // Update metrics
    update_helm_releases_count(&ctx.client).await;

    // Get current deployed version from status (last_attempted_revision for Flux v2)
    let deployed_version = helm_release
        .status
        .as_ref()
        .and_then(|s| s.last_attempted_revision.as_deref());

    // For now, check if there's a spec version different from deployed version
    // In a full implementation, this would query a Helm repository for new versions
    if let Some(deployed_ver) = deployed_version {
        if current_version != "*" && current_version != deployed_ver {
            // Increment version check metric
            HELM_CHART_VERSIONS_CHECKED.inc();

            // Potential update available - increment found metric
            HELM_UPDATES_FOUND.inc();

            debug!(
                "HelmRelease {}/{} - Spec version {} differs from deployed {}",
                namespace, name, current_version, deployed_ver
            );

            // Build resource policy from annotations
            let resource_policy =
                build_resource_policy(helm_release.metadata.annotations.as_ref(), policy);

            // Check if update should proceed based on policy
            match ctx
                .policy_engine
                .should_update(&resource_policy, deployed_ver, current_version)
            {
                Ok(true) => {
                    // Increment approved metric
                    HELM_UPDATES_APPROVED.inc();

                    info!(
                        "HelmRelease {}/{} - Update from {} to {} approved by policy",
                        namespace, name, deployed_ver, current_version
                    );

                    // Create UpdateRequest
                    let update_request = create_update_request(
                        &namespace,
                        &name,
                        chart_name,
                        deployed_ver,
                        current_version,
                        &resource_policy,
                    );

                    let update_request_name =
                        update_request.metadata.name.as_deref().unwrap_or("unknown");

                    info!(
                        "Created update request {} for HelmRelease {}/{}",
                        update_request_name, namespace, name
                    );

                    // Send notification for UpdateRequest creation
                    crate::notifications::notify_update_request_created(
                        crate::notifications::DeploymentInfo {
                            name: name.clone(),
                            namespace: namespace.clone(),
                            current_image: format!("{}:{}", chart_name, deployed_ver),
                            new_image: format!("{}:{}", chart_name, current_version),
                            container: None,
                            resource_kind: Some("HelmRelease".to_string()),
                        },
                        format!("{:?}", resource_policy.policy),
                        resource_policy.require_approval,
                        update_request_name.to_string(),
                    );

                    // TODO: Store UpdateRequest in a persistent store
                    // For now, we just log it
                },
                Ok(false) => {
                    // Increment rejected metric
                    HELM_UPDATES_REJECTED.inc();

                    debug!(
                        "HelmRelease {}/{} - Update from {} to {} rejected by policy",
                        namespace, name, deployed_ver, current_version
                    );
                },
                Err(e) => {
                    warn!(
                        "HelmRelease {}/{} - Error checking update policy: {}",
                        namespace, name, e
                    );
                },
            }
        }
    } else {
        debug!(
            "HelmRelease {}/{} - No deployed version found in status",
            namespace, name
        );
    }

    // Requeue after a reasonable interval
    Ok(Action::requeue(Duration::from_secs(300)))
}

fn error_policy(
    _helm_release: Arc<HelmRelease>,
    error: &kube::Error,
    _ctx: Arc<ControllerContext>,
) -> Action {
    RECONCILE_ERRORS.inc();
    error!("Reconciliation error: {}", error);
    Action::requeue(Duration::from_secs(60))
}

fn parse_policy_from_annotations(annotations: Option<&BTreeMap<String, String>>) -> UpdatePolicy {
    annotations
        .and_then(|ann| ann.get(annotations::POLICY))
        .map(|policy_str| match policy_str.to_lowercase().as_str() {
            "patch" => UpdatePolicy::Patch,
            "minor" => UpdatePolicy::Minor,
            "major" => UpdatePolicy::Major,
            "all" => UpdatePolicy::All,
            "glob" => UpdatePolicy::Glob,
            "force" => UpdatePolicy::Force,
            "none" => UpdatePolicy::None,
            _ => {
                warn!("Unknown policy value: {}, defaulting to None", policy_str);
                UpdatePolicy::None
            },
        })
        .unwrap_or(UpdatePolicy::None)
}

async fn update_helm_releases_count(client: &Client) {
    let api: Api<HelmRelease> = Api::all(client.clone());
    match api.list(&ListParams::default()).await {
        Ok(list) => {
            HELM_RELEASES_WATCHED.set(list.items.len() as i64);
        },
        Err(e) => {
            error!("Failed to count HelmReleases: {}", e);
        },
    }
}

fn build_resource_policy(
    annotations: Option<&BTreeMap<String, String>>,
    policy: UpdatePolicy,
) -> ResourcePolicy {
    let pattern = annotations
        .and_then(|ann| ann.get(annotations::PATTERN))
        .map(|s| s.to_string());

    let require_approval = annotations
        .and_then(|ann| ann.get(annotations::REQUIRE_APPROVAL))
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(true);

    let min_update_interval = annotations
        .and_then(|ann| ann.get(annotations::MIN_UPDATE_INTERVAL))
        .and_then(|v| v.parse::<u64>().ok());

    ResourcePolicy {
        policy,
        pattern,
        require_approval,
        min_update_interval,
        images: Vec::new(),
    }
}

fn create_update_request(
    namespace: &str,
    name: &str,
    chart_name: &str,
    current_version: &str,
    new_version: &str,
    policy: &ResourcePolicy,
) -> UpdateRequest {
    let policy_type = match policy.policy {
        UpdatePolicy::Patch => UpdatePolicyType::Patch,
        UpdatePolicy::Minor => UpdatePolicyType::Minor,
        UpdatePolicy::Major => UpdatePolicyType::Major,
        UpdatePolicy::Glob => UpdatePolicyType::Glob,
        _ => UpdatePolicyType::None,
    };

    let spec = UpdateRequestSpec {
        target_ref: TargetRef {
            api_version: "helm.toolkit.fluxcd.io/v2".to_string(),
            kind: "HelmRelease".to_string(),
            name: name.to_string(),
            namespace: namespace.to_string(),
        },
        update_type: UpdateType::HelmChart,
        container_name: None,
        current_image: format!("{}:{}", chart_name, current_version),
        new_image: format!("{}:{}", chart_name, new_version),
        policy: policy_type,
        reason: Some(format!("New chart version {} available", new_version)),
        require_approval: policy.require_approval,
        expires_at: None,
    };

    let status = UpdateRequestStatus {
        phase: if policy.require_approval {
            UpdatePhase::Pending
        } else {
            UpdatePhase::Approved
        },
        ..Default::default()
    };

    UpdateRequest {
        metadata: ObjectMeta {
            name: Some(format!("{}-{}", name, chrono::Utc::now().timestamp())),
            namespace: Some(namespace.to_string()),
            ..Default::default()
        },
        spec,
        status: Some(status),
    }
}
