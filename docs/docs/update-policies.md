---
sidebar_position: 3
---

# Update Policies

Headwind uses semantic versioning policies to determine when updates should be applied. This allows you to have fine-grained control over which updates are automatically applied to your workloads.

## Policy Types

### `none` (Default)

Never update automatically. This is the default policy if no annotation is specified.

```yaml
metadata:
  annotations:
    headwind.sh/policy: "none"
```

**Use case**: Manual control only, or when you don't want Headwind to manage updates.

### `patch`

Only update patch versions (1.2.3 → 1.2.4). Minor and major versions are ignored.

```yaml
metadata:
  annotations:
    headwind.sh/policy: "patch"
```

**Examples**:
- ✅ `nginx:1.25.0` → `nginx:1.25.1` (allowed)
- ❌ `nginx:1.25.0` → `nginx:1.26.0` (blocked - minor update)
- ❌ `nginx:1.25.0` → `nginx:2.0.0` (blocked - major update)

**Use case**: Production environments where you only want bug fixes and security patches.

### `minor`

Update patch and minor versions (1.2.3 → 1.3.0). Major versions are ignored.

```yaml
metadata:
  annotations:
    headwind.sh/policy: "minor"
```

**Examples**:
- ✅ `postgres:14.5` → `postgres:14.6` (allowed - patch)
- ✅ `postgres:14.5` → `postgres:14.10` (allowed - minor)
- ❌ `postgres:14.5` → `postgres:15.0` (blocked - major)

**Use case**: Staging/production where you want new features but avoid breaking changes.

### `major`

Update to any new version, including major versions (1.2.3 → 2.0.0).

```yaml
metadata:
  annotations:
    headwind.sh/policy: "major"
```

**Examples**:
- ✅ `redis:6.2.0` → `redis:6.2.1` (allowed)
- ✅ `redis:6.2.0` → `redis:6.3.0` (allowed)
- ✅ `redis:6.2.0` → `redis:7.0.0` (allowed)

**Use case**: Development environments or when you trust the upstream project's versioning.

### `all`

Accept any new version, regardless of semantic versioning.

```yaml
metadata:
  annotations:
    headwind.sh/policy: "all"
```

**Examples**:
- ✅ Any tag that's newer than the current one

**Use case**: Development, testing, or when semantic versioning doesn't apply.

### `glob`

Match a glob pattern. Useful for specific version patterns or naming conventions.

```yaml
metadata:
  annotations:
    headwind.sh/policy: "glob"
    headwind.sh/pattern: "v1.*-stable"
```

**Examples**:
- Pattern: `v1.*-stable`
  - ✅ `myapp:v1.5-stable` (matches)
  - ✅ `myapp:v1.10-stable` (matches)
  - ❌ `myapp:v2.0-stable` (doesn't match)
  - ❌ `myapp:v1.5-beta` (doesn't match)

**Use case**: Custom tagging schemes, stable/beta channels, or specific version ranges.

### `force`

Always update to the latest available version, even if it's older (force update).

```yaml
metadata:
  annotations:
    headwind.sh/policy: "force"
```

**Use case**: Forcing downgrades, testing, or emergency rollouts.

## Version Prefix Handling

Headwind automatically handles common version prefixes:

- `v1.2.3` → treats as `1.2.3`
- `1.2.3` → treats as `1.2.3`

Both formats work correctly with all semantic versioning policies.

## Prerelease and Metadata

Headwind follows semantic versioning 2.0.0 specification:

- **Prerelease**: `1.0.0-alpha`, `1.0.0-beta.1`, `1.0.0-rc.2`
- **Build metadata**: `1.0.0+build.123`, `1.0.0+sha.abc123`

**Behavior**:
- Prereleases are considered less than the normal version
- `1.0.0-alpha` < `1.0.0-beta` < `1.0.0`
- Build metadata is ignored in version comparison

## Real-World Examples

### Production Deployment

Only security patches and bug fixes:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: production-api
  annotations:
    headwind.sh/policy: "patch"
    headwind.sh/require-approval: "true"
    headwind.sh/min-update-interval: "3600"  # 1 hour minimum
spec:
  template:
    spec:
      containers:
      - name: api
        image: myapp:1.5.0
```

### Staging Environment

Allow minor updates automatically:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: staging-api
  annotations:
    headwind.sh/policy: "minor"
    headwind.sh/require-approval: "false"
    headwind.sh/min-update-interval: "300"  # 5 minutes
spec:
  template:
    spec:
      containers:
      - name: api
        image: myapp:1.5.0
```

### Development Environment

Accept all updates:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: dev-api
  annotations:
    headwind.sh/policy: "all"
    headwind.sh/require-approval: "false"
spec:
  template:
    spec:
      containers:
      - name: api
        image: myapp:latest
```

### Stable Channel Only

Use glob pattern for specific tags:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: customer-facing
  annotations:
    headwind.sh/policy: "glob"
    headwind.sh/pattern: "*-stable"
    headwind.sh/require-approval: "true"
spec:
  template:
    spec:
      containers:
      - name: app
        image: myapp:v2.1-stable
```

## Policy Priority

When multiple containers exist in a pod, each container can have different policies. Headwind tracks each container independently.

To target specific images, use the `headwind.sh/images` annotation:

```yaml
metadata:
  annotations:
    headwind.sh/policy: "minor"
    headwind.sh/images: "nginx, redis"  # Only track these images
```

## Next Steps

- [Configure Deployments](./configuration/deployments.md)
- [Set up Approval Workflow](./configuration/approval-workflow.md)
- [Working with UpdateRequests](./guides/update-requests.md)
