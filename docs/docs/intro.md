---
sidebar_position: 1
slug: /
---

# Introduction

Headwind is a Kubernetes operator written in Rust that automates workload updates based on container image changes. It provides both webhook-driven updates and registry polling, a full approval workflow system, and comprehensive observability.

## Features

- **Dual Update Triggers**: Event-driven webhooks **or** registry polling for maximum flexibility
- **Semver Policy Engine**: Intelligent update decisions based on semantic versioning (patch, minor, major, glob, force, all)
- **Approval Workflow**: Full HTTP API for approval requests with integration possibilities (Slack, webhooks, etc.)
- **Rollback Support**: Manual rollback to previous versions with update history tracking and automatic rollback on failures
- **Notifications**: Slack, Microsoft Teams, and generic webhook notifications for all deployment events
- **Full Observability**: Prometheus metrics, distributed tracing, and structured logging
- **Resource Support**:
  - Kubernetes Deployments ✅
  - Kubernetes StatefulSets ✅
  - Kubernetes DaemonSets ✅
  - Flux HelmReleases ✅
- **Lightweight**: Single binary, no database required
- **Secure**: Runs as non-root, read-only filesystem, minimal permissions

## How It Works

```
┌─────────────────┐
│  Registry       │
│  (Docker Hub,   │
│   Harbor, etc)  │
└────┬────────┬───┘
     │        │
     │Webhook │Polling
     │        │(optional)
     ▼        ▼
┌──────────────────┐
│  Headwind        │
│  ┌────────────┐  │
│  │  Webhook   │  │◄─── Port 8080
│  │  Server    │  │
│  └──────┬─────┘  │
│         │        │
│  ┌──────▼─────┐  │
│  │  Policy    │  │
│  │  Engine    │  │
│  └──────┬─────┘  │
│         │        │
│  ┌──────▼─────┐  │
│  │  Approval  │  │◄─── Port 8081 (API)
│  │  System    │  │
│  └──────┬─────┘  │
│         │        │
│  ┌──────▼─────┐  │
│  │Controller  │  │
│  └──────┬─────┘  │
│         │        │
│  ┌──────▼─────┐  │
│  │  Metrics   │  │◄─── Port 9090
│  └────────────┘  │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│   Kubernetes     │
│   API Server     │
└──────────────────┘
```

## Quick Example

Add annotations to your Deployment:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-app
  annotations:
    # Update policy
    headwind.sh/policy: "minor"
    # Require approval
    headwind.sh/require-approval: "true"
spec:
  # ... rest of deployment spec
```

When a new minor version is pushed to your registry, Headwind will:
1. Detect the new image
2. Check the policy (minor versions allowed)
3. Create an UpdateRequest CRD
4. Wait for approval via API
5. Apply the update when approved
6. Send notifications

## Next Steps

- [Installation Guide](./installation.md) - Get Headwind running in your cluster
- [Configuration](./configuration/index.md) - Learn about all configuration options
- [Update Policies](./update-policies.md) - Understand semantic versioning policies
- [API Reference](./api/index.md) - Explore the approval and rollback APIs
