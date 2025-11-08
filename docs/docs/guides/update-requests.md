---
sidebar_position: 1
---

# Working with UpdateRequests

UpdateRequest CRDs are created by Headwind when a new image version is detected and approval is required. They represent pending updates awaiting manual approval.

## Overview

An UpdateRequest is a Kubernetes Custom Resource that tracks:
- Which workload needs updating
- Current and new image versions
- Update policy that triggered it
- Approval status and who approved/rejected it
- Timestamps for all lifecycle events

## Viewing UpdateRequests

### Using kubectl

```bash
# List all UpdateRequests across all namespaces
kubectl get updaterequests -A

# Example output:
NAMESPACE    NAME                    PHASE      AGE
production   nginx-update-v1-27-0    Pending    5m
staging      redis-update-v7-2-0     Pending    2m
dev          postgres-update-15-3    Completed  1h

# Get details of a specific UpdateRequest
kubectl get updaterequest nginx-update-v1-27-0 -n production -o yaml

# Watch for new UpdateRequests in real-time
kubectl get updaterequests -A --watch

# Filter by phase
kubectl get updaterequests -A -o json | jq '.items[] | select(.status.phase == "Pending")'

# Filter by workload kind
kubectl get updaterequests -A -o json | jq '.items[] | select(.spec.targetRef.kind == "Deployment")'
```

### Using kubectl Plugin

```bash
# List all pending updates (formatted output)
kubectl headwind list

# Example output:
Namespace  | Name                 | Deployment       | Current Image | New Image    | Phase
-----------|----------------------|------------------|---------------|--------------|--------
production | nginx-update-v1-27-0 | nginx-deployment | nginx:1.26.0  | nginx:1.27.0 | Pending
```

### Using API

```bash
# List all UpdateRequests
curl http://headwind-api:8081/api/v1/updates

# Get specific UpdateRequest
curl http://headwind-api:8081/api/v1/updates/production/nginx-update-v1-27-0
```

## UpdateRequest Structure

```yaml
apiVersion: headwind.sh/v1alpha1
kind: UpdateRequest
metadata:
  name: nginx-update-v1-27-0
  namespace: production
  creationTimestamp: "2025-11-06T10:00:00Z"
spec:
  targetRef:
    kind: Deployment  # or StatefulSet, DaemonSet, HelmRelease
    name: nginx-deployment
    namespace: production
  containerName: nginx  # For image updates
  currentImage: nginx:1.26.0  # For image updates
  newImage: nginx:1.27.0  # For image updates
  currentVersion: "1.26.0"  # For HelmRelease updates
  newVersion: "1.27.0"  # For HelmRelease updates
  policy: minor  # Update policy that triggered this
status:
  phase: Pending  # Pending, Completed, Rejected, or Failed
  createdAt: "2025-11-06T10:00:00Z"
  lastUpdated: "2025-11-06T10:00:00Z"
  # After approval/rejection:
  approvedBy: "admin@example.com"
  approvedAt: "2025-11-06T10:15:00Z"
  # Or if rejected:
  rejectedBy: "admin@example.com"
  rejectedAt: "2025-11-06T10:15:00Z"
  rejectionReason: "Not ready for production"
```

## UpdateRequest Phases

| Phase | Description |
|-------|-------------|
| `Pending` | Waiting for approval |
| `Completed` | Approved and successfully applied |
| `Rejected` | Rejected by approver |
| `Failed` | Approval granted but update failed to apply |

## Approving Updates

### Using kubectl Plugin (Recommended)

```bash
# Approve with explicit approver
kubectl headwind approve nginx-update-v1-27-0 -n production --approver admin@example.com

# Set default approver
export HEADWIND_APPROVER=admin@example.com
kubectl headwind approve nginx-update-v1-27-0 -n production
```

### Using API

```bash
# Approve update
curl -X POST http://headwind-api:8081/api/v1/updates/production/nginx-update-v1-27-0/approve \
  -H "Content-Type: application/json" \
  -d '{"approver":"admin@example.com"}'

# Response:
{
  "status": "approved",
  "message": "Update approved and executed successfully",
  "updateRequest": "nginx-update-v1-27-0"
}
```

### What Happens on Approval

1. **Status Update**: UpdateRequest phase changes to `Completed`
2. **Execution**: Headwind immediately applies the update to the workload
3. **Tracking**: Approver and approval timestamp recorded
4. **History**: Update added to workload's update history annotation
5. **Notification**: Slack/Teams/webhook notification sent

## Rejecting Updates

### Using kubectl Plugin

```bash
# Reject with reason
kubectl headwind reject nginx-update-v1-27-0 "Not ready for production" \
  -n production --approver admin@example.com

# Reject without explicit reason
kubectl headwind reject nginx-update-v1-27-0 -n production
```

### Using API

```bash
# Reject update
curl -X POST http://headwind-api:8081/api/v1/updates/production/nginx-update-v1-27-0/reject \
  -H "Content-Type: application/json" \
  -d '{
    "approver": "admin@example.com",
    "reason": "Not ready for production"
  }'

# Response:
{
  "status": "rejected",
  "message": "Update rejected",
  "updateRequest": "nginx-update-v1-27-0"
}
```

### What Happens on Rejection

1. **Status Update**: UpdateRequest phase changes to `Rejected`
2. **No Action**: Workload remains unchanged
3. **Tracking**: Rejector, timestamp, and reason recorded
4. **Notification**: Notification sent with rejection reason
5. **Cleanup**: UpdateRequest CRD remains for historical purposes

## Filtering and Querying

### By Namespace

```bash
# All updates in a namespace
kubectl get updaterequests -n production

# All pending updates in a namespace
kubectl get updaterequests -n production -o json | \
  jq '.items[] | select(.status.phase == "Pending")'
```

### By Workload Type

```bash
# All Deployment updates
kubectl get updaterequests -A -o json | \
  jq '.items[] | select(.spec.targetRef.kind == "Deployment")'

# All HelmRelease updates
kubectl get updaterequests -A -o json | \
  jq '.items[] | select(.spec.targetRef.kind == "HelmRelease")'
```

### By Age

```bash
# Updates older than 1 hour
kubectl get updaterequests -A --sort-by=.metadata.creationTimestamp

# Updates from last hour using jq
kubectl get updaterequests -A -o json | \
  jq --arg since "$(date -u -d '1 hour ago' '+%Y-%m-%dT%H:%M:%SZ')" \
  '.items[] | select(.metadata.creationTimestamp > $since)'
```

### By Specific Deployment

```bash
# Find all updates for a specific deployment
kubectl get updaterequests -n production -o json | \
  jq '.items[] | select(.spec.targetRef.name == "nginx-deployment")'
```

## Managing UpdateRequests

### Deleting UpdateRequests

```bash
# Delete completed/rejected updates (cleanup)
kubectl delete updaterequest nginx-update-v1-27-0 -n production

# Bulk delete all completed updates
kubectl get updaterequests -A -o json | \
  jq -r '.items[] | select(.status.phase == "Completed") | "\(.metadata.namespace) \(.metadata.name)"' | \
  while read ns name; do kubectl delete updaterequest "$name" -n "$ns"; done
```

:::warning
Deleting a Pending UpdateRequest will prevent the update from being applied. Only delete UpdateRequests you're sure you want to cancel.
:::

### Auto-Cleanup

Headwind does not automatically delete UpdateRequests. They remain as historical records. You can set up a CronJob for cleanup:

```yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: cleanup-old-updaterequests
  namespace: headwind-system
spec:
  schedule: "0 0 * * *"  # Daily at midnight
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: headwind
          containers:
          - name: cleanup
            image: bitnami/kubectl:latest
            command:
            - /bin/bash
            - -c
            - |
              # Delete completed/rejected updates older than 7 days
              kubectl get updaterequests -A -o json | \
                jq -r '.items[] | select(.status.phase == "Completed" or .status.phase == "Rejected") | select((.metadata.creationTimestamp | fromdateiso8601) < (now - (7 * 24 * 60 * 60))) | "\(.metadata.namespace) \(.metadata.name)"' | \
                while read ns name; do
                  kubectl delete updaterequest "$name" -n "$ns"
                done
          restartPolicy: Never
```

## Monitoring UpdateRequests

### Prometheus Metrics

```promql
# Pending updates
headwind_updates_pending

# Approved updates (total)
headwind_updates_approved_total

# Rejected updates (total)
headwind_updates_rejected_total

# Successfully applied updates
headwind_updates_applied_total

# Failed update attempts
headwind_updates_failed_total
```

### Alerting

Create alerts for stale UpdateRequests:

```yaml
groups:
- name: headwind_updates
  rules:
  - alert: StaleUpdateRequests
    expr: headwind_updates_pending > 10
    for: 1h
    annotations:
      summary: "Many pending UpdateRequests"
      description: "{{ $value }} UpdateRequests pending for over 1 hour"

  - alert: HighUpdateFailureRate
    expr: rate(headwind_updates_failed_total[5m]) > 0.1
    for: 5m
    annotations:
      summary: "High update failure rate"
      description: "Update failures detected"
```

## Common Workflows

### Bulk Approval

Approve all pending updates in a namespace:

```bash
kubectl get updaterequests -n staging -o json | \
  jq -r '.items[] | select(.status.phase == "Pending") | .metadata.name' | \
  while read name; do
    kubectl headwind approve "$name" -n staging --approver ci-bot@example.com
  done
```

### Review Before Approval

```bash
# 1. List pending updates
kubectl headwind list

# 2. Get details of specific update
kubectl get updaterequest nginx-update-v1-27-0 -n production -o yaml

# 3. Check deployment history
kubectl headwind history nginx-deployment -n production

# 4. Make decision
kubectl headwind approve nginx-update-v1-27-0 -n production --approver admin@example.com
# OR
kubectl headwind reject nginx-update-v1-27-0 "Waiting for security scan" -n production
```

### Automated Approval Based on Environment

```bash
#!/bin/bash
# Auto-approve staging, require manual approval for production

NAMESPACE=$1
UPDATE_REQUEST=$2

if [ "$NAMESPACE" == "staging" ] || [ "$NAMESPACE" == "dev" ]; then
    echo "Auto-approving $UPDATE_REQUEST in $NAMESPACE"
    kubectl headwind approve "$UPDATE_REQUEST" -n "$NAMESPACE" --approver auto-approve-bot@example.com
else
    echo "Manual approval required for $UPDATE_REQUEST in $NAMESPACE"
    # Send notification to Slack/Teams
fi
```

## Troubleshooting

### UpdateRequest Not Created

Check if Headwind detected the update:

```bash
# Check Headwind logs
kubectl logs -n headwind-system deployment/headwind | grep -i "update request"

# Verify deployment has annotations
kubectl get deployment nginx-deployment -n production -o jsonpath='{.metadata.annotations}'

# Check webhook/polling events
kubectl logs -n headwind-system deployment/headwind | grep -i webhook
```

### Update Not Applied After Approval

Check UpdateRequest status:

```bash
kubectl get updaterequest nginx-update-v1-27-0 -n production -o yaml

# Look for status.phase: Failed
# Check status.message for error details
```

Check Headwind logs for errors:

```bash
kubectl logs -n headwind-system deployment/headwind | grep -i "execute update\|failed"
```

### Stale Pending Updates

If updates remain Pending indefinitely:

1. Check if they should be approved or rejected
2. Consider implementing auto-cleanup
3. Review approval workflow

## Next Steps

- [Approval Workflow](../configuration/approval-workflow.md) - Configure approval process
- [kubectl Plugin](./kubectl-plugin.md) - Command-line tools
- [API Reference](../api/) - API documentation
- [Rollback](../configuration/rollback.md) - Rollback failed updates
