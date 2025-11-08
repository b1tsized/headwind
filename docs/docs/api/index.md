---
sidebar_position: 1
---

# API Reference

Headwind exposes HTTP APIs for managing update approvals, rollbacks, and viewing metrics.

## API Endpoints

### Approval API (Port 8081)

The Approval API manages UpdateRequest CRDs and executes approved updates.

**Base URL**: `http://headwind-api:8081/api/v1`

#### List All Updates

```http
GET /updates
```

Returns all UpdateRequest CRDs across all namespaces.

**Response**:
```json
[
  {
    "metadata": {
      "name": "nginx-update-1-26-0",
      "namespace": "default"
    },
    "spec": {
      "targetRef": {
        "kind": "Deployment",
        "name": "nginx-example",
        "namespace": "default"
      },
      "containerName": "nginx",
      "currentImage": "nginx:1.25.0",
      "newImage": "nginx:1.26.0",
      "policy": "minor"
    },
    "status": {
      "phase": "Pending",
      "createdAt": "2025-11-06T01:00:00Z"
    }
  }
]
```

#### Get Specific Update

```http
GET /updates/{namespace}/{name}
```

**Parameters**:
- `namespace` - Namespace of the UpdateRequest
- `name` - Name of the UpdateRequest

**Response**: Single UpdateRequest object

#### Approve Update

```http
POST /updates/{namespace}/{name}/approve
```

Approves and immediately executes the update.

**Request Body**:
```json
{
  "approver": "admin@example.com"
}
```

**Response**:
```json
{
  "message": "Update approved and applied successfully",
  "updateRequest": {
    "metadata": {...},
    "status": {
      "phase": "Completed",
      "approvedBy": "admin@example.com",
      "approvedAt": "2025-11-06T10:00:00Z"
    }
  }
}
```

#### Reject Update

```http
POST /updates/{namespace}/{name}/reject
```

**Request Body**:
```json
{
  "approver": "admin@example.com",
  "reason": "Not ready for production deployment"
}
```

**Response**:
```json
{
  "message": "Update rejected",
  "updateRequest": {
    "metadata": {...},
    "status": {
      "phase": "Rejected",
      "approvedBy": "admin@example.com",
      "rejectedAt": "2025-11-06T10:00:00Z",
      "reason": "Not ready for production deployment"
    }
  }
}
```

### Rollback API (Port 8081)

The Rollback API provides manual rollback capabilities and update history.

**Base URL**: `http://headwind-api:8081/api/v1/rollback`

#### Get Update History

```http
GET /rollback/{namespace}/{deployment}/history
```

Returns the last 10 updates for a deployment.

**Response**:
```json
{
  "deployment": "nginx-example",
  "namespace": "default",
  "history": [
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
}
```

#### Rollback Deployment

```http
POST /rollback/{namespace}/{deployment}/{container}
```

Rolls back the specified container to its previous image.

**Response**:
```json
{
  "message": "Rollback successful",
  "deployment": "nginx-example",
  "namespace": "default",
  "container": "nginx",
  "previousImage": "nginx:1.26.0",
  "rolledBackTo": "nginx:1.25.0"
}
```

### Health Check (Port 8080, 8081, 9090)

All services expose a `/health` endpoint:

```http
GET /health
```

**Response**:
```json
{
  "status": "healthy"
}
```

### Metrics (Port 9090)

Prometheus metrics endpoint:

```http
GET /metrics
```

Returns metrics in Prometheus text format. See [Metrics Reference](./metrics.md) for details.

## Authentication

:::warning
The current version does not include built-in authentication. For production use, you should:
- Use Kubernetes RBAC to restrict access to the service
- Deploy behind an API gateway with authentication
- Use network policies to restrict access
- Consider adding OAuth2/OIDC proxy
:::

## Rate Limiting

No built-in rate limiting is currently implemented. Consider using an API gateway or ingress controller with rate limiting capabilities.

## Error Responses

All APIs return consistent error responses:

```json
{
  "error": "UpdateRequest not found",
  "code": 404
}
```

**Common Error Codes**:
- `400` - Bad Request (invalid parameters)
- `404` - Not Found (resource doesn't exist)
- `500` - Internal Server Error

## Examples

### Using curl

```bash
# Port forward the API service
kubectl port-forward -n headwind-system svc/headwind-api 8081:8081

# List all updates
curl http://localhost:8081/api/v1/updates | jq

# Get specific update
curl http://localhost:8081/api/v1/updates/default/nginx-update-1-26-0 | jq

# Approve update
curl -X POST http://localhost:8081/api/v1/updates/default/nginx-update-1-26-0/approve \
  -H "Content-Type: application/json" \
  -d '{"approver":"admin@example.com"}' | jq

# Reject update
curl -X POST http://localhost:8081/api/v1/updates/default/nginx-update-1-26-0/reject \
  -H "Content-Type: application/json" \
  -d '{"approver":"admin@example.com","reason":"Not ready"}' | jq

# Get rollback history
curl http://localhost:8081/api/v1/rollback/default/nginx-example/history | jq

# Rollback deployment
curl -X POST http://localhost:8081/api/v1/rollback/default/nginx-example/nginx | jq
```

### Using kubectl plugin

Install the kubectl plugin for a better experience:

```bash
# List updates
kubectl headwind list

# Approve update
kubectl headwind approve nginx-update-1-26-0 --approver admin@example.com

# Reject update
kubectl headwind reject nginx-update-1-26-0 "Not ready" --approver admin@example.com

# View history
kubectl headwind history nginx-example -n default

# Rollback
kubectl headwind rollback nginx-example -n default
```

See the [kubectl plugin documentation](../guides/kubectl-plugin.md) for more details.

## Next Steps

- [Metrics Reference](./metrics.md)
- [kubectl Plugin Guide](../guides/kubectl-plugin.md)
- [Approval Workflow Configuration](../configuration/approval-workflow.md)
