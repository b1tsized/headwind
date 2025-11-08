# Headwind Examples

This directory contains example configurations and test files for Headwind.

## Production Examples

These are production-ready example configurations:

- **deployment-with-headwind.yaml** - Example Deployment with Headwind annotations
- **updaterequest.yaml** - Example UpdateRequest CRD
- **notification-configmap.yaml** - Example notification configuration

## Test Manifests

Located in `test-manifests/`, these are for manual testing and development:

- **test-deployment.yaml** - Test Deployment for image updates
- **test-helmrelease.yaml** - Test HelmRelease configuration
- **test-podinfo-helmrelease.yaml** - Podinfo HelmRelease test
- **test-busybox-http.yaml** - Busybox HelmRelease with HTTP repository
- **test-busybox-oci.yaml** - Busybox HelmRelease with OCI repository

## Test Scripts

Located in `scripts/`, these are for testing specific features:

- **test-webhook-server.py** - Mock webhook server for testing webhook processing
- **test-notifications.sh** - Test notification integrations (Slack, Teams, webhooks)

## Usage

### Applying Examples

```bash
# Apply production example
kubectl apply -f deployment-with-headwind.yaml

# Apply test manifests
kubectl apply -f test-manifests/test-deployment.yaml
```

### Running Test Scripts

```bash
# Test webhook processing
cd scripts
python test-webhook-server.py

# Test notifications
./test-notifications.sh
```

## Contributing

When adding new examples:
- Production examples go in the root `examples/` directory
- Test files go in `test-manifests/` or `scripts/`
- Include comments explaining the configuration
- Follow the same annotation patterns as existing examples
