---
sidebar_position: 2
---

# Installation

This guide will walk you through installing Headwind in your Kubernetes cluster.

## Prerequisites

- Kubernetes cluster (1.25+)
- kubectl configured
- Docker (for building the image)

## Installation Methods

### Method 1: Using Pre-built Manifests (Recommended)

```bash
# Create namespace and apply CRDs
kubectl apply -f https://raw.githubusercontent.com/headwind.sh/headwind/main/deploy/k8s/namespace.yaml
kubectl apply -f https://raw.githubusercontent.com/headwind.sh/headwind/main/deploy/k8s/crds/updaterequest.yaml

# Optional: Apply HelmRepository CRD if you want Helm chart auto-discovery
# (Skip if you already have Flux CD installed)
kubectl apply -f https://raw.githubusercontent.com/headwind.sh/headwind/main/deploy/k8s/crds/helmrepository.yaml

# Apply RBAC and deployment
kubectl apply -f https://raw.githubusercontent.com/headwind.sh/headwind/main/deploy/k8s/rbac.yaml
kubectl apply -f https://raw.githubusercontent.com/headwind.sh/headwind/main/deploy/k8s/deployment.yaml
kubectl apply -f https://raw.githubusercontent.com/headwind.sh/headwind/main/deploy/k8s/service.yaml
```

### Method 2: Building from Source

```bash
# Clone the repository
git clone https://github.com/headwind.sh/headwind.git
cd headwind

# Build the Docker image
docker build -t headwind:latest .

# Load into your cluster (for kind/minikube)
kind load docker-image headwind:latest  # or minikube image load headwind:latest

# Apply all manifests
kubectl apply -f deploy/k8s/namespace.yaml
kubectl apply -f deploy/k8s/crds/updaterequest.yaml
kubectl apply -f deploy/k8s/crds/helmrepository.yaml  # Optional
kubectl apply -f deploy/k8s/rbac.yaml
kubectl apply -f deploy/k8s/deployment.yaml
kubectl apply -f deploy/k8s/service.yaml
```

## Verify Installation

Check that Headwind is running:

```bash
# Check pod status
kubectl get pods -n headwind-system

# Expected output:
# NAME                        READY   STATUS    RESTARTS   AGE
# headwind-7d9f8c9b6d-xxxxx   1/1     Running   0          1m

# Check logs
kubectl logs -n headwind-system deployment/headwind

# Check services
kubectl get svc -n headwind-system

# Expected output:
# NAME               TYPE        CLUSTER-IP      EXTERNAL-IP   PORT(S)             AGE
# headwind-webhook   ClusterIP   10.96.xxx.xxx   <none>        8080/TCP            1m
# headwind-api       ClusterIP   10.96.xxx.xxx   <none>        8081/TCP            1m
# headwind-metrics   ClusterIP   10.96.xxx.xxx   <none>        9090/TCP            1m
```

## Post-Installation Steps

### 1. Configure Registry Webhooks (Recommended)

For event-driven updates, configure your container registry to send webhooks to Headwind.

**Expose Headwind webhook service:**

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: headwind-webhook
  namespace: headwind-system
spec:
  rules:
  - host: headwind.yourdomain.com
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

**Configure your registry:**

- **Docker Hub**: `https://headwind.yourdomain.com/webhook/dockerhub`
- **Harbor/GitLab/GCR**: `https://headwind.yourdomain.com/webhook/registry`

### 2. Enable Registry Polling (Alternative)

If webhooks aren't available, enable polling:

```yaml
# Edit deployment
kubectl edit deployment headwind -n headwind-system

# Add environment variables:
env:
- name: HEADWIND_POLLING_ENABLED
  value: "true"
- name: HEADWIND_POLLING_INTERVAL
  value: "300"  # Poll every 5 minutes
```

### 3. Configure Notifications (Optional)

See the [Notifications Guide](./configuration/notifications.md) for setting up Slack, Teams, or webhook notifications.

### 4. Set Up Prometheus Scraping (Optional)

```yaml
# prometheus-config.yaml
- job_name: 'headwind'
  kubernetes_sd_configs:
  - role: pod
    namespaces:
      names:
      - headwind-system
  relabel_configs:
  - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
    action: keep
    regex: true
  - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_port]
    action: replace
    target_label: __address__
    regex: (.+):(.+)
    replacement: $1:9090
```

## Next Steps

- [Configure your first Deployment](./configuration/deployments.md)
- [Set up update policies](./update-policies.md)
- [Configure the approval workflow](./configuration/approval-workflow.md)
