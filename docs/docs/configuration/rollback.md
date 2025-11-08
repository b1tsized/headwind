---
sidebar_position: 8
---

# Rollback Configuration

Headwind provides both automatic and manual rollback capabilities to quickly recover from failed deployments. All updates are tracked in deployment annotations, allowing you to rollback to any previous version.

## Overview

Headwind supports two types of rollbacks:

- **Automatic Rollback**: Monitors deployment health after updates and automatically reverts on failures
- **Manual Rollback**: Use the API or kubectl plugin to rollback to a previous version at any time

## Automatic Rollback

### Configuration

Enable automatic rollback using annotations:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
  namespace: production
  annotations:
    headwind.sh/policy: "minor"

    # Enable automatic rollback
    headwind.sh/auto-rollback: "true"

    # How long to monitor deployment health (default: 300s)
    headwind.sh/rollback-timeout: "300"

    # Number of failed health checks before rollback (default: 3)
    headwind.sh/health-check-retries: "3"
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: app
        image: myapp:1.0.0
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          periodSeconds: 30
```

### Annotations

| Annotation | Type | Default | Description |
|------------|------|---------|-------------|
| `headwind.sh/auto-rollback` | boolean | `false` | Enable automatic rollback on failures |
| `headwind.sh/rollback-timeout` | integer | `300` | Health check monitoring duration (seconds) |
| `headwind.sh/health-check-retries` | integer | `3` | Failed health checks before rollback |

### Failure Detection

Automatic rollback is triggered when Headwind detects any of the following conditions:

**Pod Failures:**
- **CrashLoopBackOff**: Pods repeatedly crashing after update
- **ImagePullBackOff**: Unable to pull the new image
- **High restart count**: Container restarts exceed 5 times

**Readiness Failures:**
- Pods not becoming Ready within the timeout period
- Readiness probe failures exceeding retry threshold

**Deployment Conditions:**
- **ProgressDeadlineExceeded**: Deployment fails to progress
- Deployment stuck in updating state beyond timeout

### Workflow

When a failure is detected:

1. **Detection**: Headwind monitors pod status and deployment conditions
2. **Validation**: Confirms failure criteria met (health check retries exceeded)
3. **Decision**: Determines rollback is necessary
4. **Execution**: Reverts container image to previous version
5. **Notification**: Sends Slack/Teams/webhook notification
6. **Tracking**: Records rollback in update history
7. **Monitoring**: Continues monitoring rolled-back deployment

### Example: Automatic Rollback in Action

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-server
  namespace: production
  annotations:
    headwind.sh/policy: "patch"
    headwind.sh/require-approval: "true"
    headwind.sh/auto-rollback: "true"
    headwind.sh/rollback-timeout: "600"  # Monitor for 10 minutes
    headwind.sh/health-check-retries: "2"  # Rollback after 2 failures
spec:
  replicas: 5
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 1
  template:
    spec:
      containers:
      - name: api
        image: api-server:2.5.0
        ports:
        - containerPort: 8080
        readinessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 10
          failureThreshold: 3
        livenessProbe:
          httpGet:
            path: /api/health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 30
```

**Scenario**: Update to `api-server:2.5.1` is approved and applied

1. Headwind applies the update
2. Kubernetes begins rolling update
3. New pods start with v2.5.1
4. Readiness probes fail repeatedly (bug in v2.5.1)
5. After 2 failed health checks, Headwind triggers rollback
6. Image reverted to `api-server:2.5.0`
7. Kubernetes rolls back to working version
8. Notification sent: "Automatic Rollback: api-server CrashLoopBackOff"

## Manual Rollback

### Using kubectl Plugin

The easiest way to rollback manually:

```bash
# Install plugin
sudo cp kubectl-headwind /usr/local/bin/
sudo chmod +x /usr/local/bin/kubectl-headwind

# Rollback to previous version (auto-detects first container)
kubectl headwind rollback my-app -n production

# Rollback specific container
kubectl headwind rollback my-app app-container -n production

# View update history first
kubectl headwind history my-app -n production
```

### Using API Directly

```bash
# Get update history
curl http://headwind-api:8081/api/v1/rollback/production/my-app/history

# Rollback to previous image
curl -X POST http://headwind-api:8081/api/v1/rollback/production/my-app/app-container
```

### Using kubectl (Native)

You can also use native kubectl rollback:

```bash
# Rollback to previous revision
kubectl rollout undo deployment/my-app -n production

# Rollback to specific revision
kubectl rollout undo deployment/my-app --to-revision=2 -n production

# View rollout history
kubectl rollout history deployment/my-app -n production
```

:::info
Native kubectl rollback uses Kubernetes' built-in revision history, while Headwind rollback uses the update history tracked in annotations. Both work, but Headwind provides more context about images and approvers.
:::

## Update History

All updates are automatically tracked in deployment annotations.

### View History

```bash
# Using kubectl plugin
kubectl headwind history my-app -n production

# Using kubectl directly
kubectl get deployment my-app -n production \
  -o jsonpath='{.metadata.annotations.headwind\.sh/update-history}' | jq

# Using API
curl http://headwind-api:8081/api/v1/rollback/production/my-app/history
```

### History Format

```json
[
  {
    "container": "app",
    "image": "myapp:v1.2.0",
    "timestamp": "2025-11-06T10:30:00Z",
    "updateRequestName": "myapp-update-v1-2-0",
    "approvedBy": "admin@example.com"
  },
  {
    "container": "app",
    "image": "myapp:v1.1.0",
    "timestamp": "2025-11-05T14:20:00Z",
    "updateRequestName": "myapp-update-v1-1-0",
    "approvedBy": "webhook"
  },
  {
    "container": "app",
    "image": "myapp:v1.0.0",
    "timestamp": "2025-11-01T09:15:00Z",
    "updateRequestName": "myapp-update-v1-0-0",
    "approvedBy": "admin@example.com"
  }
]
```

### History Retention

Headwind keeps the last **10 updates** per container. Older entries are automatically removed.

## Metrics

Monitor rollback operations with Prometheus:

```promql
# Total rollback operations
headwind_rollbacks_total

# Manual rollbacks
headwind_rollbacks_manual_total

# Automatic rollbacks
headwind_rollbacks_automatic_total

# Failed rollback operations
headwind_rollbacks_failed_total

# Deployment health checks performed
headwind_deployment_health_checks_total

# Health check failures detected
headwind_deployment_health_failures_total
```

### Alerting

Create alerts for rollback events:

```yaml
groups:
- name: headwind_rollbacks
  rules:
  - alert: FrequentRollbacks
    expr: rate(headwind_rollbacks_total[1h]) > 3
    for: 5m
    annotations:
      summary: "Frequent rollbacks detected"
      description: "{{ $value }} rollbacks in the last hour"

  - alert: AutomaticRollbackTriggered
    expr: increase(headwind_rollbacks_automatic_total[5m]) > 0
    annotations:
      summary: "Automatic rollback triggered"
      description: "Headwind triggered an automatic rollback"

  - alert: RollbackFailed
    expr: increase(headwind_rollbacks_failed_total[5m]) > 0
    annotations:
      summary: "Rollback operation failed"
      description: "A rollback operation has failed"
```

## Best Practices

### 1. Always Enable Auto-Rollback in Production

```yaml
annotations:
  headwind.sh/auto-rollback: "true"
  headwind.sh/rollback-timeout: "600"  # 10 minutes
  headwind.sh/health-check-retries: "2"  # Quick response
```

### 2. Configure Proper Health Checks

Automatic rollback depends on health checks:

```yaml
readinessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 10
  failureThreshold: 3  # Must fail 3 times to be considered unhealthy

livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 30
```

### 3. Set Appropriate Timeouts

Match timeout to application startup time:

```yaml
# Fast-starting apps
headwind.sh/rollback-timeout: "180"  # 3 minutes

# Slow-starting apps (databases, Java apps)
headwind.sh/rollback-timeout: "900"  # 15 minutes
```

### 4. Test Rollback Procedures

Periodically test rollback works:

```bash
# 1. Deploy intentionally broken version
kubectl set image deployment/my-app app=my-app:broken -n staging

# 2. Wait for automatic rollback (if enabled)
kubectl get pods -n staging -w

# 3. Or manually rollback
kubectl headwind rollback my-app -n staging

# 4. Verify rollback succeeded
kubectl headwind history my-app -n staging
```

### 5. Monitor Rollback Metrics

Set up dashboards and alerts:

```promql
# Rollback rate
rate(headwind_rollbacks_total[1h])

# Automatic vs manual rollbacks
headwind_rollbacks_automatic_total / headwind_rollbacks_total

# Rollback success rate
(headwind_rollbacks_total - headwind_rollbacks_failed_total) / headwind_rollbacks_total
```

### 6. Review Rollback History

Regularly review what's being rolled back and why:

```bash
# Check recent rollbacks across all deployments
kubectl get updaterequests -A | grep Rejected

# Review health check failures
kubectl logs -n headwind-system deployment/headwind | grep "health check failed"
```

## Troubleshooting

### Automatic Rollback Not Triggering

**Check health check configuration:**

```bash
# Verify readinessProbe is configured
kubectl get deployment my-app -o yaml | grep -A 10 readinessProbe

# Check pod status
kubectl get pods -n production -l app=my-app

# View pod events
kubectl describe pod my-app-xyz -n production
```

**Check Headwind logs:**

```bash
kubectl logs -n headwind-system deployment/headwind | grep -i "rollback\|health"
```

**Verify annotations:**

```bash
kubectl get deployment my-app -n production \
  -o jsonpath='{.metadata.annotations}' | jq
```

### Manual Rollback Fails

**Check update history exists:**

```bash
kubectl headwind history my-app -n production
```

**Check API connectivity:**

```bash
# Port forward if needed
kubectl port-forward -n headwind-system svc/headwind-api 8081:8081

# Test API
curl http://localhost:8081/api/v1/rollback/production/my-app/history
```

**Check permissions:**

```bash
# Verify Headwind ServiceAccount has update permissions
kubectl auth can-i update deployments --as=system:serviceaccount:headwind-system:headwind -n production
```

### History Not Being Tracked

**Verify annotation is being set:**

```bash
kubectl get deployment my-app -n production \
  -o jsonpath='{.metadata.annotations.headwind\.sh/update-history}'
```

**Check for annotation size limits:**

Kubernetes annotations have a size limit. If history is very long, older entries are automatically pruned to keep the last 10 updates.

### Rollback to Wrong Version

**View full history before rollback:**

```bash
kubectl headwind history my-app -n production
```

**Rollback rolls back to N-1 (previous version)**. To rollback to a specific version, use native kubectl:

```bash
# View revision history
kubectl rollout history deployment/my-app -n production

# Rollback to specific revision
kubectl rollout undo deployment/my-app --to-revision=3 -n production
```

## Integration with CI/CD

### Automated Rollback on Test Failures

```bash
#!/bin/bash
# deploy-with-tests.sh

DEPLOYMENT=$1
NAMESPACE=$2

# Deploy via Headwind approval
kubectl headwind approve "$DEPLOYMENT-update" -n "$NAMESPACE" --approver ci-bot@example.com

# Wait for rollout
kubectl rollout status deployment/"$DEPLOYMENT" -n "$NAMESPACE" --timeout=5m

# Run smoke tests
if ! ./run-smoke-tests.sh "$DEPLOYMENT" "$NAMESPACE"; then
    echo "Smoke tests failed! Rolling back..."
    kubectl headwind rollback "$DEPLOYMENT" -n "$NAMESPACE"
    exit 1
fi

echo "Deployment successful!"
```

### Monitor Rollbacks in CI

```bash
# Check if automatic rollback occurred
ROLLBACKS=$(kubectl logs -n headwind-system deployment/headwind --since=5m | grep -c "automatic rollback triggered")

if [ "$ROLLBACKS" -gt 0 ]; then
    echo "Automatic rollback detected - deployment failed"
    exit 1
fi
```

## Next Steps

- [Notifications](./notifications.md) - Get notified about rollbacks
- [Approval Workflow](./approval-workflow.md) - Configure update approvals
- [Metrics Reference](../api/metrics.md) - Monitor rollback metrics
- [kubectl Plugin Guide](../guides/kubectl-plugin.md) - Manual rollback commands
