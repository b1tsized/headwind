---
sidebar_position: 7
---

# Event Sources

Headwind supports two methods for detecting new container images and Helm chart versions: **webhooks** (event-driven) and **registry polling** (periodic checking). You can configure which method to use on a per-resource basis.

## Overview

By default, Headwind uses webhooks as the event source. This provides the fastest update detection with minimal resource usage. However, you can configure each resource individually to use:

- **Webhook** (default) - Event-driven updates from registry webhooks
- **Polling** - Periodic checking of registries for new versions
- **Both** - Redundant detection using both methods
- **None** - Disable all update detection for this resource

## Event Source Annotation

Use the `headwind.sh/event-source` annotation to control how updates are detected:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
  annotations:
    headwind.sh/policy: "minor"
    headwind.sh/event-source: "webhook"  # webhook, polling, both, or none
```

## Webhook Event Source

**Best for**: Most production workloads with registries that support webhooks

**Advantages**:
- Immediate detection when images are pushed
- Minimal resource usage (no periodic polling)
- Lower registry API load
- Faster time-to-update

**Configuration**:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: production-api
  annotations:
    headwind.sh/policy: "patch"
    headwind.sh/event-source: "webhook"  # Default, can be omitted
```

**Requirements**:
- Registry must support webhooks (Docker Hub, Harbor, GitLab, GCR, etc.)
- Headwind must be accessible from the registry (Ingress/LoadBalancer)
- Webhook endpoint configured in registry settings

**Setup**:

1. Expose Headwind webhook service:
   ```yaml
   # Using Ingress
   apiVersion: networking.k8s.io/v1
   kind: Ingress
   metadata:
     name: headwind-webhook
     namespace: headwind-system
   spec:
     rules:
     - host: headwind.example.com
       http:
         paths:
         - path: /webhook
           pathType: Prefix
           backend:
             service:
               name: headwind-webhook
               port:
                 number: 8080
   ```

2. Configure registry webhook:
   - **Docker Hub**: `https://headwind.example.com/webhook/dockerhub`
   - **Generic OCI Registry**: `https://headwind.example.com/webhook/registry`

## Polling Event Source

**Best for**: Registries without webhook support, development environments, or when Headwind is not publicly accessible

**Advantages**:
- Works with any registry (no webhook support required)
- No need for public Headwind endpoint
- Detects image rebuilds (digest changes)
- Discovers new versions automatically

**Configuration**:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: legacy-app
  annotations:
    headwind.sh/policy: "minor"
    headwind.sh/event-source: "polling"
    headwind.sh/polling-interval: "600"  # Poll every 10 minutes (optional)
```

**Global Settings**:

Enable polling globally in Headwind deployment:

```yaml
# deploy/k8s/deployment.yaml
env:
- name: HEADWIND_POLLING_ENABLED
  value: "true"
- name: HEADWIND_POLLING_INTERVAL
  value: "300"  # Default: 5 minutes
```

**Disadvantages**:
- Delayed detection (depends on polling interval)
- Higher registry API usage
- More resource intensive

## Per-Resource Polling Intervals

Override the global polling interval for specific resources:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: critical-app
  annotations:
    headwind.sh/policy: "patch"
    headwind.sh/event-source: "polling"
    headwind.sh/polling-interval: "60"  # Poll every 60 seconds (fast)
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: background-job
  annotations:
    headwind.sh/policy: "minor"
    headwind.sh/event-source: "polling"
    headwind.sh/polling-interval: "1800"  # Poll every 30 minutes (slow)
```

This allows you to:
- Poll critical resources more frequently
- Reduce API load by polling non-critical resources less often
- Optimize resource usage per workload priority

## Both (Redundant Detection)

**Best for**: Critical workloads requiring guaranteed detection

Use both webhooks and polling for maximum reliability:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: critical-service
  annotations:
    headwind.sh/policy: "patch"
    headwind.sh/event-source: "both"
    headwind.sh/polling-interval: "600"  # Fallback polling every 10 minutes
```

**How it works**:
- Responds immediately to webhook events (fast path)
- Also polls registry periodically (fallback path)
- Update is detected via whichever method fires first
- Provides redundancy if webhooks fail or are delayed

**Use cases**:
- Mission-critical applications
- Registries with unreliable webhooks
- During webhook endpoint migrations
- Extra assurance for production deployments

## None (Disable Updates)

**Best for**: Temporarily disabling updates without removing annotations

Disable all update detection:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frozen-app
  annotations:
    headwind.sh/policy: "minor"
    headwind.sh/event-source: "none"  # No updates
```

This resource will:
- Not respond to webhook events
- Not be included in registry polling
- Keep all Headwind annotations intact
- Can be re-enabled by changing event-source

**Use cases**:
- Temporarily freezing deployments during incidents
- Maintenance windows
- Testing scenarios
- Gradual rollout control

## Mixed Configurations

Different resources can use different event sources:

```yaml
# Production API - webhook-only (fastest)
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-production
  namespace: production
  annotations:
    headwind.sh/policy: "patch"
    headwind.sh/event-source: "webhook"
    headwind.sh/require-approval: "true"
---
# Staging API - polling every 5 minutes
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-staging
  namespace: staging
  annotations:
    headwind.sh/policy: "all"
    headwind.sh/event-source: "polling"
    headwind.sh/polling-interval: "300"
    headwind.sh/require-approval: "false"
---
# Background worker - polling every 30 minutes (low priority)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: worker-background
  namespace: production
  annotations:
    headwind.sh/policy: "minor"
    headwind.sh/event-source: "polling"
    headwind.sh/polling-interval: "1800"
---
# Critical database - both methods (redundant)
apiVersion: apps/v1
kind: Deployment
metadata:
  name: database-backup
  namespace: production
  annotations:
    headwind.sh/policy: "patch"
    headwind.sh/event-source: "both"
    headwind.sh/polling-interval: "600"
    headwind.sh/require-approval: "true"
```

## Comparison

| Event Source | Detection Speed | Resource Usage | Registry API Load | Requires Webhook | Best For |
|--------------|----------------|----------------|-------------------|------------------|----------|
| **webhook** | Immediate (~seconds) | Minimal | Minimal | Yes | Production with webhook support |
| **polling** | Delayed (interval) | Medium | Medium-High | No | Development, no webhook support |
| **both** | Immediate + fallback | Medium | Medium-High | Optional | Critical workloads |
| **none** | N/A | None | None | N/A | Temporarily frozen deployments |

## Monitoring

### Metrics

Track event source behavior with Prometheus metrics:

```promql
# Resources filtered from polling (event-source: webhook)
headwind_polling_resources_filtered_total

# Polling cycles completed
headwind_polling_cycles_total

# Images checked during polling
headwind_polling_images_checked_total

# Webhook events received
headwind_webhook_events_total

# Webhook events processed
headwind_webhook_events_processed
```

### Logs

Debug event source filtering:

```bash
# View polling filter decisions
kubectl logs -n headwind-system deployment/headwind | grep "Skipping.*event source"

# View webhook processing
kubectl logs -n headwind-system deployment/headwind | grep "Processing.*webhook"
```

## Resource Type Support

Event sources work with all resource types:

- **Deployments** ✅
- **StatefulSets** ✅
- **DaemonSets** ✅
- **HelmReleases** ✅

Configuration is identical across all resource types.

## Next Steps

- [Configure Deployments](./deployments.md)
- [Configure StatefulSets](./statefulsets.md)
- [Configure DaemonSets](./daemonsets.md)
- [Configure HelmReleases](./helmreleases.md)
- [View Metrics](../api/metrics.md)
