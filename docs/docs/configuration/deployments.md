---
sidebar_position: 2
---

# Configuring Deployments

Deployments are the most common workload type in Kubernetes and are fully supported by Headwind.

## Basic Configuration

Add Headwind annotations to your Deployment metadata:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx-example
  namespace: production
  annotations:
    headwind.sh/policy: "minor"
    headwind.sh/require-approval: "true"
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nginx
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
      - name: nginx
        image: nginx:1.25.0
        ports:
        - containerPort: 80
```

## Update Workflow

When a new image version is pushed to your registry:

1. **Detection**: Headwind detects the new version via webhook or polling
2. **Policy Check**: Validates the new version against your policy (`minor`)
3. **Interval Check**: Ensures minimum update interval has elapsed
4. **UpdateRequest**: Creates an UpdateRequest CRD (if approval required)
5. **Approval**: Waits for approval via API
6. **Application**: Updates the Deployment's container image
7. **Notification**: Sends notifications (if configured)
8. **History**: Records update in annotation history

## Multi-Container Deployments

For deployments with multiple containers, Headwind tracks each container independently:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-app
  annotations:
    headwind.sh/policy: "minor"
    # Only track specific containers
    headwind.sh/images: "web, api"
spec:
  template:
    spec:
      containers:
      - name: web
        image: myorg/web:1.5.0
      - name: api
        image: myorg/api:2.3.0
      - name: sidecar  # Not tracked by Headwind
        image: envoyproxy/envoy:1.28.0
```

## Production Example

A production-ready configuration with all safety features:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: production-api
  namespace: production
  annotations:
    # Only patch versions (security fixes)
    headwind.sh/policy: "patch"

    # Require manual approval
    headwind.sh/require-approval: "true"

    # Wait 1 hour between updates minimum
    headwind.sh/min-update-interval: "3600"

    # Enable automatic rollback on failures
    headwind.sh/auto-rollback: "true"

    # Monitor for 10 minutes after update
    headwind.sh/rollback-timeout: "600"

    # Rollback after 2 failed health checks
    headwind.sh/health-check-retries: "2"
spec:
  replicas: 5
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
      maxSurge: 2
  template:
    spec:
      containers:
      - name: api
        image: myorg/api:1.5.0
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
```

## Staging Example

A staging environment with automatic updates:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: staging-api
  namespace: staging
  annotations:
    # Allow minor version updates
    headwind.sh/policy: "minor"

    # No approval required
    headwind.sh/require-approval: "false"

    # Update frequently (5 minutes)
    headwind.sh/min-update-interval: "300"

    # Enable auto-rollback
    headwind.sh/auto-rollback: "true"
spec:
  replicas: 2
  template:
    spec:
      containers:
      - name: api
        image: myorg/api:1.5.0
```

## Private Registry Support

Headwind automatically uses your existing imagePullSecrets:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: private-app
  annotations:
    headwind.sh/policy: "minor"
spec:
  template:
    spec:
      # Headwind reads these credentials automatically
      imagePullSecrets:
      - name: docker-registry-secret
      containers:
      - name: app
        image: myregistry.com/myapp:1.0.0
```

Create the secret:

```bash
kubectl create secret docker-registry docker-registry-secret \
  --docker-server=myregistry.com \
  --docker-username=myuser \
  --docker-password=mypassword \
  --docker-email=myemail@example.com
```

## Viewing Update History

Check the update history in annotations:

```bash
kubectl get deployment my-app -o jsonpath='{.metadata.annotations.headwind\.sh/update-history}' | jq
```

Example output:

```json
[
  {
    "container": "nginx",
    "image": "nginx:1.26.0",
    "timestamp": "2025-11-06T10:30:00Z",
    "updateRequestName": "nginx-update-1-26-0",
    "approvedBy": "admin@example.com"
  },
  {
    "container": "nginx",
    "image": "nginx:1.25.0",
    "timestamp": "2025-11-05T14:20:00Z",
    "updateRequestName": "nginx-update-1-25-0",
    "approvedBy": "webhook"
  }
]
```

## Event Sources

Control how Headwind detects updates for this Deployment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
  annotations:
    headwind.sh/policy: "minor"
    # Use webhooks (default, fastest)
    headwind.sh/event-source: "webhook"

    # Or use polling (for registries without webhook support)
    # headwind.sh/event-source: "polling"
    # headwind.sh/polling-interval: "600"  # Poll every 10 minutes

    # Or use both (redundant detection)
    # headwind.sh/event-source: "both"
```

See [Event Sources](./event-sources.md) for detailed configuration options.

## Next Steps

- [Configure Update Policies](../update-policies.md)
- [Configure Event Sources](./event-sources.md) - Webhooks vs polling
- [Set up Approval Workflow](./approval-workflow.md)
- [Configure Automatic Rollback](./rollback.md)
- [Working with UpdateRequests](../guides/update-requests.md)
