---
sidebar_position: 6
---

# Notifications

Headwind can send notifications about deployment updates to Slack, Microsoft Teams, or generic webhooks. Get notified when updates are discovered, approved, applied, or when rollbacks occur.

## Overview

Headwind sends notifications for the following events:

- **UpdateRequestCreated**: New UpdateRequest CRD created (requires approval)
- **UpdateApproved**: Update approved by user
- **UpdateRejected**: Update rejected by user
- **UpdateCompleted**: Update successfully applied
- **UpdateFailed**: Update failed to apply
- **RollbackTriggered**: Automatic rollback triggered due to health check failure
- **RollbackCompleted**: Rollback completed successfully
- **RollbackFailed**: Rollback failed

## Slack Integration

### Setup

1. Create a Slack incoming webhook:
   - Go to https://api.slack.com/apps
   - Create an app or select an existing one
   - Enable "Incoming Webhooks"
   - Add a webhook to your workspace
   - Copy the webhook URL

2. Configure Headwind deployment:

```yaml
# deploy/k8s/deployment.yaml
env:
- name: SLACK_ENABLED
  value: "true"
- name: SLACK_WEBHOOK_URL
  value: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

# Optional: Override webhook defaults
- name: SLACK_CHANNEL  # Optional: override webhook channel
  value: "#deployments"
- name: SLACK_USERNAME  # Optional: customize bot name
  value: "Headwind Bot"
- name: SLACK_ICON_EMOJI  # Optional: customize bot icon
  value: ":rocket:"
```

### Using Kubernetes Secrets

Store the webhook URL securely:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: slack-webhook
  namespace: headwind-system
type: Opaque
stringData:
  url: https://hooks.slack.com/services/YOUR/WEBHOOK/URL
---
# deploy/k8s/deployment.yaml
env:
- name: SLACK_ENABLED
  value: "true"
- name: SLACK_WEBHOOK_URL
  valueFrom:
    secretKeyRef:
      name: slack-webhook
      key: url
```

### Message Format

Slack notifications use Block Kit for rich formatting:

**Update Request Created:**
```
🔔 New Update Request
Namespace: production
Deployment: nginx-deployment
Container: nginx
Current Image: nginx:1.26.0
New Image: nginx:1.27.0
Policy: minor
```

**Update Approved:**
```
✅ Update Approved
Update Request: nginx-update-v1-27-0
Approved By: admin@example.com
Deployment: nginx-deployment (production)
Image: nginx:1.26.0 → nginx:1.27.0
```

**Update Completed:**
```
🚀 Update Completed
Deployment: nginx-deployment (production)
Container: nginx
Old Image: nginx:1.26.0
New Image: nginx:1.27.0
Approved By: admin@example.com
```

**Rollback Triggered:**
```
⚠️  Automatic Rollback Triggered
Deployment: nginx-deployment (production)
Container: nginx
Reason: CrashLoopBackOff
Current Image: nginx:1.27.0
Rolling back to: nginx:1.26.0
```

## Microsoft Teams Integration

### Setup

1. Create a Teams incoming webhook:
   - Open Teams and go to the channel
   - Click "..." → "Connectors"
   - Add "Incoming Webhook"
   - Name it and copy the URL

2. Configure Headwind deployment:

```yaml
# deploy/k8s/deployment.yaml
env:
- name: TEAMS_ENABLED
  value: "true"
- name: TEAMS_WEBHOOK_URL
  value: "https://outlook.office.com/webhook/YOUR-WEBHOOK-URL"
```

### Using Kubernetes Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: teams-webhook
  namespace: headwind-system
type: Opaque
stringData:
  url: https://outlook.office.com/webhook/YOUR-WEBHOOK-URL
---
# deploy/k8s/deployment.yaml
env:
- name: TEAMS_ENABLED
  value: "true"
- name: TEAMS_WEBHOOK_URL
  valueFrom:
    secretKeyRef:
      name: teams-webhook
      key: url
```

### Message Format

Teams notifications use Adaptive Cards with:
- Color themes matching event severity
- Structured fact display
- Action buttons for approvals
- Kubernetes logo branding

## Generic Webhook Integration

For custom integrations, PagerDuty, Opsgenie, or custom notification systems.

### Setup

```yaml
# deploy/k8s/deployment.yaml
env:
- name: WEBHOOK_ENABLED
  value: "true"
- name: WEBHOOK_URL
  value: "https://your-webhook-endpoint.com/notifications"

# Optional: HMAC signature verification
- name: WEBHOOK_SECRET
  value: "your-secret-key"

# Optional: timeout in seconds (default: 10)
- name: WEBHOOK_TIMEOUT
  value: "10"

# Optional: max retries (default: 3)
- name: WEBHOOK_MAX_RETRIES
  value: "3"
```

### Payload Format

Headwind sends JSON payloads:

```json
{
  "event": "update_completed",
  "timestamp": "2025-11-06T10:30:00Z",
  "deployment": {
    "name": "nginx",
    "namespace": "production",
    "currentImage": "nginx:1.25.0",
    "newImage": "nginx:1.26.0",
    "container": "nginx"
  },
  "policy": "minor",
  "requiresApproval": true,
  "updateRequestName": "nginx-update-1-26-0"
}
```

**Event Types:**
- `update_request_created`
- `update_approved`
- `update_rejected`
- `update_completed`
- `update_failed`
- `rollback_triggered`
- `rollback_completed`
- `rollback_failed`

### HMAC Signature Verification

When `WEBHOOK_SECRET` is configured, Headwind sends an HMAC SHA256 signature in the `X-Headwind-Signature` header.

**Format:** `sha256=<hex>`

**Verification (Python):**
```python
import hmac
import hashlib

def verify_signature(secret, payload, signature):
    expected = hmac.new(
        secret.encode(),
        payload.encode(),
        hashlib.sha256
    ).hexdigest()
    return f"sha256={expected}" == signature

# Usage
signature = request.headers.get('X-Headwind-Signature')
payload = request.get_data(as_text=True)
is_valid = verify_signature('your-secret-key', payload, signature)
```

**Verification (Node.js):**
```javascript
const crypto = require('crypto');

function verifySignature(secret, payload, signature) {
    const hmac = crypto.createHmac('sha256', secret);
    hmac.update(payload);
    const expected = `sha256=${hmac.digest('hex')}`;
    return crypto.timingSafeEqual(
        Buffer.from(signature),
        Buffer.from(expected)
    );
}

// Usage
const signature = req.headers['x-headwind-signature'];
const isValid = verifySignature('your-secret-key', req.body, signature);
```

**Verification (Go):**
```go
import (
    "crypto/hmac"
    "crypto/sha256"
    "encoding/hex"
    "fmt"
)

func verifySignature(secret, payload, signature string) bool {
    mac := hmac.New(sha256.New, []byte(secret))
    mac.Write([]byte(payload))
    expected := fmt.Sprintf("sha256=%s", hex.EncodeToString(mac.Sum(nil)))
    return hmac.Equal([]byte(signature), []byte(expected))
}
```

## Multiple Notification Channels

Enable multiple channels simultaneously:

```yaml
env:
# Slack for team notifications
- name: SLACK_ENABLED
  value: "true"
- name: SLACK_WEBHOOK_URL
  valueFrom:
    secretKeyRef:
      name: slack-webhook
      key: url

# Teams for management notifications
- name: TEAMS_ENABLED
  value: "true"
- name: TEAMS_WEBHOOK_URL
  valueFrom:
    secretKeyRef:
      name: teams-webhook
      key: url

# Generic webhook for PagerDuty integration
- name: WEBHOOK_ENABLED
  value: "true"
- name: WEBHOOK_URL
  value: "https://events.pagerduty.com/v2/enqueue"
```

## Configuration Examples

### Production Deployment

Conservative notifications for production:

```yaml
env:
# Slack for team
- name: SLACK_ENABLED
  value: "true"
- name: SLACK_WEBHOOK_URL
  valueFrom:
    secretKeyRef:
      name: slack-webhook
      key: url
- name: SLACK_CHANNEL
  value: "#production-alerts"
- name: SLACK_USERNAME
  value: "Headwind Production"

# PagerDuty for incidents
- name: WEBHOOK_ENABLED
  value: "true"
- name: WEBHOOK_URL
  valueFrom:
    secretKeyRef:
      name: pagerduty-webhook
      key: url
- name: WEBHOOK_SECRET
  valueFrom:
    secretKeyRef:
      name: pagerduty-webhook
      key: secret
```

### Development/Staging

Less noisy notifications:

```yaml
env:
# Slack only for staging
- name: SLACK_ENABLED
  value: "true"
- name: SLACK_WEBHOOK_URL
  valueFrom:
    secretKeyRef:
      name: slack-webhook
      key: url
- name: SLACK_CHANNEL
  value: "#staging-updates"
- name: SLACK_USERNAME
  value: "Headwind Staging"
```

## Monitoring Notifications

### Metrics

Monitor notification delivery with Prometheus:

```promql
# Total notifications sent successfully
headwind_notifications_sent_total

# Total notification failures
headwind_notifications_failed_total

# Notifications sent to Slack
headwind_notifications_slack_sent_total

# Notifications sent to Teams
headwind_notifications_teams_sent_total

# Notifications sent via webhook
headwind_notifications_webhook_sent_total
```

### Alert on Failures

Create Prometheus alerts for notification failures:

```yaml
groups:
- name: headwind_notifications
  rules:
  - alert: HeadwindNotificationFailures
    expr: rate(headwind_notifications_failed_total[5m]) > 0
    for: 5m
    annotations:
      summary: "Headwind notification failures detected"
      description: "Headwind has failed to send {{ $value }} notifications in the last 5 minutes"
```

## Troubleshooting

### Notifications Not Received

Check Headwind logs:

```bash
kubectl logs -n headwind-system deployment/headwind | grep -i notification
```

### Verify Configuration

```bash
# Check environment variables
kubectl get deployment headwind -n headwind-system -o jsonpath='{.spec.template.spec.containers[0].env}' | jq

# Check secrets
kubectl get secret slack-webhook -n headwind-system -o jsonpath='{.data.url}' | base64 -d
```

### Test Webhooks Manually

**Slack:**
```bash
curl -X POST https://hooks.slack.com/services/YOUR/WEBHOOK/URL \
  -H "Content-Type: application/json" \
  -d '{"text":"Test from Headwind"}'
```

**Teams:**
```bash
curl -X POST https://outlook.office.com/webhook/YOUR-WEBHOOK-URL \
  -H "Content-Type: application/json" \
  -d '{"text":"Test from Headwind"}'
```

**Generic Webhook:**
```bash
curl -X POST https://your-webhook-endpoint.com/notifications \
  -H "Content-Type: application/json" \
  -H "X-Headwind-Signature: sha256=test" \
  -d '{"event":"test","timestamp":"2025-11-06T10:00:00Z"}'
```

### Common Issues

**Slack webhook returns 404:**
- Webhook URL is invalid or expired
- Recreate the webhook in Slack

**Teams webhook returns 400:**
- Payload format is invalid
- Check Teams connector is still configured

**Generic webhook timeouts:**
- Increase `WEBHOOK_TIMEOUT`
- Check endpoint is accessible from cluster

**HMAC signature mismatch:**
- Verify secret matches on both sides
- Ensure payload is not modified in transit
- Check encoding (UTF-8)

## Integration Examples

### PagerDuty

```yaml
env:
- name: WEBHOOK_ENABLED
  value: "true"
- name: WEBHOOK_URL
  value: "https://events.pagerduty.com/v2/enqueue"
- name: WEBHOOK_SECRET
  valueFrom:
    secretKeyRef:
      name: pagerduty-integration
      key: routing-key
```

### Opsgenie

```yaml
env:
- name: WEBHOOK_ENABLED
  value: "true"
- name: WEBHOOK_URL
  value: "https://api.opsgenie.com/v2/alerts"
# Add API key in custom headers if needed
```

### Custom HTTP Endpoint

```yaml
env:
- name: WEBHOOK_ENABLED
  value: "true"
- name: WEBHOOK_URL
  value: "https://my-app.example.com/webhooks/headwind"
- name: WEBHOOK_SECRET
  value: "my-secure-secret"
- name: WEBHOOK_TIMEOUT
  value: "30"
- name: WEBHOOK_MAX_RETRIES
  value: "5"
```

## Best Practices

### 1. Use Secrets for Webhook URLs

Never hardcode webhook URLs in manifests:

```yaml
# Good
- name: SLACK_WEBHOOK_URL
  valueFrom:
    secretKeyRef:
      name: slack-webhook
      key: url

# Bad
- name: SLACK_WEBHOOK_URL
  value: "https://hooks.slack.com/services/..."
```

### 2. Enable HMAC Signatures

For generic webhooks, always use signature verification:

```yaml
- name: WEBHOOK_SECRET
  valueFrom:
    secretKeyRef:
      name: webhook-secret
      key: hmac-key
```

### 3. Separate Channels by Environment

Use different channels for different environments:

```yaml
# Production
SLACK_CHANNEL: "#production-alerts"

# Staging
SLACK_CHANNEL: "#staging-updates"

# Development
SLACK_CHANNEL: "#dev-notifications"
```

### 4. Monitor Notification Metrics

Set up alerts for notification failures:

```promql
rate(headwind_notifications_failed_total[5m]) > 0
```

### 5. Test Webhooks Before Deployment

Always test webhook URLs before deploying:

```bash
# Test Slack
curl -X POST $SLACK_WEBHOOK_URL -d '{"text":"Test"}'

# Test Teams
curl -X POST $TEAMS_WEBHOOK_URL -d '{"text":"Test"}'
```

## Next Steps

- [Rollback Configuration](./rollback.md) - Configure automatic rollback
- [Approval Workflow](./approval-workflow.md) - Set up approval process
- [Metrics Reference](../api/metrics.md) - Monitor notification metrics
