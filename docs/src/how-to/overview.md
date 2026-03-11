# How-To Guides

This chapter contains goal-oriented guides for operating pgtuskmaster with `pgtm` as the primary interface. The guides focus on practical execution and use raw HTTP only in low-level reference material.

## Cluster Lifecycle

- [Bootstrap a New Cluster from Zero State](bootstrap-cluster.md) - Establish a new PostgreSQL HA cluster from a single initiating node
- [Add a Cluster Node](add-cluster-node.md) - Safely add a new node to an existing cluster and verify replica behavior
- [Remove a Cluster Node](remove-cluster-node.md) - Decommission a node conservatively by moving leadership and stopping it externally

## Health and Monitoring

- [Check Cluster Health](check-cluster-health.md) - Inspect runtime health using `pgtm` against reachable node APIs
- [Monitor via CLI Signals](monitor-via-metrics.md) - Track leader changes, trust degradation, and retained debug history with `pgtm status -v` and `pgtm debug verbose`
- [Debug Cluster Issues](debug-cluster-issues.md) - Investigate incidents with `pgtm status -v` first, then save JSON or consult protocol reference only when needed

## Failure Handling

- [Handle Complex Failures](handle-complex-failures.md) - Diagnose quorum loss, mixed partitions, and other compound HA incidents before manual intervention
- [Handle Primary Failure](handle-primary-failure.md) - Detect, assess, and respond to PostgreSQL primary node failures automatically
- [Handle a Network Partition](handle-network-partition.md) - Detect, monitor, and recover from network partitions using trust and leader consensus

## TLS and Security

- [Configure TLS](configure-tls.md) - Enable TLS for HTTP API and PostgreSQL server surfaces while keeping `pgtm` truthful
- [Configure TLS Security](configure-tls-security.md) - Harden deployments with role tokens and optional client certificate verification

## Operations

- [Perform a Planned Switchover](perform-switchover.md) - Transfer primary leadership to another cluster member without stopping the cluster
- [Run The Test Suite](run-tests.md) - Execute validation gates including fast compile checks, default tests, and long HA scenarios
