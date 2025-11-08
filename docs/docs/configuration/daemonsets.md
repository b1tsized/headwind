---
sidebar_position: 4
---

# Configuring DaemonSets

DaemonSets are fully supported by Headwind with the same annotation-based configuration as Deployments and StatefulSets. DaemonSets ensure that all (or some) nodes run a copy of a pod, making them ideal for node-level services.

## Why DaemonSets?

Use DaemonSets for applications that need to run on every node:
- **Node monitoring**: Prometheus Node Exporter, Datadog agents, New Relic
- **Log collection**: Fluentd, Fluent Bit, Logstash
- **Network plugins**: Calico, Weave, Cilium
- **Storage daemons**: Ceph, GlusterFS
- **Security agents**: Falco, Aqua Security

## Supported Annotations

DaemonSets support the exact same annotations as Deployments and StatefulSets:

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
kind: DaemonSet
metadata:
  name: node-exporter
  namespace: monitoring
  annotations:
    # Allow minor version updates
    headwind.sh/policy: "minor"

    # Auto-update without approval (monitoring agent)
    headwind.sh/require-approval: "false"

    # Update every 12 hours max
    headwind.sh/min-update-interval: "43200"
spec:
  selector:
    matchLabels:
      app: node-exporter
  template:
    metadata:
      labels:
        app: node-exporter
    spec:
      hostNetwork: true
      hostPID: true
      containers:
      - name: node-exporter
        image: prom/node-exporter:v1.5.0
        args:
        - --path.procfs=/host/proc
        - --path.sysfs=/host/sys
        - --path.rootfs=/host/root
        ports:
        - containerPort: 9100
          name: metrics
        volumeMounts:
        - name: proc
          mountPath: /host/proc
          readOnly: true
        - name: sys
          mountPath: /host/sys
          readOnly: true
        - name: root
          mountPath: /host/root
          readOnly: true
      volumes:
      - name: proc
        hostPath:
          path: /proc
      - name: sys
        hostPath:
          path: /sys
      - name: root
        hostPath:
          path: /
```

## Update Workflow

When a new image version is detected:

1. **Detection**: Headwind detects via webhook or polling
2. **Policy Check**: Validates version against policy
3. **Interval Check**: Ensures minimum interval has elapsed
4. **UpdateRequest**: Creates UpdateRequest CRD (if approval required)
5. **Approval**: Waits for approval via API (if required)
6. **Application**: Updates DaemonSet spec
7. **Rolling Update**: Kubernetes updates pods node-by-node
8. **Notification**: Sends notifications
9. **History**: Records update in annotations

:::info
DaemonSet updates follow Kubernetes' rolling update strategy: pods are updated node-by-node based on the configured `maxUnavailable` setting. This ensures continuous coverage across all nodes.
:::

## Log Collection Example

Fluent Bit for cluster-wide log collection:

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: fluent-bit
  namespace: logging
  annotations:
    # Only patch versions (bug fixes)
    headwind.sh/policy: "patch"

    # Require approval for production logs
    headwind.sh/require-approval: "true"

    # Wait 24 hours between updates
    headwind.sh/min-update-interval: "86400"

    # Enable auto-rollback
    headwind.sh/auto-rollback: "true"
    headwind.sh/rollback-timeout: "600"
spec:
  selector:
    matchLabels:
      app: fluent-bit
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1  # Update one node at a time
  template:
    metadata:
      labels:
        app: fluent-bit
    spec:
      serviceAccountName: fluent-bit
      containers:
      - name: fluent-bit
        image: fluent/fluent-bit:2.0.0
        ports:
        - containerPort: 2020
          name: metrics
        readinessProbe:
          httpGet:
            path: /api/v1/health
            port: 2020
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /api/v1/health
            port: 2020
          periodSeconds: 30
        volumeMounts:
        - name: varlog
          mountPath: /var/log
          readOnly: true
        - name: varlibdockercontainers
          mountPath: /var/lib/docker/containers
          readOnly: true
        - name: fluent-bit-config
          mountPath: /fluent-bit/etc/
      volumes:
      - name: varlog
        hostPath:
          path: /var/log
      - name: varlibdockercontainers
        hostPath:
          path: /var/lib/docker/containers
      - name: fluent-bit-config
        configMap:
          name: fluent-bit-config
```

## Network Plugin Example

Calico CNI plugin with conservative updates:

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: calico-node
  namespace: kube-system
  annotations:
    # Only patch versions (critical fixes only)
    headwind.sh/policy: "patch"

    # Always require approval (networking is critical)
    headwind.sh/require-approval: "true"

    # Wait 7 days between updates
    headwind.sh/min-update-interval: "604800"

    # Auto-rollback on failure
    headwind.sh/auto-rollback: "true"
    headwind.sh/rollback-timeout: "300"
    headwind.sh/health-check-retries: "2"
spec:
  selector:
    matchLabels:
      k8s-app: calico-node
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1  # Very conservative
  template:
    metadata:
      labels:
        k8s-app: calico-node
    spec:
      hostNetwork: true
      serviceAccountName: calico-node
      containers:
      - name: calico-node
        image: calico/node:v3.25.0
        env:
        - name: DATASTORE_TYPE
          value: kubernetes
        - name: WAIT_FOR_DATASTORE
          value: "true"
        - name: NODENAME
          valueFrom:
            fieldRef:
              fieldPath: spec.nodeName
        - name: CALICO_NETWORKING_BACKEND
          value: bird
        - name: CLUSTER_TYPE
          value: k8s,bgp
        securityContext:
          privileged: true
        readinessProbe:
          exec:
            command:
            - /bin/calico-node
            - -felix-ready
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /liveness
            port: 9099
          periodSeconds: 10
          initialDelaySeconds: 10
        volumeMounts:
        - name: lib-modules
          mountPath: /lib/modules
          readOnly: true
        - name: var-run-calico
          mountPath: /var/run/calico
        - name: var-lib-calico
          mountPath: /var/lib/calico
      volumes:
      - name: lib-modules
        hostPath:
          path: /lib/modules
      - name: var-run-calico
        hostPath:
          path: /var/run/calico
      - name: var-lib-calico
        hostPath:
          path: /var/lib/calico
```

## Monitoring Agent Example

Datadog agent with automatic updates:

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: datadog-agent
  namespace: monitoring
  annotations:
    # Allow minor versions (new features)
    headwind.sh/policy: "minor"

    # Auto-update (monitoring can self-heal)
    headwind.sh/require-approval: "false"

    # Update every 6 hours max
    headwind.sh/min-update-interval: "21600"

    # Enable auto-rollback
    headwind.sh/auto-rollback: "true"
spec:
  selector:
    matchLabels:
      app: datadog-agent
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 10%  # Update 10% of nodes at once
  template:
    metadata:
      labels:
        app: datadog-agent
    spec:
      serviceAccountName: datadog-agent
      containers:
      - name: agent
        image: datadog/agent:7.42.0
        env:
        - name: DD_API_KEY
          valueFrom:
            secretKeyRef:
              name: datadog-secret
              key: api-key
        - name: DD_KUBERNETES_KUBELET_HOST
          valueFrom:
            fieldRef:
              fieldPath: status.hostIP
        - name: DD_LOGS_ENABLED
          value: "true"
        - name: DD_LOGS_CONFIG_CONTAINER_COLLECT_ALL
          value: "true"
        - name: DD_PROCESS_AGENT_ENABLED
          value: "true"
        readinessProbe:
          httpGet:
            path: /ready
            port: 5555
          periodSeconds: 10
        livenessProbe:
          httpGet:
            path: /live
            port: 5555
          periodSeconds: 10
        volumeMounts:
        - name: dockersocket
          mountPath: /var/run/docker.sock
          readOnly: true
        - name: procdir
          mountPath: /host/proc
          readOnly: true
        - name: cgroups
          mountPath: /host/sys/fs/cgroup
          readOnly: true
      volumes:
      - name: dockersocket
        hostPath:
          path: /var/run/docker.sock
      - name: procdir
        hostPath:
          path: /proc
      - name: cgroups
        hostPath:
          path: /sys/fs/cgroup
```

## Security Agent Example

Falco security monitoring with glob pattern:

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: falco
  namespace: security
  annotations:
    # Only stable releases
    headwind.sh/policy: "glob"
    headwind.sh/pattern: "*-stable"

    # Require approval
    headwind.sh/require-approval: "true"

    # Wait 3 days between updates
    headwind.sh/min-update-interval: "259200"
spec:
  selector:
    matchLabels:
      app: falco
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1
  template:
    metadata:
      labels:
        app: falco
    spec:
      hostNetwork: true
      hostPID: true
      serviceAccountName: falco
      containers:
      - name: falco
        image: falcosecurity/falco:0.34.0-stable
        args:
        - /usr/bin/falco
        - --cri
        - /run/containerd/containerd.sock
        - -K
        - /var/run/secrets/kubernetes.io/serviceaccount/token
        - -k
        - https://kubernetes.default
        - -pk
        securityContext:
          privileged: true
        volumeMounts:
        - name: dev
          mountPath: /host/dev
        - name: proc
          mountPath: /host/proc
          readOnly: true
        - name: boot
          mountPath: /host/boot
          readOnly: true
        - name: lib-modules
          mountPath: /host/lib/modules
          readOnly: true
        - name: usr
          mountPath: /host/usr
          readOnly: true
        - name: etc
          mountPath: /host/etc
          readOnly: true
      volumes:
      - name: dev
        hostPath:
          path: /dev
      - name: proc
        hostPath:
          path: /proc
      - name: boot
        hostPath:
          path: /boot
      - name: lib-modules
        hostPath:
          path: /lib/modules
      - name: usr
        hostPath:
          path: /usr
      - name: etc
        hostPath:
          path: /etc
```

## Node-Specific DaemonSets

Run DaemonSets only on specific nodes using nodeSelector:

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: gpu-driver
  namespace: kube-system
  annotations:
    headwind.sh/policy: "patch"
    headwind.sh/require-approval: "true"
spec:
  selector:
    matchLabels:
      app: gpu-driver
  template:
    metadata:
      labels:
        app: gpu-driver
    spec:
      nodeSelector:
        gpu: "true"  # Only run on GPU nodes
      containers:
      - name: nvidia-driver
        image: nvidia/driver:515.48.07
        securityContext:
          privileged: true
```

Or use affinity for more complex node selection:

```yaml
spec:
  template:
    spec:
      affinity:
        nodeAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
            nodeSelectorTerms:
            - matchExpressions:
              - key: node.kubernetes.io/instance-type
                operator: In
                values:
                - c5.large
                - c5.xlarge
```

## Update Strategy Considerations

### Conservative Rolling Updates

For critical infrastructure (networking, security):

```yaml
spec:
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 1  # One node at a time
```

### Faster Rolling Updates

For monitoring and logging (can tolerate brief gaps):

```yaml
spec:
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      maxUnavailable: 25%  # Quarter of nodes simultaneously
```

### OnDelete Strategy

Manual control over pod updates:

```yaml
spec:
  updateStrategy:
    type: OnDelete  # Pods updated only when manually deleted
```

:::warning
With `OnDelete` strategy, Headwind will update the DaemonSet spec but pods won't be recreated until you manually delete them. This gives maximum control but requires manual intervention.
:::

## Private Registry Support

DaemonSets work with private registries using imagePullSecrets:

```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: private-agent
  annotations:
    headwind.sh/policy: "minor"
spec:
  template:
    spec:
      imagePullSecrets:
      - name: registry-credentials
      containers:
      - name: agent
        image: myregistry.com/agent:1.0.0
```

## Monitoring Updates

### View Update History

```bash
# Get update history from annotations
kubectl get daemonset node-exporter -n monitoring \
  -o jsonpath='{.metadata.annotations.headwind\.sh/update-history}' | jq

# Example output
[
  {
    "container": "node-exporter",
    "image": "prom/node-exporter:v1.5.0",
    "timestamp": "2025-11-06T10:30:00Z",
    "updateRequestName": "node-exporter-update-v1-5-0",
    "approvedBy": "webhook"
  },
  {
    "container": "node-exporter",
    "image": "prom/node-exporter:v1.4.0",
    "timestamp": "2025-10-20T14:15:00Z",
    "updateRequestName": "node-exporter-update-v1-4-0",
    "approvedBy": "admin@example.com"
  }
]
```

### Check UpdateRequests

```bash
# List pending updates for DaemonSets
kubectl get updaterequests -A -o json | \
  jq '.items[] | select(.spec.targetRef.kind == "DaemonSet")'
```

### Monitor Pod Updates

Watch DaemonSet rollout progress:

```bash
# Check rollout status
kubectl rollout status daemonset/fluent-bit -n logging

# Watch pod updates across nodes
kubectl get pods -n logging -l app=fluent-bit -o wide --watch

# Check how many nodes are running updated pods
kubectl get daemonset fluent-bit -n logging
```

### Metrics

Monitor DaemonSet updates with Prometheus:

```promql
# DaemonSets being watched
headwind_daemonsets_watched

# Updates applied to DaemonSets
headwind_updates_applied_total{kind="DaemonSet"}

# Pending updates for DaemonSets
headwind_updates_pending{kind="DaemonSet"}

# Rollback operations for DaemonSets
headwind_rollbacks_total{kind="DaemonSet"}
```

## Best Practices

### 1. Conservative Policies for Critical Infrastructure

For networking, security, and storage:

```yaml
annotations:
  headwind.sh/policy: "patch"  # Only security fixes
  headwind.sh/require-approval: "true"  # Always require approval
  headwind.sh/min-update-interval: "604800"  # Wait 1 week
```

### 2. More Permissive for Observability

For monitoring and logging:

```yaml
annotations:
  headwind.sh/policy: "minor"  # Allow feature updates
  headwind.sh/require-approval: "false"  # Auto-update
  headwind.sh/min-update-interval: "21600"  # Wait 6 hours
```

### 3. Enable Auto-Rollback

Always enable for production DaemonSets:

```yaml
annotations:
  headwind.sh/auto-rollback: "true"
  headwind.sh/rollback-timeout: "600"  # 10 minutes
  headwind.sh/health-check-retries: "2"
```

### 4. Configure Proper Update Strategy

Match `maxUnavailable` to your tolerance:

```yaml
spec:
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      # Critical: 1 node at a time
      maxUnavailable: 1

      # OR Non-critical: 25% of nodes
      maxUnavailable: 25%
```

### 5. Use Health Checks

Essential for automatic rollback:

```yaml
readinessProbe:
  httpGet:
    path: /ready
    port: 9090
  periodSeconds: 10

livenessProbe:
  httpGet:
    path: /health
    port: 9090
  periodSeconds: 30
```

### 6. Test Node Coverage

After updates, verify all nodes are covered:

```bash
# Check number of desired vs current pods
kubectl get daemonset -n monitoring

# Verify pods on each node
kubectl get pods -n monitoring -o wide | grep node-exporter

# Count nodes
kubectl get nodes --no-headers | wc -l
```

### 7. Environment-Specific Policies

Different policies per environment:

```yaml
# Production - very conservative
headwind.sh/policy: "patch"
headwind.sh/require-approval: "true"
headwind.sh/min-update-interval: "604800"  # 1 week

# Development - permissive
headwind.sh/policy: "all"
headwind.sh/require-approval: "false"
headwind.sh/min-update-interval: "3600"  # 1 hour
```

## Troubleshooting

### DaemonSet Not Updating

Check if pods are running on all nodes:

```bash
# Get DaemonSet status
kubectl get daemonset -n monitoring

# Check for pod scheduling issues
kubectl get pods -n monitoring -o wide

# Look for node taints or constraints
kubectl describe nodes | grep -A 5 Taints
```

### Pod Stuck on Node

DaemonSet updates wait for pod to be Ready:

```bash
# Check specific pod
kubectl describe pod fluent-bit-xyz -n logging

# Check pod logs
kubectl logs fluent-bit-xyz -n logging

# Check node conditions
kubectl describe node node-1 | grep Conditions -A 10
```

### Update Too Slow

If `maxUnavailable: 1` is too slow:

```yaml
spec:
  updateStrategy:
    rollingUpdate:
      maxUnavailable: 3  # Update 3 nodes at once
```

Or use percentage:

```yaml
maxUnavailable: 10%  # 10% of nodes
```

### Node Not Getting Updated

Check node selectors and taints:

```bash
# Check DaemonSet node selector
kubectl get daemonset fluent-bit -o jsonpath='{.spec.template.spec.nodeSelector}'

# Check node labels
kubectl get nodes --show-labels

# Check node taints
kubectl describe nodes | grep Taints
```

## Next Steps

- [Update Policies](../update-policies.md) - Understand semantic versioning policies
- [Approval Workflow](./approval-workflow.md) - Configure approval process
- [Rollback Configuration](./rollback.md) - Set up automatic rollback
- [Notifications](./notifications.md) - Configure Slack/Teams notifications
