---
sidebar_position: 7
---

# Approval Workflow

Configure and manage the approval workflow for deployment updates.

## Coming Soon

Full approval workflow documentation is being written. For now, see the [API Reference](/docs/api/) for approval endpoints.

## Quick Start

Enable approval workflow:

```yaml
metadata:
  annotations:
    headwind.sh/require-approval: "true"
```

Approve via API:

```bash
curl -X POST http://headwind-api:8081/api/v1/updates/{namespace}/{name}/approve \
  -H "Content-Type: application/json" \
  -d '{"approver":"admin@example.com"}'
```
