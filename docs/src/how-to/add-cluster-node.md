# Add a Cluster Node

This guide shows how to add a new node to an existing cluster and verify that it joins safely with `pgtm` as the primary operator interface.

## Goal

Bring up a new node that:

- uses the same cluster identity and DCS scope as the existing cluster
- publishes its own member record
- converges into expected replica behavior when a healthy primary already exists

## Prerequisites

- a running cluster with a known `cluster.name` and `dcs.scope`
- PostgreSQL 16 binaries installed on the new node
- valid runtime-config paths for PostgreSQL data, socket, and logs
- network reachability to the cluster's DCS endpoints
- network reachability to the relevant PostgreSQL endpoints in the cluster
- an operator-facing config for the new node that either sets `[pgtm].api_url` or derives an operator-reachable URL from `api.listen_addr`

## Step 1: Prepare a runtime config for the new node

Use an existing runtime config as your starting point and change the node-specific identity and addresses.

The docker example at `docker/configs/cluster/node-a/runtime.toml` shows the full daemon config shape. If that daemon config binds an unspecified address such as `0.0.0.0:8080`, add a docs- or ops-owned `[pgtm].api_url` for the operator-reachable API URL before you use it with `pgtm`.

Fields that must be correct for the new node:

- `cluster.name`
- `cluster.member_id`
- `postgres.listen_host`
- `postgres.listen_port`
- `dcs.endpoints`
- `dcs.scope`
- `process.binaries.*`
- `api.listen_addr`

## Step 2: Check connectivity before you start it

Before starting the new node, verify:

- it can reach the configured DCS endpoints
- the relevant PostgreSQL listen address and port are reachable in your environment
- the operator config points `pgtm` at the node's reachable API URL

## Step 3: Start the node with the prepared config

Start `pgtuskmaster` using your normal service method for this environment.

The node starts by observing PostgreSQL and DCS state, then converges into one of the normal HA roles such as `follower`, `candidate`, or `idle` depending on the observed world state.

## Step 4: Watch the new node through `pgtm`

Seed the cluster view through the new node:

```bash
pgtm -c config.toml status -v
```

Watch these fields first:

- `ROLE`
- `TRUST`
- `PHASE`
- `DECISION`
- `LEADER`
- `DEBUG`

When a healthy primary already exists, the usual steady-state goal is replica behavior rather than leadership.

## Step 5: Verify that the node is joining the existing topology

For a normal join into a healthy cluster, look for:

- trusted DCS state
- a visible leader
- the new node settling into replica-oriented behavior

In practice that means:

- `TRUST=full_quorum`
- `LEADER` agrees with the rest of the cluster
- `PHASE` stops moving through startup transitions
- `DECISION` stops showing startup or recovery churn

If the table is not enough, inspect the joining node directly:

```bash
pgtm -c config.toml debug verbose
```

## Step 6: Verify replica behavior

Use PostgreSQL-level checks that fit your environment to confirm the new node is following the current primary.

The exact SQL and access path depend on your deployment, but the goal is:

- the new node is not acting as a second primary
- the new node can follow the current leader
- fresh writes on the primary become visible after replication catches up

## Step 7: Compare more than one seed before you declare success

Repeat `pgtm status -v` from at least one surviving node's operator config. You want:

- agreement on the same leader
- no sustained dual-primary evidence
- no node stuck in `fail_safe`
- the new node no longer bouncing through startup transitions

## Troubleshooting

### The node stays in `waiting_postgres_reachable`

Check:

- local PostgreSQL startup
- `process.binaries.*`
- PostgreSQL data, socket, and log paths

### The node stays in `waiting_dcs_trusted` or enters `fail_safe`

Check:

- DCS endpoint reachability
- DCS scope correctness
- whether the node can publish fresh membership

### The node attempts leadership unexpectedly

Check:

- whether the existing primary is still visible and healthy
- whether the cluster is suffering a trust or freshness problem
- whether the new node was started against the correct scope and endpoints
