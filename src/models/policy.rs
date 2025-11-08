use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdatePolicy {
    /// Only update patch versions (1.2.3 -> 1.2.4)
    Patch,
    /// Update minor versions (1.2.3 -> 1.3.0)
    Minor,
    /// Update major versions (1.2.3 -> 2.0.0)
    Major,
    /// Update to any new version
    All,
    /// Match glob pattern
    Glob,
    /// Force update regardless of version
    Force,
    /// Never update automatically
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EventSource {
    /// Process webhook events only (default)
    #[default]
    Webhook,
    /// Process polling events only
    Polling,
    /// Process both webhook and polling events
    Both,
    /// Only process manual UpdateRequest CRDs
    None,
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("Invalid policy: {0}")]
    InvalidPolicy(String),
    #[error("Invalid event source: {0}")]
    InvalidEventSource(String),
}

impl FromStr for UpdatePolicy {
    type Err = PolicyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "patch" => Ok(UpdatePolicy::Patch),
            "minor" => Ok(UpdatePolicy::Minor),
            "major" => Ok(UpdatePolicy::Major),
            "all" => Ok(UpdatePolicy::All),
            "glob" => Ok(UpdatePolicy::Glob),
            "force" => Ok(UpdatePolicy::Force),
            "none" => Ok(UpdatePolicy::None),
            _ => Err(PolicyError::InvalidPolicy(s.to_string())),
        }
    }
}

impl FromStr for EventSource {
    type Err = PolicyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "webhook" => Ok(EventSource::Webhook),
            "polling" => Ok(EventSource::Polling),
            "both" => Ok(EventSource::Both),
            "none" => Ok(EventSource::None),
            _ => Err(PolicyError::InvalidEventSource(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePolicy {
    /// Update policy to apply
    pub policy: UpdatePolicy,

    /// Optional glob pattern for matching versions (when policy is Glob)
    pub pattern: Option<String>,

    /// Whether approval is required before updating
    pub require_approval: bool,

    /// Minimum time between updates (in seconds)
    pub min_update_interval: Option<u64>,

    /// Images to track (if empty, track all)
    pub images: Vec<String>,

    /// Event source configuration (webhook, polling, both, none)
    pub event_source: EventSource,

    /// Per-resource polling interval in seconds (overrides global setting)
    pub polling_interval: Option<u64>,
}

impl Default for ResourcePolicy {
    fn default() -> Self {
        Self {
            policy: UpdatePolicy::None,
            pattern: None,
            require_approval: true,
            min_update_interval: Some(300), // 5 minutes
            images: Vec::new(),
            event_source: EventSource::default(),
            polling_interval: None,
        }
    }
}

/// Annotation keys used on Kubernetes resources
pub mod annotations {
    pub const POLICY: &str = "headwind.sh/policy";
    pub const PATTERN: &str = "headwind.sh/pattern";
    pub const REQUIRE_APPROVAL: &str = "headwind.sh/require-approval";
    pub const MIN_UPDATE_INTERVAL: &str = "headwind.sh/min-update-interval";
    pub const IMAGES: &str = "headwind.sh/images";
    #[allow(dead_code)]
    pub const LAST_UPDATE: &str = "headwind.sh/last-update";

    // Event source configuration
    pub const EVENT_SOURCE: &str = "headwind.sh/event-source";
    pub const POLLING_INTERVAL: &str = "headwind.sh/polling-interval";

    // Automatic rollback annotations
    pub const AUTO_ROLLBACK: &str = "headwind.sh/auto-rollback";
    pub const ROLLBACK_TIMEOUT: &str = "headwind.sh/rollback-timeout";
    pub const HEALTH_CHECK_RETRIES: &str = "headwind.sh/health-check-retries";
}
