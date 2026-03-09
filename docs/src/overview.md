# Overview

pgtuskmaster is a PostgreSQL high-availability manager. This documentation uses the Diataxis framework, organizing content into four complementary types.

## Tutorials

Tutorials are learning experiences that teach you pgtuskmaster through hands-on practice. They guide you step-by-step toward concrete goals.

- [First HA Cluster](tutorial/first-ha-cluster.md) - Build your initial high-availability cluster from scratch
- [Single-Node Setup](tutorial/single-node-setup.md) - Deploy a minimal single-node configuration
- [Observing a Failover Event](tutorial/observing-failover.md) - Watch automatic failover behavior in action
- [Debug API Usage](tutorial/debug-api-usage.md) - Learn to use the debug API for development

[Tutorial Overview](tutorial/overview.md)

## How-To Guides

How-to guides solve specific real-world problems. Use them when you know what you want to accomplish and need step-by-step directions.

- [Check Cluster Health](how-to/check-cluster-health.md) - Verify your cluster's operational status
- [Bootstrap a New Cluster from Zero State](how-to/bootstrap-cluster.md) - Initialize a fresh deployment
- [Add a Cluster Node](how-to/add-cluster-node.md) - Expand an existing cluster
- [Remove a Cluster Node](how-to/remove-cluster-node.md) - Safely decommission nodes
- [Perform a Planned Switchover](how-to/perform-switchover.md) - Execute controlled primary failover
- [Handle Primary Failure](how-to/handle-primary-failure.md) - Respond to primary node outages
- [Handle a Network Partition](how-to/handle-network-partition.md) - Manage split-brain scenarios
- [Configure TLS](how-to/configure-tls.md) - Set up encrypted connections
- [Configure TLS Security](how-to/configure-tls-security.md) - Harden TLS configurations
- [Monitor via API and CLI Signals](how-to/monitor-via-metrics.md) - Track cluster metrics
- [Debug Cluster Issues](how-to/debug-cluster-issues.md) - Troubleshoot common problems
- [Run The Test Suite](how-to/run-tests.md) - Execute pgtuskmaster's tests

[How-To Overview](how-to/overview.md)

## Explanation

Explanation illuminates concepts, context, and design decisions. Read these to understand why pgtuskmaster works the way it does.

- [Introduction](explanation/introduction.md) - Core concepts and philosophy
- [Architecture](explanation/architecture.md) - System design and component relationships
- [Failure Modes and Recovery Behavior](explanation/failure-modes.md) - How pgtuskmaster handles outages
- [HA Decision Engine](explanation/ha-decision-engine.md) - Logic behind failover decisions

[Explanation Overview](explanation/overview.md)

## Reference

Reference provides comprehensive technical descriptions of pgtuskmaster's machinery. Consult these for exact details.

- [HTTP API](reference/http-api.md) - REST API endpoints and payloads
- [HA Decisions](reference/ha-decisions.md) - Failover decision parameters
- [Debug API](reference/debug-api.md) - Development and diagnostic endpoints
- [DCS State Model](reference/dcs-state-model.md) - Distributed consensus state representations
- [pgtuskmaster CLI](reference/pgtuskmaster-cli.md) - Command-line interface options
- [pgtm CLI](reference/pgtm-cli.md) - Cluster control utility
- [Runtime Configuration](reference/runtime-configuration.md) - Configuration parameters and defaults

[Reference Overview](reference/overview.md)
