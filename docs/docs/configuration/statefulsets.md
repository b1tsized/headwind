---
sidebar_position: 3
---

# Configuring StatefulSets

StatefulSets are fully supported by Headwind with the same annotation-based configuration as Deployments. StatefulSets are designed for applications that require persistent storage, stable network identities, or ordered deployment/scaling.

## Why StatefulSets?

Use StatefulSets for applications that need:
- **Stable network identities**: Predictable pod names (pod-0, pod-1, etc.)
- **Persistent storage**: Each pod gets its own persistent volume
- **Ordered operations**: Pods are created, scaled, and deleted in order
- **Stateful applications**: Databases, message queues, distributed systems

## Supported Annotations

StatefulSets support the exact same annotations as Deployments:

| Annotation | Type | Default | Description |
|------------|------|---------|-------------|
| `headwind.sh/policy` | string | `none` | Update policy: `none`, `patch`, `minor`, `major`, `all`, `glob`, `force` |
| `headwind.sh/pattern` | string | - | Glob pattern (required for `glob` policy) |
| `headwind.sh/require-approval` | boolean | `true` | Whether updates require manual approval |
| `headwind.sh/min-update-interval` | integer | `300` | Minimum seconds between updates |
| `headwind.sh/images` | string | - | Comma-separated list of images to track |
| `headwind.sh/auto-rollback` | boolean | `false` | Enable automatic rollback on failures |
| `headwind.sh/rollback-timeout` | integer | `300` | Health check monitoring duration (seconds) |
| `headwind.sh/health-check-retries` | integer | `3` | Failed health checks before rollback |

## Basic Configuration

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres
  namespace: production
  annotations:
    # Only patch versions (security fixes)
    headwind.sh/policy: "patch"

    # Require approval
    headwind.sh/require-approval: "true"

    # Wait at least 1 hour between updates
    headwind.sh/min-update-interval: "3600"
spec:
  serviceName: postgres
  replicas: 3
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:14.5
        ports:
        - containerPort: 5432
          name: postgres
        volumeMounts:
        - name: data
          mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
```

## Update Workflow

When a new image version is detected:

1. **Detection**: Headwind detects via webhook or polling
2. **Policy Check**: Validates version against policy
3. **Interval Check**: Ensures minimum interval has elapsed
4. **UpdateRequest**: Creates UpdateRequest CRD (if approval required)
5. **Approval**: Waits for approval via API
6. **Application**: Updates StatefulSet spec
7. **Rolling Update**: Kubernetes updates pods in reverse ordinal order (pod-2, pod-1, pod-0)
8. **Notification**: Sends notifications
9. **History**: Records update in annotations

:::info
StatefulSet updates follow Kubernetes' default rolling update strategy: pods are updated in reverse ordinal order (highest to lowest). This ensures the master/leader (typically pod-0) is updated last.
:::

## Production Database Example

A production PostgreSQL cluster with safety features:

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres-cluster
  namespace: production
  annotations:
    # Only security patches
    headwind.sh/policy: "patch"

    # Require manual approval
    headwind.sh/require-approval: "true"

    # Wait 24 hours between updates
    headwind.sh/min-update-interval: "86400"

    # Enable automatic rollback
    headwind.sh/auto-rollback: "true"

    # Monitor for 15 minutes after update
    headwind.sh/rollback-timeout: "900"

    # Rollback after 2 failed health checks
    headwind.sh/health-check-retries: "2"
spec:
  serviceName: postgres
  replicas: 3
  podManagementPolicy: OrderedReady
  updateStrategy:
    type: RollingUpdate
  selector:
    matchLabels:
      app: postgres
  template:
    metadata:
      labels:
        app: postgres
    spec:
      containers:
      - name: postgres
        image: postgres:14.5
        ports:
        - containerPort: 5432
          name: postgres
        env:
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: postgres-secret
              key: password
        - name: PGDATA
          value: /var/lib/postgresql/data/pgdata
        readinessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - postgres
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 5
        livenessProbe:
          exec:
            command:
            - pg_isready
            - -U
            - postgres
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
        volumeMounts:
        - name: data
          mountPath: /var/lib/postgresql/data
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 100Gi
```

## Redis Cluster Example

A Redis cluster allowing minor version updates:

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: redis-cluster
  namespace: cache
  annotations:
    # Allow minor version updates
    headwind.sh/policy: "minor"

    # Auto-update without approval
    headwind.sh/require-approval: "false"

    # Update every 6 hours max
    headwind.sh/min-update-interval: "21600"

    # Enable auto-rollback
    headwind.sh/auto-rollback: "true"
spec:
  serviceName: redis
  replicas: 6
  selector:
    matchLabels:
      app: redis
  template:
    metadata:
      labels:
        app: redis
    spec:
      containers:
      - name: redis
        image: redis:7.0.0
        command:
        - redis-server
        args:
        - --cluster-enabled
        - "yes"
        - --cluster-config-file
        - /data/nodes.conf
        - --cluster-node-timeout
        - "5000"
        - --appendonly
        - "yes"
        ports:
        - containerPort: 6379
          name: client
        - containerPort: 16379
          name: gossip
        readinessProbe:
          exec:
            command:
            - redis-cli
            - ping
          initialDelaySeconds: 10
          periodSeconds: 5
        livenessProbe:
          exec:
            command:
            - redis-cli
            - ping
          initialDelaySeconds: 30
          periodSeconds: 10
        volumeMounts:
        - name: data
          mountPath: /data
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi
```

## Kafka Cluster Example

Apache Kafka with glob pattern for stable releases:

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: kafka
  namespace: streaming
  annotations:
    # Only stable releases
    headwind.sh/policy: "glob"
    headwind.sh/pattern: "*-stable"

    # Require approval
    headwind.sh/require-approval: "true"

    # Wait 7 days between updates
    headwind.sh/min-update-interval: "604800"
spec:
  serviceName: kafka
  replicas: 3
  selector:
    matchLabels:
      app: kafka
  template:
    metadata:
      labels:
        app: kafka
    spec:
      containers:
      - name: kafka
        image: confluentinc/cp-kafka:7.4.0-stable
        ports:
        - containerPort: 9092
          name: kafka
        env:
        - name: KAFKA_BROKER_ID
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: KAFKA_ZOOKEEPER_CONNECT
          value: zookeeper:2181
        volumeMounts:
        - name: data
          mountPath: /var/lib/kafka/data
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 100Gi
```

## Multi-Container StatefulSet

Tracking multiple container images independently:

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: app-with-sidecar
  annotations:
    headwind.sh/policy: "minor"
    # Track both containers
    headwind.sh/images: "app, metrics-exporter"
spec:
  serviceName: app
  replicas: 3
  selector:
    matchLabels:
      app: myapp
  template:
    spec:
      containers:
      - name: app
        image: myorg/app:1.5.0
      - name: metrics-exporter
        image: myorg/exporter:2.3.0
      - name: log-shipper  # Not tracked by Headwind
        image: fluent/fluent-bit:2.0.0
```

## Update Strategy Considerations

### Ordered Updates (Default)

```yaml
spec:
  podManagementPolicy: OrderedReady  # Default
  updateStrategy:
    type: RollingUpdate
```

- Pods updated in reverse ordinal order (N-1, N-2, ..., 0)
- Next pod waits for previous to be Ready
- Safest for databases and clustered applications

### Parallel Updates

```yaml
spec:
  podManagementPolicy: Parallel
  updateStrategy:
    type: RollingUpdate
```

- All pods updated simultaneously
- Faster but riskier
- Use only for truly stateless workloads in StatefulSet form

## Private Registry Support

StatefulSets work with private registries using imagePullSecrets:

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: private-db
  annotations:
    headwind.sh/policy: "patch"
spec:
  template:
    spec:
      imagePullSecrets:
      - name: registry-credentials
      containers:
      - name: db
        image: myregistry.com/database:1.0.0
```

## Monitoring Updates

### View Update History

```bash
# Get update history from annotations
kubectl get statefulset postgres -o jsonpath='{.metadata.annotations.headwind\.sh/update-history}' | jq

# Example output
[
  {
    "container": "postgres",
    "image": "postgres:14.6",
    "timestamp": "2025-11-06T10:30:00Z",
    "updateRequestName": "postgres-update-14-6",
    "approvedBy": "admin@example.com"
  },
  {
    "container": "postgres",
    "image": "postgres:14.5",
    "timestamp": "2025-10-15T08:20:00Z",
    "updateRequestName": "postgres-update-14-5",
    "approvedBy": "admin@example.com"
  }
]
```

### Check UpdateRequests

```bash
# List pending updates for StatefulSets
kubectl get updaterequests -A -o json | \
  jq '.items[] | select(.spec.targetRef.kind == "StatefulSet")'
```

### Metrics

Monitor StatefulSet updates with Prometheus:

```promql
# StatefulSets being watched
headwind_statefulsets_watched

# Updates applied to StatefulSets
headwind_updates_applied_total{kind="StatefulSet"}

# Pending updates for StatefulSets
headwind_updates_pending{kind="StatefulSet"}
```

## Best Practices

### 1. Conservative Update Policies

For stateful applications, use conservative policies:
- **Production**: `patch` policy only
- **Staging**: `minor` policy
- **Development**: `major` or `all` policy

### 2. Always Require Approval

```yaml
annotations:
  headwind.sh/require-approval: "true"  # Recommended for StatefulSets
```

### 3. Longer Update Intervals

Stateful apps need more time between updates:

```yaml
annotations:
  # Wait at least 1 week
  headwind.sh/min-update-interval: "604800"
```

### 4. Enable Auto-Rollback

```yaml
annotations:
  headwind.sh/auto-rollback: "true"
  headwind.sh/rollback-timeout: "900"  # 15 minutes
  headwind.sh/health-check-retries: "2"
```

### 5. Use Proper Health Checks

Ensure readiness and liveness probes are configured:

```yaml
readinessProbe:
  # Check if pod is ready to serve traffic
  periodSeconds: 5

livenessProbe:
  # Check if pod is alive
  periodSeconds: 10
```

### 6. Test in Staging First

Use different policies per environment:

```yaml
# Production - very conservative
headwind.sh/policy: "patch"
headwind.sh/require-approval: "true"
headwind.sh/min-update-interval: "604800"  # 1 week

# Staging - more permissive
headwind.sh/policy: "minor"
headwind.sh/require-approval: "false"
headwind.sh/min-update-interval: "86400"  # 1 day
```

## Troubleshooting

### Update Not Applied

Check the UpdateRequest status:

```bash
kubectl get updaterequests -A
kubectl describe updaterequest <name> -n <namespace>
```

### Pod Stuck During Update

StatefulSets wait for each pod to be Ready before updating the next:

```bash
# Check pod status
kubectl get pods -l app=postgres

# Check pod events
kubectl describe pod postgres-2

# Check if pod is ready
kubectl get pod postgres-2 -o jsonpath='{.status.conditions[?(@.type=="Ready")].status}'
```

### Rollback Failed

Check Headwind logs:

```bash
kubectl logs -n headwind-system deployment/headwind | grep rollback
```

## Event Sources

Control how Headwind detects updates for this StatefulSet:

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: my-database
  annotations:
    headwind.sh/policy: "patch"
    # Use webhooks (default, fastest)
    headwind.sh/event-source: "webhook"

    # Or use polling with custom interval
    # headwind.sh/event-source: "polling"
    # headwind.sh/polling-interval: "600"
```

See [Event Sources](./event-sources.md) for detailed configuration options.

## Next Steps

- [Update Policies](../update-policies.md) - Understand semantic versioning policies
- [Configure Event Sources](./event-sources.md) - Webhooks vs polling
- [Approval Workflow](./approval-workflow.md) - Configure approval process
- [Rollback Configuration](./rollback.md) - Set up automatic rollback
- [API Reference](../api/) - Approve updates via API
