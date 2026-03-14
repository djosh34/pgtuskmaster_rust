# Overview

pgtuskmaster is a PostgreSQL high-availability manager. This documentation follows Diataxis and keeps the operator path centered on one node read surface and one control noun.

## Tutorials

- [First HA Cluster](tutorial/first-ha-cluster.md) - start the local three-node cluster and inspect it with `pgtm`
- [Observing a Failover Event](tutorial/observing-failover.md) - watch automatic failover behavior
- [Validating Cluster Behavior](tutorial/validating-cluster-behavior.md) - validate expected HA outcomes

[Tutorial Overview](tutorial/overview.md)

## How-To Guides

- [Check Cluster Health](how-to/check-cluster-health.md) - verify cluster health from one operator command
- [Bootstrap a New Cluster from Zero State](how-to/bootstrap-cluster.md) - initialize a fresh deployment
- [Add a Cluster Node](how-to/add-cluster-node.md) - expand an existing cluster safely
- [Remove a Cluster Node](how-to/remove-cluster-node.md) - decommission a node conservatively
- [Perform a Planned Switchover](how-to/perform-switchover.md) - execute controlled primary failover
- [Handle Primary Failure](how-to/handle-primary-failure.md) - respond to primary node outages
- [Handle a Network Partition](how-to/handle-network-partition.md) - monitor and recover from split connectivity
- [Configure TLS](how-to/configure-tls.md) - enable TLS while keeping `pgtm` truthful
- [Configure TLS Security](how-to/configure-tls-security.md) - harden TLS and token posture
- [Monitor via CLI Signals](how-to/monitor-via-metrics.md) - track trust, leadership, and process state
- [Debug Cluster Issues](how-to/debug-cluster-issues.md) - investigate incidents with `pgtm` and `/state`
- [Run The Test Suite](how-to/run-tests.md) - execute the validation gates

[How-To Overview](how-to/overview.md)

## Explanation

- [Introduction](explanation/introduction.md) - core concepts and philosophy
- [Architecture](explanation/architecture.md) - system design and component relationships
- [Failure Modes and Recovery Behavior](explanation/failure-modes.md) - outage handling
- [HA Decision Engine](explanation/ha-decision-engine.md) - how the runtime decides authority and role

[Explanation Overview](explanation/overview.md)

## Reference

- [HTTP API](reference/http-api.md) - raw protocol reference for `/state` and `/switchover`
- [HA Decisions](reference/ha-decisions.md) - meaning of the `ha` section inside `/state`
- [DCS State Model](reference/dcs-state-model.md) - distributed coordination state structures
- [pgtm CLI](reference/pgtm-cli.md) - operator CLI reference
- [pgtuskmaster CLI](reference/pgtuskmaster-cli.md) - daemon binary reference
- [Runtime Configuration](reference/runtime-configuration.md) - configuration parameters and defaults

[Reference Overview](reference/overview.md)
