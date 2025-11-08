---
sidebar_position: 1
---

# Configuration Overview

Headwind is configured through Kubernetes annotations on your workload resources. This allows you to configure update behavior per-resource without modifying the operator itself.

## Supported Resources

Headwind supports the following Kubernetes resources:

- **Deployments** - Standard stateless applications
- **StatefulSets** - Stateful applications with persistent storage
- **DaemonSets** - Per-node applications (logging, monitoring, etc.)
- **HelmReleases** - Flux CD Helm chart deployments

## Common Annotations

All resource types support the same set of annotations:

| Annotation | Type | Default | Description |
|------------|------|---------|-------------|
| `headwind.sh/policy` | string | `none` | Update policy: `none`, `patch`, `minor`, `major`, `all`, `glob`, `force` |
| `headwind.sh/pattern` | string | - | Glob pattern (required for `glob` policy) |
| `headwind.sh/require-approval` | boolean | `true` | Whether updates require manual approval |
| `headwind.sh/min-update-interval` | integer | `300` | Minimum seconds between updates |
| `headwind.sh/images` | string | - | Comma-separated list of images to track (empty = all) |
| `headwind.sh/event-source` | string | `webhook` | Event source: `webhook`, `polling`, `both`, or `none` |
| `headwind.sh/polling-interval` | integer | - | Per-resource polling interval (seconds), overrides global setting |
| `headwind.sh/auto-rollback` | boolean | `false` | Enable automatic rollback on failures |
| `headwind.sh/rollback-timeout` | integer | `300` | Health check monitoring duration (seconds) |
| `headwind.sh/health-check-retries` | integer | `3` | Failed health checks before rollback |

## Managed Annotations

These annotations are managed by Headwind and should not be modified manually:

| Annotation | Description |
|------------|-------------|
| `headwind.sh/last-update` | RFC3339 timestamp of last update |
| `headwind.sh/update-history` | JSON array of previous updates (last 10) |

## Basic Example

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
  annotations:
    # Update to minor versions automatically
    headwind.sh/policy: "minor"

    # Require approval before applying
    headwind.sh/require-approval: "true"

    # Wait at least 10 minutes between updates
    headwind.sh/min-update-interval: "600"

    # Enable automatic rollback on failures
    headwind.sh/auto-rollback: "true"
spec:
  # ... rest of deployment spec
```

## Environment Variables

Configure the Headwind operator itself using environment variables in the deployment:

### Webhook Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `HEADWIND_WEBHOOK_PORT` | `8080` | Webhook server port |

### Polling Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `HEADWIND_POLLING_ENABLED` | `false` | Enable registry polling |
| `HEADWIND_POLLING_INTERVAL` | `300` | Poll interval in seconds |

### Helm Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `HEADWIND_HELM_AUTO_DISCOVERY` | `true` | Enable automatic Helm chart version discovery |

### Notification Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `SLACK_ENABLED` | `false` | Enable Slack notifications |
| `SLACK_WEBHOOK_URL` | - | Slack incoming webhook URL |
| `SLACK_CHANNEL` | - | Override webhook default channel |
| `TEAMS_ENABLED` | `false` | Enable Microsoft Teams notifications |
| `TEAMS_WEBHOOK_URL` | - | Teams incoming webhook URL |
| `WEBHOOK_ENABLED` | `false` | Enable generic webhook notifications |
| `WEBHOOK_URL` | - | Generic webhook endpoint URL |

See the specific configuration guides for each resource type and feature:

- [Deployments](./deployments.md)
- [StatefulSets](./statefulsets.md)
- [DaemonSets](./daemonsets.md)
- [HelmReleases](./helmreleases.md)
- [Event Sources](./event-sources.md) - Configure webhooks vs polling per-resource
- [Notifications](./notifications.md)
- [Approval Workflow](./approval-workflow.md)
- [Rollback](./rollback.md)
