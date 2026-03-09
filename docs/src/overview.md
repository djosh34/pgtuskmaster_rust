# Overview

pgtuskmaster is a PostgreSQL high-availability manager. This documentation uses the Diataxis framework, organizing content into four complementary types.

The operator-facing path is `pgtm`. Use the daemon binary reference only when you are configuring or running `pgtuskmaster` itself.

## Tutorials

Tutorials teach the system through hands-on practice.

- [First HA Cluster](tutorial/first-ha-cluster.md) - Start the local three-node cluster and inspect it through `pgtm`
- [Single-Node Setup](tutorial/single-node-setup.md) - Run the smallest shipped deployment with a truthful operator config
- [Observing a Failover Event](tutorial/observing-failover.md) - Watch automatic failover behavior with cluster-wide status and retained debug history
- [Debug API Usage](tutorial/debug-api-usage.md) - Learn `pgtm debug verbose`, JSON capture, and incremental polling with `since`

[Tutorial Overview](tutorial/overview.md)

## How-To Guides

How-to guides solve specific operational problems.

- [Check Cluster Health](how-to/check-cluster-health.md) - Verify cluster health from one operator command
- [Bootstrap a New Cluster from Zero State](how-to/bootstrap-cluster.md) - Initialize a fresh deployment
- [Add a Cluster Node](how-to/add-cluster-node.md) - Expand an existing cluster safely
- [Remove a Cluster Node](how-to/remove-cluster-node.md) - Decommission a node conservatively
- [Perform a Planned Switchover](how-to/perform-switchover.md) - Execute controlled primary failover
- [Handle Primary Failure](how-to/handle-primary-failure.md) - Respond to primary node outages
- [Handle a Network Partition](how-to/handle-network-partition.md) - Monitor and recover from split connectivity
- [Configure TLS](how-to/configure-tls.md) - Enable TLS while keeping `pgtm` truthful
- [Configure TLS Security](how-to/configure-tls-security.md) - Harden TLS and role-token posture
- [Monitor via CLI Signals](how-to/monitor-via-metrics.md) - Track trust, leadership, and retained debug history with `pgtm`
- [Debug Cluster Issues](how-to/debug-cluster-issues.md) - Investigate incidents with `pgtm` first
- [Run The Test Suite](how-to/run-tests.md) - Execute pgtuskmaster's validation gates

[How-To Overview](how-to/overview.md)

## Explanation

Explanation illuminates concepts, context, and design decisions.

- [Introduction](explanation/introduction.md) - Core concepts and philosophy
- [Architecture](explanation/architecture.md) - System design and component relationships
- [Failure Modes and Recovery Behavior](explanation/failure-modes.md) - How pgtuskmaster handles outages
- [HA Decision Engine](explanation/ha-decision-engine.md) - Logic behind failover decisions

[Explanation Overview](explanation/overview.md)

## Reference

Reference states exact technical details.

- [HTTP API](reference/http-api.md) - Raw protocol reference for endpoints and payloads
- [HA Decisions](reference/ha-decisions.md) - Failover decision parameters
- [Debug API](reference/debug-api.md) - Underlying debug endpoint contracts behind `pgtm debug verbose`
- [DCS State Model](reference/dcs-state-model.md) - Distributed consensus state representations
- [pgtm CLI](reference/pgtm-cli.md) - Operator CLI reference
- [pgtuskmaster CLI](reference/pgtuskmaster-cli.md) - Daemon binary reference
- [Runtime Configuration](reference/runtime-configuration.md) - Configuration parameters and defaults

[Reference Overview](reference/overview.md)
