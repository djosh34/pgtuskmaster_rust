# How-To Guides

This chapter contains goal-oriented guides that help you perform specific operational tasks with PGTuskMaster. Each guide provides a logical sequence of steps to navigate real-world problems and achieve concrete results.

## Cluster Lifecycle

- [Bootstrap a New Cluster from Zero State](bootstrap-cluster.md) - Establish a new PostgreSQL HA cluster from a single initiating node
- [Add a Cluster Node](add-cluster-node.md) - Safely add a new node to an existing cluster and verify replica behavior
- [Remove a Cluster Node](remove-cluster-node.md) - Decommission a node conservatively by moving leadership and stopping externally

## Health and Monitoring

- [Check Cluster Health](check-cluster-health.md) - Inspect runtime health using `pgtm` CLI against node APIs
- [Monitor via API and CLI Signals](monitor-via-metrics.md) - Track leader changes, trust degradation, and recovery activity using JSON observation surfaces
- [Debug Cluster Issues](debug-cluster-issues.md) - Investigate incidents using `/ha/state` and `/debug/verbose` endpoints

## Failure Handling

- [Handle Primary Failure](handle-primary-failure.md) - Detect, assess, and respond to PostgreSQL primary node failures automatically
- [Handle a Network Partition](handle-network-partition.md) - Detect, monitor, and recover from network partitions using trust and leader consensus

## TLS and Security

- [Configure TLS](configure-tls.md) - Enable TLS for HTTP API and PostgreSQL server surfaces
- [Configure TLS Security](configure-tls-security.md) - Harden deployments with bearer token authorization and optional client certificate verification

## Operations

- [Perform a Planned Switchover](perform-switchover.md) - Transfer primary leadership to another cluster member without stopping the cluster
- [Run The Test Suite](run-tests.md) - Execute validation gates including fast compile checks, default tests, and long HA scenarios

These guides assume you are already competent with PostgreSQL, etcd, and basic HA concepts. They focus on practical execution rather than learning or theoretical explanation.
