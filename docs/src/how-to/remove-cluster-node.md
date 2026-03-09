# Remove a Cluster Node

This guide describes the most conservative operator-facing removal flow supported by the current codebase: move leadership away first when needed, stop the node externally, then verify the remaining cluster converges through `pgtm`.

## Goal

Decommission a node without leaving the rest of the cluster in an unclear HA state.

## Important boundary

The requested source files establish these limits:

- there is no dedicated remove-node job in the process layer
- there is no dedicated remove-node key in the DCS model
- there is no first-class operator API or CLI for deleting `member/<member_id>` records

The normal supported control surfaces are:

- observe state through `pgtm status`
- request a switchover through the switchover CLI
- stop the node or its PostgreSQL process externally in the environment you run

## Step 1: If the node is primary, move leadership first

Do not begin by deleting DCS state. If the node you want to remove is currently primary, use the planned switchover flow first so that another node becomes leader cleanly.

After the switchover, confirm from a surviving node that:

- `LEADER` points at the new primary
- `TRUST` is healthy
- the node you are removing is no longer the active leader

## Step 2: Stop the node externally

Use the service mechanism that matches your environment. The codebase models node loss as an external event, not as an internal HA command.

Examples of environment-specific actions:

- stop the service unit that runs `pgtuskmaster`
- stop the container that runs the node
- if needed, stop PostgreSQL on that host as part of your decommission flow

## Step 3: Watch the surviving cluster converge

Poll cluster state from at least one surviving node:

```bash
pgtm -c /etc/pgtuskmaster/config.toml status -v
```

Keep checking until the remaining cluster has a stable view of:

- the current leader
- the current trust level
- the local HA phase on the nodes you care about

If the cluster view is degraded or warns about incomplete sampling, repeat the same command from another surviving node's operator config before you conclude the cluster has converged.

## Step 4: Understand what the DCS layer actually stores

Member presence is represented by ordinary member records under:

```text
/{scope}/member/{member_id}
```

The DCS helpers in the requested sources provide explicit deletion support for:

- `/{scope}/leader`
- `/{scope}/switchover`

They do not provide a dedicated helper for operator-driven member removal.

## Step 5: Treat manual member-key deletion as unsupported cleanup

The watch/update path can observe member-key deletion, but that is not the same thing as a documented operator workflow.

If your operational policy requires cleaning up a stale member key manually:

- do it only after the node is permanently decommissioned
- do it only after the remaining cluster has converged without that node
- treat it as manual DCS surgery, not as the standard path

The key pattern is:

```text
/{scope}/member/{member_id}
```

## Verification

Before you consider the node removed, check that:

- a surviving node reports the expected leader
- the surviving cluster is not stuck in `fail_safe`
- the removed node is no longer participating in the topology you observe
- any application or load-balancer references to the old node have been removed

## What this page does not claim

This page does not claim that:

- a dedicated remove-node command exists
- stale member keys are always harmless
- manual etcd key deletion is the preferred operator path
