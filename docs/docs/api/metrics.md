---
sidebar_position: 2
---

# Metrics Reference

Headwind exposes Prometheus metrics on port 9090 at `/metrics` for comprehensive monitoring and alerting.

## Accessing Metrics

```bash
# Port forward metrics endpoint
kubectl port-forward -n headwind-system svc/headwind-metrics 9090:9090

# View metrics
curl http://localhost:9090/metrics

# Or open in browser
open http://localhost:9090/metrics
```

## Webhook Metrics

Track webhook event processing:

### `headwind_webhook_events_total`

**Type**: Counter

**Description**: Total webhook events received from container registries

**Labels**:
- `registry` - Registry type (dockerhub, harbor, gitlab, etc.)

**Example**:
```promql
# Rate of webhook events per minute
rate(headwind_webhook_events_total[5m]) * 60

# Total events by registry
sum by (registry) (headwind_webhook_events_total)
```

### `headwind_webhook_events_processed`

**Type**: Counter

**Description**: Successfully processed webhook events

**Example**:
```promql
# Processing success rate
rate(headwind_webhook_events_processed[5m]) / rate(headwind_webhook_events_total[5m])
```

## Polling Metrics

Monitor registry polling operations:

### `headwind_polling_cycles_total`

**Type**: Counter

**Description**: Total polling cycles completed

**Example**:
```promql
# Polling frequency
rate(headwind_polling_cycles_total[5m])
```

### `headwind_polling_errors_total`

**Type**: Counter

**Description**: Polling errors encountered

**Example**:
```promql
# Error rate
rate(headwind_polling_errors_total[5m])
```

### `headwind_polling_images_checked_total`

**Type**: Counter

**Description**: Container images checked during polling

**Example**:
```promql
# Images checked per polling cycle
rate(headwind_polling_images_checked_total[5m]) / rate(headwind_polling_cycles_total[5m])
```

### `headwind_polling_new_tags_found_total`

**Type**: Counter

**Description**: New image tags discovered via polling

**Example**:
```promql
# Tag discovery rate
rate(headwind_polling_new_tags_found_total[1h])
```

### `headwind_polling_helm_charts_checked_total`

**Type**: Counter

**Description**: Helm charts checked during polling

**Example**:
```promql
# Helm charts checked per cycle
rate(headwind_polling_helm_charts_checked_total[5m])
```

### `headwind_polling_helm_new_versions_found_total`

**Type**: Counter

**Description**: New Helm chart versions discovered via polling

**Example**:
```promql
# Helm version discovery rate
rate(headwind_polling_helm_new_versions_found_total[1h])
```

### `headwind_polling_resources_filtered_total`

**Type**: Counter

**Description**: Resources filtered out from polling due to `event-source` annotation

**Details**: Incremented when resources have `event-source: webhook` or `event-source: none` set. These resources are skipped during polling cycles to reduce unnecessary registry API calls.

**Example**:
```promql
# Resources filtered from polling
headwind_polling_resources_filtered_total

# Filter rate per polling cycle
rate(headwind_polling_resources_filtered_total[5m]) / rate(headwind_polling_cycles_total[5m])

# Percentage of resources using webhooks only
headwind_polling_resources_filtered_total /
  (headwind_polling_resources_filtered_total + headwind_polling_images_checked_total)
```

**Use Cases**:
- Monitor adoption of webhook vs polling event sources
- Track resource distribution across event source types
- Optimize polling efficiency

## Update Metrics

Track update requests and their lifecycle:

### `headwind_updates_pending`

**Type**: Gauge

**Description**: Number of UpdateRequests currently awaiting approval

**Example**:
```promql
# Current pending updates
headwind_updates_pending

# Alert on too many pending updates
headwind_updates_pending > 20
```

### `headwind_updates_approved_total`

**Type**: Counter

**Description**: Total approved updates

**Example**:
```promql
# Approval rate
rate(headwind_updates_approved_total[1h])
```

### `headwind_updates_rejected_total`

**Type**: Counter

**Description**: Total rejected updates

**Example**:
```promql
# Rejection rate
rate(headwind_updates_rejected_total[1h])

# Approval vs rejection ratio
headwind_updates_approved_total / (headwind_updates_approved_total + headwind_updates_rejected_total)
```

### `headwind_updates_applied_total`

**Type**: Counter

**Description**: Successfully applied updates

**Labels**:
- `kind` - Workload kind (Deployment, StatefulSet, DaemonSet, HelmRelease)

**Example**:
```promql
# Update success rate
rate(headwind_updates_applied_total[1h])

# Updates by workload type
sum by (kind) (headwind_updates_applied_total)
```

### `headwind_updates_failed_total`

**Type**: Counter

**Description**: Failed update attempts

**Example**:
```promql
# Failure rate
rate(headwind_updates_failed_total[5m])

# Update success rate
rate(headwind_updates_applied_total[5m]) / (rate(headwind_updates_applied_total[5m]) + rate(headwind_updates_failed_total[5m]))
```

### `headwind_updates_skipped_interval_total`

**Type**: Counter

**Description**: Updates skipped due to minimum update interval not elapsed

**Example**:
```promql
# Rate of skipped updates
rate(headwind_updates_skipped_interval_total[1h])
```

## Controller Metrics

Monitor Kubernetes controllers:

### `headwind_reconcile_duration_seconds`

**Type**: Histogram

**Description**: Time spent in reconciliation loops

**Buckets**: 0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0

**Example**:
```promql
# 95th percentile reconciliation time
histogram_quantile(0.95, rate(headwind_reconcile_duration_seconds_bucket[5m]))

# Average reconciliation duration
rate(headwind_reconcile_duration_seconds_sum[5m]) / rate(headwind_reconcile_duration_seconds_count[5m])
```

### `headwind_reconcile_errors_total`

**Type**: Counter

**Description**: Reconciliation errors

**Example**:
```promql
# Error rate
rate(headwind_reconcile_errors_total[5m])
```

## Workload Watching Metrics

Track resources being monitored:

### `headwind_deployments_watched`

**Type**: Gauge

**Description**: Number of Deployments being monitored by Headwind

**Example**:
```promql
headwind_deployments_watched
```

### `headwind_statefulsets_watched`

**Type**: Gauge

**Description**: Number of StatefulSets being monitored

**Example**:
```promql
headwind_statefulsets_watched
```

### `headwind_daemonsets_watched`

**Type**: Gauge

**Description**: Number of DaemonSets being monitored

**Example**:
```promql
headwind_daemonsets_watched
```

### `headwind_helm_releases_watched`

**Type**: Gauge

**Description**: Number of HelmReleases being monitored

**Example**:
```promql
headwind_helm_releases_watched

# Total workloads watched
headwind_deployments_watched + headwind_statefulsets_watched + headwind_daemonsets_watched + headwind_helm_releases_watched
```

## Helm Metrics

Track Helm chart version discovery and updates:

### `headwind_helm_chart_versions_checked_total`

**Type**: Counter

**Description**: Helm chart version checks performed

**Example**:
```promql
rate(headwind_helm_chart_versions_checked_total[5m])
```

### `headwind_helm_updates_found_total`

**Type**: Counter

**Description**: Helm chart updates discovered

**Example**:
```promql
rate(headwind_helm_updates_found_total[1h])
```

### `headwind_helm_updates_approved_total`

**Type**: Counter

**Description**: Helm chart updates approved by policy

**Example**:
```promql
# Approval rate
headwind_helm_updates_approved_total / headwind_helm_updates_found_total
```

### `headwind_helm_updates_rejected_total`

**Type**: Counter

**Description**: Helm chart updates rejected by policy

**Example**:
```promql
# Rejection rate
headwind_helm_updates_rejected_total / headwind_helm_updates_found_total
```

### `headwind_helm_updates_applied_total`

**Type**: Counter

**Description**: Helm chart updates successfully applied

**Example**:
```promql
rate(headwind_helm_updates_applied_total[1h])
```

### `headwind_helm_repository_queries_total`

**Type**: Counter

**Description**: Helm repository queries performed

**Example**:
```promql
rate(headwind_helm_repository_queries_total[5m])
```

### `headwind_helm_repository_errors_total`

**Type**: Counter

**Description**: Helm repository query errors

**Example**:
```promql
# Error rate
rate(headwind_helm_repository_errors_total[5m]) / rate(headwind_helm_repository_queries_total[5m])
```

### `headwind_helm_repository_query_duration_seconds`

**Type**: Histogram

**Description**: Helm repository query duration

**Example**:
```promql
# 95th percentile query time
histogram_quantile(0.95, rate(headwind_helm_repository_query_duration_seconds_bucket[5m]))
```

## Rollback Metrics

Monitor rollback operations:

### `headwind_rollbacks_total`

**Type**: Counter

**Description**: Total rollback operations (manual + automatic)

**Example**:
```promql
rate(headwind_rollbacks_total[1h])
```

### `headwind_rollbacks_manual_total`

**Type**: Counter

**Description**: Manual rollback operations

**Example**:
```promql
rate(headwind_rollbacks_manual_total[1h])
```

### `headwind_rollbacks_automatic_total`

**Type**: Counter

**Description**: Automatic rollback operations triggered by health failures

**Example**:
```promql
rate(headwind_rollbacks_automatic_total[1h])

# Automatic rollback ratio
headwind_rollbacks_automatic_total / headwind_rollbacks_total
```

### `headwind_rollbacks_failed_total`

**Type**: Counter

**Description**: Failed rollback operations

**Example**:
```promql
# Rollback success rate
(headwind_rollbacks_total - headwind_rollbacks_failed_total) / headwind_rollbacks_total
```

### `headwind_deployment_health_checks_total`

**Type**: Counter

**Description**: Deployment health checks performed after updates

**Example**:
```promql
rate(headwind_deployment_health_checks_total[5m])
```

### `headwind_deployment_health_failures_total`

**Type**: Counter

**Description**: Deployment health check failures detected

**Example**:
```promql
# Health failure rate
rate(headwind_deployment_health_failures_total[5m]) / rate(headwind_deployment_health_checks_total[5m])
```

## Notification Metrics

Track notification delivery:

### `headwind_notifications_sent_total`

**Type**: Counter

**Description**: Total notifications sent successfully

**Example**:
```promql
rate(headwind_notifications_sent_total[5m])
```

### `headwind_notifications_failed_total`

**Type**: Counter

**Description**: Total notification failures

**Example**:
```promql
# Failure rate
rate(headwind_notifications_failed_total[5m]) / rate(headwind_notifications_sent_total[5m])
```

### `headwind_notifications_slack_sent_total`

**Type**: Counter

**Description**: Notifications sent to Slack

**Example**:
```promql
rate(headwind_notifications_slack_sent_total[5m])
```

### `headwind_notifications_teams_sent_total`

**Type**: Counter

**Description**: Notifications sent to Microsoft Teams

**Example**:
```promql
rate(headwind_notifications_teams_sent_total[5m])
```

### `headwind_notifications_webhook_sent_total`

**Type**: Counter

**Description**: Notifications sent via generic webhooks

**Example**:
```promql
rate(headwind_notifications_webhook_sent_total[5m])
```

## Prometheus Alerts

Example alert rules for Headwind:

```yaml
groups:
- name: headwind
  rules:
  # Update alerts
  - alert: HeadwindStaleUpdateRequests
    expr: headwind_updates_pending > 10
    for: 1h
    annotations:
      summary: "Many pending UpdateRequests"
      description: "{{ $value }} UpdateRequests pending for over 1 hour"

  - alert: HeadwindHighUpdateFailureRate
    expr: rate(headwind_updates_failed_total[5m]) > 0.1
    for: 5m
    annotations:
      summary: "High update failure rate"
      description: "Update failures detected"

  # Rollback alerts
  - alert: HeadwindAutomaticRollback
    expr: increase(headwind_rollbacks_automatic_total[5m]) > 0
    annotations:
      summary: "Automatic rollback triggered"
      description: "Headwind triggered an automatic rollback"

  - alert: HeadwindFrequentRollbacks
    expr: rate(headwind_rollbacks_total[1h]) > 3
    for: 5m
    annotations:
      summary: "Frequent rollbacks detected"
      description: "{{ $value }} rollbacks in the last hour"

  # Helm alerts
  - alert: HeadwindHelmRepositoryErrors
    expr: rate(headwind_helm_repository_errors_total[5m]) > 0
    for: 5m
    annotations:
      summary: "Helm repository query errors"
      description: "Errors querying Helm repositories"

  # Notification alerts
  - alert: HeadwindNotificationFailures
    expr: rate(headwind_notifications_failed_total[5m]) > 0
    for: 5m
    annotations:
      summary: "Notification failures detected"
      description: "Headwind notifications are failing"

  # Reconciliation alerts
  - alert: HeadwindSlowReconciliation
    expr: histogram_quantile(0.95, rate(headwind_reconcile_duration_seconds_bucket[5m])) > 5
    for: 10m
    annotations:
      summary: "Slow reconciliation loops"
      description: "95th percentile reconciliation time > 5s"

  - alert: HeadwindReconciliationErrors
    expr: rate(headwind_reconcile_errors_total[5m]) > 0.1
    for: 5m
    annotations:
      summary: "Reconciliation errors"
      description: "Controller reconciliation errors detected"
```

## Grafana Dashboard

Example PromQL queries for a Grafana dashboard:

### Overview Panel

```promql
# Pending updates
headwind_updates_pending

# Watched resources
sum(headwind_deployments_watched + headwind_statefulsets_watched + headwind_daemonsets_watched + headwind_helm_releases_watched)

# Update success rate (last hour)
rate(headwind_updates_applied_total[1h]) / (rate(headwind_updates_applied_total[1h]) + rate(headwind_updates_failed_total[1h]))
```

### Update Activity Panel

```promql
# Updates approved (rate)
rate(headwind_updates_approved_total[5m])

# Updates applied by type
sum by (kind) (rate(headwind_updates_applied_total[5m]))

# Updates rejected (rate)
rate(headwind_updates_rejected_total[5m])
```

### Rollback Panel

```promql
# Total rollbacks
rate(headwind_rollbacks_total[1h])

# Automatic vs Manual
rate(headwind_rollbacks_automatic_total[1h])
rate(headwind_rollbacks_manual_total[1h])

# Health check failure rate
rate(headwind_deployment_health_failures_total[5m]) / rate(headwind_deployment_health_checks_total[5m])
```

### Performance Panel

```promql
# Reconciliation latency (p95)
histogram_quantile(0.95, rate(headwind_reconcile_duration_seconds_bucket[5m]))

# Helm repository query latency (p95)
histogram_quantile(0.95, rate(headwind_helm_repository_query_duration_seconds_bucket[5m]))
```

## Scraping Configuration

Configure Prometheus to scrape Headwind metrics:

```yaml
scrape_configs:
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
  - source_labels: [__meta_kubernetes_pod_name]
    action: replace
    target_label: pod
  - source_labels: [__meta_kubernetes_namespace]
    action: replace
    target_label: namespace
```

## Next Steps

- [Notifications](../configuration/notifications.md) - Configure notifications
- [Rollback Configuration](../configuration/rollback.md) - Set up rollback
- [API Reference](./index.md) - REST API documentation
