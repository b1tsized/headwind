---
sidebar_position: 5
---

# Configuring HelmReleases

Headwind provides full support for Flux HelmRelease resources with **automatic chart version discovery**. Unlike image-based updates, Headwind actively queries Helm repositories to discover new chart versions and automatically applies semantic versioning policies.

## Overview

Headwind watches Flux HelmRelease CRDs and:
1. Automatically queries the referenced HelmRepository for available chart versions
2. Uses the PolicyEngine to find the best matching version based on your policy
3. Compares discovered versions with `status.lastAttemptedRevision` or `spec.chart.spec.version`
4. Either creates an UpdateRequest CRD (if approval required) or applies the update directly
5. Sends notifications about the update

## Prerequisites

Headwind requires the HelmRepository CRD to query Helm repositories:

**If you have Flux CD installed:** The CRD already exists - no action needed!

**If you DON'T have Flux CD:** Apply the HelmRepository CRD:

```bash
kubectl apply -f deploy/k8s/crds/helmrepository.yaml
```

## Supported Annotations

| Annotation | Type | Default | Description |
|------------|------|---------|-------------|
| `headwind.sh/policy` | string | `none` | Update policy: `none`, `patch`, `minor`, `major`, `all`, `glob`, `force` |
| `headwind.sh/pattern` | string | - | Glob pattern (required for `glob` policy) |
| `headwind.sh/require-approval` | boolean | `true` | Whether updates require manual approval |
| `headwind.sh/min-update-interval` | integer | `300` | Minimum seconds between updates |

## Repository Types

Headwind supports both traditional HTTP Helm repositories and modern OCI registries.

### HTTP Helm Repository

Traditional Helm repositories using `index.yaml`:

```yaml
apiVersion: source.toolkit.fluxcd.io/v1
kind: HelmRepository
metadata:
  name: my-repo
  namespace: default
spec:
  url: https://charts.example.com  # HTTP(S) URL
  interval: 5m
  type: default
```

**Popular HTTP repositories:**
- Bitnami: `https://charts.bitnami.com/bitnami`
- Jetstack: `https://charts.jetstack.io`
- Prometheus Community: `https://prometheus-community.github.io/helm-charts`
- Grafana: `https://grafana.github.io/helm-charts`

### OCI Registry

Modern OCI-based Helm chart storage:

```yaml
apiVersion: source.toolkit.fluxcd.io/v1
kind: HelmRepository
metadata:
  name: my-oci-repo
  namespace: default
spec:
  url: oci://registry.example.com/helm-charts  # OCI URL
  interval: 5m
  type: oci
```

**Popular OCI registries:**
- AWS ECR: `oci://123456789.dkr.ecr.us-west-2.amazonaws.com/charts`
- Google Artifact Registry: `oci://us-docker.pkg.dev/project/charts`
- Azure ACR: `oci://myregistry.azurecr.io/helm`
- GitHub GHCR: `oci://ghcr.io/myorg/charts`
- Harbor: `oci://harbor.example.com/charts`
- JFrog Artifactory: `oci://myorg.jfrog.io/charts`

:::warning Known Limitation
Due to a limitation in the underlying `oci-distribution` Rust crate (v0.11), OCI Helm repositories may incorrectly query Docker Hub when the chart name matches a common Docker image name (e.g., `busybox`, `nginx`, `redis`, `postgres`). This results in discovering Docker container image tags instead of Helm chart versions.

**Workaround**: Use traditional HTTP Helm repositories (fully supported) or ensure your OCI Helm chart names don't conflict with popular Docker Hub image names.
:::

## Basic Configuration

### Simple Auto-Discovery

```yaml
apiVersion: source.toolkit.fluxcd.io/v1
kind: HelmRepository
metadata:
  name: bitnami
  namespace: default
spec:
  url: https://charts.bitnami.com/bitnami
  interval: 5m
---
apiVersion: helm.toolkit.fluxcd.io/v2
kind: HelmRelease
metadata:
  name: wordpress
  namespace: default
  annotations:
    # Allow minor version updates
    headwind.sh/policy: "minor"

    # Require approval
    headwind.sh/require-approval: "true"

    # Check every 5 minutes (default)
    headwind.sh/min-update-interval: "300"
spec:
  interval: 5m
  chart:
    spec:
      chart: wordpress
      version: "15.0.0"  # Headwind monitors this version
      sourceRef:
        kind: HelmRepository
        name: bitnami
        namespace: default
  values:
    wordpressUsername: admin
    wordpressPassword: changeme
```

### Auto-Update Without Approval

For development environments:

```yaml
apiVersion: helm.toolkit.fluxcd.io/v2
kind: HelmRelease
metadata:
  name: nginx-dev
  namespace: dev
  annotations:
    # Allow any version update
    headwind.sh/policy: "all"

    # Auto-update without approval
    headwind.sh/require-approval: "false"

    # Update every hour max
    headwind.sh/min-update-interval: "3600"
spec:
  interval: 5m
  chart:
    spec:
      chart: nginx
      version: "13.0.0"
      sourceRef:
        kind: HelmRepository
        name: bitnami
```

## Update Workflow

When Headwind discovers a new chart version:

1. **Repository Query**: Headwind queries the HelmRepository for available versions
2. **Policy Evaluation**: Compares versions using PolicyEngine (semver-aware)
3. **Interval Check**: Ensures minimum update interval has elapsed
4. **Decision Path**:
   - **If `require-approval: true`**: Creates UpdateRequest CRD
   - **If `require-approval: false`**: Applies update directly (respects min-update-interval)
5. **Notification**: Sends Slack/Teams/webhook notification
6. **Update**: Patches `spec.chart.spec.version` via Kubernetes API
7. **Flux Reconciliation**: Flux detects change and deploys new chart version

## Production Example

PostgreSQL with conservative update policy:

```yaml
apiVersion: source.toolkit.fluxcd.io/v1
kind: HelmRepository
metadata:
  name: bitnami
  namespace: production
spec:
  url: https://charts.bitnami.com/bitnami
  interval: 10m
---
apiVersion: helm.toolkit.fluxcd.io/v2
kind: HelmRelease
metadata:
  name: postgresql
  namespace: production
  annotations:
    # Only patch versions (bug fixes)
    headwind.sh/policy: "patch"

    # Require manual approval
    headwind.sh/require-approval: "true"

    # Wait at least 7 days between updates
    headwind.sh/min-update-interval: "604800"
spec:
  interval: 10m
  chart:
    spec:
      chart: postgresql
      version: "12.5.0"
      sourceRef:
        kind: HelmRepository
        name: bitnami
        namespace: production
  values:
    auth:
      username: admin
      password: "${POSTGRES_PASSWORD}"
      database: mydb
    primary:
      persistence:
        enabled: true
        size: 100Gi
    readReplicas:
      replicaCount: 2
```

## Monitoring Stack Example

Prometheus stack with glob pattern for stable releases:

```yaml
apiVersion: source.toolkit.fluxcd.io/v1
kind: HelmRepository
metadata:
  name: prometheus-community
  namespace: monitoring
spec:
  url: https://prometheus-community.github.io/helm-charts
  interval: 5m
---
apiVersion: helm.toolkit.fluxcd.io/v2
kind: HelmRelease
metadata:
  name: kube-prometheus-stack
  namespace: monitoring
  annotations:
    # Only stable releases
    headwind.sh/policy: "glob"
    headwind.sh/pattern: "*-stable"

    # Require approval
    headwind.sh/require-approval: "true"

    # Wait 3 days between updates
    headwind.sh/min-update-interval: "259200"
spec:
  interval: 10m
  chart:
    spec:
      chart: kube-prometheus-stack
      version: "45.0.0-stable"
      sourceRef:
        kind: HelmRepository
        name: prometheus-community
        namespace: monitoring
  values:
    prometheus:
      prometheusSpec:
        retention: 30d
        storageSpec:
          volumeClaimTemplate:
            spec:
              accessModes: ["ReadWriteOnce"]
              resources:
                requests:
                  storage: 100Gi
    grafana:
      adminPassword: "${GRAFANA_PASSWORD}"
```

## Private Repository Authentication

### HTTP Repositories with Basic Auth

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: helm-repo-credentials
  namespace: default
type: Opaque
stringData:
  username: myusername
  password: mypassword
---
apiVersion: source.toolkit.fluxcd.io/v1
kind: HelmRepository
metadata:
  name: private-repo
  namespace: default
spec:
  url: https://charts.example.com
  interval: 5m
  secretRef:
    name: helm-repo-credentials  # Basic auth credentials
```

### OCI Registries with Credentials

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: oci-registry-credentials
  namespace: default
type: Opaque
stringData:
  username: myusername
  password: mytoken
---
apiVersion: source.toolkit.fluxcd.io/v1
kind: HelmRepository
metadata:
  name: private-oci
  namespace: default
spec:
  url: oci://registry.example.com/charts
  interval: 5m
  type: oci
  secretRef:
    name: oci-registry-credentials
```

### Docker Config for OCI

For registries requiring Docker config format:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: docker-config
  namespace: default
type: kubernetes.io/dockerconfigjson
data:
  .dockerconfigjson: <base64-encoded-docker-config>
---
apiVersion: source.toolkit.fluxcd.io/v1
kind: HelmRepository
metadata:
  name: ecr-charts
  namespace: default
spec:
  url: oci://123456789.dkr.ecr.us-west-2.amazonaws.com/charts
  type: oci
  secretRef:
    name: docker-config
```

## Configuration Options

### Automatic Version Discovery

Enabled by default. To disable globally:

```yaml
# deploy/k8s/deployment.yaml
env:
- name: HEADWIND_HELM_AUTO_DISCOVERY
  value: "false"
```

### Polling for HelmReleases

When registry polling is enabled, Headwind also polls Helm repositories:

```yaml
# deploy/k8s/deployment.yaml
env:
- name: HEADWIND_POLLING_ENABLED
  value: "true"
- name: HEADWIND_POLLING_INTERVAL
  value: "300"  # Poll every 5 minutes
```

Polling discovers HelmReleases with headwind annotations, queries their repositories, and creates UpdateRequests when new versions are found.

## Monitoring Updates

### View UpdateRequests

```bash
# List all HelmRelease updates
kubectl get updaterequests -A -o json | \
  jq '.items[] | select(.spec.targetRef.kind == "HelmRelease")'

# Get specific HelmRelease update
kubectl get updaterequest wordpress-update-15-1-0 -o yaml
```

### Example UpdateRequest

```yaml
apiVersion: headwind.sh/v1alpha1
kind: UpdateRequest
metadata:
  name: wordpress-update-15-1-0
  namespace: default
spec:
  targetRef:
    kind: HelmRelease
    name: wordpress
    namespace: default
  currentVersion: "15.0.0"
  newVersion: "15.1.0"
  policy: minor
status:
  phase: Pending
  createdAt: "2025-11-06T10:30:00Z"
```

### Approve Updates

Via API:

```bash
curl -X POST http://headwind-api:8081/api/v1/updates/default/wordpress-update-15-1-0/approve \
  -H "Content-Type: application/json" \
  -d '{"approver":"admin@example.com"}'
```

Via kubectl plugin:

```bash
kubectl headwind approve wordpress-update-15-1-0 --approver admin@example.com
```

### Check Update History

```bash
# View Flux reconciliation history
kubectl describe helmrelease wordpress -n default

# Check Flux events
kubectl get events -n default --field-selector involvedObject.name=wordpress
```

## Metrics

Headwind provides comprehensive metrics for Helm chart monitoring:

```promql
# HelmReleases being watched
headwind_helm_releases_watched

# Chart version checks performed
headwind_helm_chart_versions_checked_total

# Updates discovered
headwind_helm_updates_found_total

# Updates approved by policy
headwind_helm_updates_approved_total

# Updates rejected by policy
headwind_helm_updates_rejected_total

# Chart updates successfully applied
headwind_helm_updates_applied_total

# Repository queries performed
headwind_helm_repository_queries_total

# Repository query errors
headwind_helm_repository_errors_total

# Repository query duration
headwind_helm_repository_query_duration_seconds
```

## Best Practices

### 1. Conservative Policies for Production

```yaml
annotations:
  headwind.sh/policy: "patch"  # Only bug fixes
  headwind.sh/require-approval: "true"  # Always approve
  headwind.sh/min-update-interval: "604800"  # Wait 1 week
```

### 2. Test in Staging First

Use different policies per environment:

```yaml
# Production - conservative
headwind.sh/policy: "patch"
headwind.sh/require-approval: "true"

# Staging - permissive
headwind.sh/policy: "minor"
headwind.sh/require-approval: "false"

# Development - aggressive
headwind.sh/policy: "all"
headwind.sh/require-approval: "false"
headwind.sh/min-update-interval: "3600"  # 1 hour
```

### 3. Use HTTP Repositories When Possible

Until OCI limitations are resolved:

```yaml
# Preferred
url: https://charts.bitnami.com/bitnami

# Works but has limitations with common chart names
url: oci://registry.example.com/charts
```

### 4. Monitor Repository Availability

Track repository query errors:

```promql
rate(headwind_helm_repository_errors_total[5m]) > 0
```

### 5. Set Appropriate Intervals

Balance freshness vs load:

```yaml
# Critical apps - longer interval
headwind.sh/min-update-interval: "604800"  # 1 week

# Non-critical - shorter interval
headwind.sh/min-update-interval: "86400"  # 1 day
```

### 6. Use Flux's Built-in Features

Combine with Flux capabilities:

```yaml
spec:
  # Flux will retry failed installations
  install:
    remediation:
      retries: 3

  # Flux will retry failed upgrades
  upgrade:
    remediation:
      retries: 3

  # Test after deployment
  test:
    enable: true

  # Rollback on failure
  rollback:
    cleanupOnFail: true
```

## Troubleshooting

### Updates Not Detected

Check repository query errors:

```bash
# View Headwind logs
kubectl logs -n headwind-system deployment/headwind | grep helm

# Check repository status
kubectl get helmrepository -A

# Test repository manually
curl https://charts.bitnami.com/bitnami/index.yaml
```

### OCI Repository Issues

If discovering wrong versions (Docker images instead of charts):

1. Verify chart name doesn't match common Docker images
2. Check repository URL is correct
3. Consider switching to HTTP repository
4. Check logs for OCI errors:

```bash
kubectl logs -n headwind-system deployment/headwind | grep -i oci
```

### Authentication Failures

For private repositories:

```bash
# Check secret exists
kubectl get secret helm-repo-credentials -n default

# Verify secret format
kubectl get secret helm-repo-credentials -o jsonpath='{.data.username}' | base64 -d
kubectl get secret helm-repo-credentials -o jsonpath='{.data.password}' | base64 -d

# Check Headwind logs
kubectl logs -n headwind-system deployment/headwind | grep -i auth
```

### Version Not Matching Policy

Verify policy configuration:

```bash
# Check HelmRelease annotations
kubectl get helmrelease wordpress -o jsonpath='{.metadata.annotations}'

# Test policy manually
# Current: 15.0.0, New: 15.1.0, Policy: minor -> Should match
# Current: 15.0.0, New: 16.0.0, Policy: minor -> Should NOT match
```

### Flux Not Reconciling

After Headwind updates the version:

```bash
# Check Flux HelmRelease status
kubectl get helmrelease wordpress -o yaml

# Force reconciliation
flux reconcile helmrelease wordpress

# Check Flux logs
kubectl logs -n flux-system deployment/helm-controller
```

## Integration with Flux

Headwind complements Flux CD:

- **Headwind**: Discovers new chart versions and manages approval workflow
- **Flux**: Handles actual chart installation and upgrades

```
┌─────────────────┐
│  Helm Repo      │
│  (Charts)       │
└────────┬────────┘
         │
         ├─────────────┐
         │             │
         ▼             ▼
  ┌──────────┐   ┌──────────┐
  │ Headwind │   │   Flux   │
  │ (Watch)  │   │ (Deploy) │
  └────┬─────┘   └────▲─────┘
       │              │
       │ Updates      │
       │ spec.chart   │
       │ .spec.version│
       │              │
       └──────────────┘
```

## Next Steps

- [Update Policies](../update-policies.md) - Understand semantic versioning
- [Approval Workflow](./approval-workflow.md) - Configure approval process
- [Notifications](./notifications.md) - Set up Slack/Teams alerts
- [Working with UpdateRequests](../guides/update-requests.md) - Manage updates
