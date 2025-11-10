# Helm Chart Repository

The Headwind Helm chart repository is available at: https://headwind.sh/charts/

## Quick Start

```bash
helm repo add headwind https://headwind.sh/charts
helm repo update
helm install headwind headwind/headwind -n headwind-system --create-namespace
```

For more information, see the [Helm Installation Guide](https://headwind.sh/docs/guides/helm-installation).
