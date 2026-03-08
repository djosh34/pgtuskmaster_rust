# Verbose extra context for docs/src/how-to/add-cluster-node.md

This note is intentionally exhaustive and source-first. It supports a how-to page for adding a node to an existing cluster.

## Scope reality from the sources

- I do not see a single high-level "join node" function in the requested files.
- The requested files instead show the configuration requirements, the DCS membership model, the HA phase progression, and test harness patterns that stand up or add nodes in realistic scenarios.
- The how-to therefore needs to be procedural and evidence-based, not a claim that there is one dedicated join RPC.

## Configuration prerequisites

- `src/config/schema.rs` shows a joining node needs a full runtime config, not just a node id.
- Relevant top-level sections are:
- `cluster`
- `postgres`
- `dcs`
- `ha`
- `process`
- `logging`
- `api`
- `debug`
- The node-specific identity lives in:
- `cluster.name`
- `cluster.member_id`
- DCS cluster membership ties to:
- `dcs.endpoints`
- `dcs.scope`
- HA timing depends on:
- `ha.loop_interval_ms`
- `ha.lease_ttl_ms`
- Runtime binaries and process operations depend on the `process.binaries` block.
- PostgreSQL listen/reachability settings come from `postgres.listen_host`, `postgres.listen_port`, `postgres.socket_dir`, and role/auth settings.

## Concrete example from docker config

- `docker/configs/cluster/node-a/runtime.toml` is a concrete complete runtime config example.
- It shows:
- cluster identity
- PostgreSQL listen host/port
- DCS endpoint and scope
- HA timing
- process binary paths
- API listen address
- optional debug enablement
- A how-to can use that file as the base template, changing member-specific identity and addresses for the new node.

## How membership appears in the DCS model

- `src/dcs/state.rs` defines `MemberRecord` with:
- `member_id`
- `postgres_host`
- `postgres_port`
- `role`
- `sql`
- `readiness`
- `timeline`
- `write_lsn`
- `replay_lsn`
- `updated_at`
- `pg_version`
- `build_local_member_record` constructs the local record from the node's PostgreSQL state and publish context.
- That means a node "joins" operationally by starting with valid config, publishing its member record, and then participating in HA decisions as trust and topology allow.

## HA progression for a new node

- `src/ha/state.rs` lists the phases a node can pass through.
- `src/ha/decide.rs` is the source for how the node progresses.
- Useful high-level operator interpretation for a new node:
- it starts from `Init`
- waits for PostgreSQL reachability
- waits for DCS trust
- then either follows a leader as a replica, attempts leadership, or performs recovery/bootstrap actions depending on observed world state
- For a normal scale-out join into an existing healthy cluster, the expected steady-state goal is replica/follower behavior, not immediate leadership.

## Network/port answer from the loaded sources

- Evidence from the requested files clearly shows DCS connectivity is required via `dcs.endpoints`.
- `MemberRecord` explicitly advertises `postgres_host` and `postgres_port`, so PostgreSQL connectivity between nodes is part of replication/follow/recovery behavior.
- The docker example exposes the API on `listen_addr = "0.0.0.0:8080"`, and the tests use node API endpoints extensively for observation/control.
- Evidence-based operator answer:
- beyond DCS connectivity, PostgreSQL network reachability on each node's configured `postgres.listen_host` and `postgres.listen_port` is required for replication and recovery workflows
- the API listener port is also operationally relevant for observation and control tooling, even if HA membership itself is modeled through DCS and PostgreSQL state
- Phrase the API-port point as operationally relevant rather than as a strict membership requirement unless the source explicitly demands it

## Test-harness cues

- `tests/ha/support/multi_node.rs` contains the real cluster fixture patterns for multi-node HA scenarios.
- The harness tracks node handles, API state polling, SQL workload checks, and convergence windows.
- That is good support for a how-to that tells operators to:
- prepare the node config first
- start the new node
- confirm it appears in DCS-backed cluster state
- watch HA/API state until it converges as a replica and stays healthy

## What the page should help the operator verify

- The new node has the correct `cluster.name` and a unique `cluster.member_id`.
- It points at the correct DCS scope and endpoints.
- Its PostgreSQL and process binary paths are valid.
- The node successfully publishes membership.
- Existing nodes continue to agree on the leader.
- The new node settles into an expected replica/follower state rather than triggering unexpected fail-safe or fencing behavior.
