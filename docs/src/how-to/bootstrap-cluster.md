# Bootstrap a New Cluster from Zero State

This guide shows you how to bootstrap a new PostgreSQL high-availability cluster from zero state using a single initiating node.

## Goal

You will establish a new cluster scope in your distributed coordination service (etcd), elect an initial primary, and prepare the environment for subsequent replica nodes to join.

## Prerequisites

- Running etcd cluster accessible from all intended cluster members
- PostgreSQL 16 binaries installed on each node
- Runtime configuration file (TOML) for the bootstrap node with complete [postgres], [dcs], [ha], and [process] sections
- Network connectivity between nodes on PostgreSQL and API ports

## Configure DCS Init Payload

Provide the DCS initialization payload in your runtime configuration. The schema supports `dcs.init = { payload_json, write_on_bootstrap }`, and the HA startup harness uses that payload to materialize `/{scope}/config` during first-node bootstrap.

Two details matter:

- `payload_json` must carry a complete runtime configuration snapshot
- the stored config sets `dcs.init` to `null`, so the configuration written into DCS does not recursively contain another bootstrap-init stanza

## Start the Bootstrap Node

Launch the first node with your configuration file and wait for HA to move through its startup phases.

The node enters the following state sequence:

1. **Init** - Worker starts with no prior state
2. **WaitingPostgresReachable** - Attempts to start PostgreSQL if not running
3. **WaitingDcsTrusted** - Monitors etcd health and member freshness

## Wait for Primary Election

Monitor the node's API endpoint for phase progression. The node will transition to **Primary** phase when:

- DCS trust evaluates to FreshQuorum
- No other active leader lease exists
- Local PostgreSQL is healthy or can be promoted

The election uses create-if-absent semantics for the `/{scope}/leader` key. The HA observer in the test suite also treats more than one primary as split-brain and fails immediately when it samples that condition.

## Verify DCS Bootstrap Completion

Confirm successful bootstrap by checking etcd contents:

1. `/{scope}/cluster/initialized` exists with non-empty value
2. `/{scope}/cluster/identity` exists and records the authoritative cluster `system_identifier`
3. `/{scope}/config` exists and matches your `payload_json` exactly
4. `/{scope}/leader` contains your bootstrap node's member ID
5. `/{scope}/member/{member_id}` exists for your node

The bootstrap node publishes its member record including PostgreSQL host, port, role, and timeline information.

## Provision Replication Roles

Before starting additional nodes, create the replication roles on the elected primary. The replicator role requires `LOGIN REPLICATION` privileges. The rewinder role requires `LOGIN SUPERUSER` privileges for `pg_rewind` operations.

After the first node becomes primary, provision the replication roles on that primary before you start later nodes. The harness requires:

- a replicator role with `LOGIN REPLICATION`
- a rewinder role with `LOGIN SUPERUSER`

These credentials must match the PostgreSQL role settings in your runtime configuration.

## Deploy Subsequent Nodes

For each additional node:

1. Update `cluster.member_id` in the runtime configuration
2. Configure `[postgres]` with unique `listen_port` and data directory
3. Keep `[dcs]` endpoints and scope identical to bootstrap node
4. Do not include `[dcs.init]` section

When started, nodes automatically:
- Detect the existing primary from DCS member records
- Transition to **Replica** phase
- Follow the active leader using streaming replication
- Publish their own member records for health monitoring

**join behavior**: A node observing a healthy primary member record in DCS will follow that primary without attempting leadership, even if no explicit leader lease exists.

## Troubleshooting

### Bootstrap Node Stays in WaitingPostgresReachable

- Check PostgreSQL logs for startup failures
- Verify `postgres.listen_host` and `listen_port` are not in use
- Confirm `[process.binaries]` paths point to valid PostgreSQL executables

### Bootstrap Node Enters FailSafe

- Verify etcd cluster health: DCS trust becomes `NotTrusted` if etcd is unreachable
- Check member record freshness: ensure system clocks are synchronized
- In multi-member setups, ensure at least two members remain fresh within `lease_ttl_ms`

### Duplicate Primary Detected

If two nodes appear primary at the same time, treat that as a split-brain signal. The HA observer used in tests fails immediately when API or SQL sampling sees more than one primary, and the HA state machine includes fencing paths for foreign-leader detection.

### Subsequent Node Fails to Join

- Confirm replication roles exist on primary with correct passwords
- Verify `rewind_conn_identity` points to a superuser role
- Check DCS member records show primary as healthy
- Review `pg_hba.conf` on primary allows replication connections from new node
