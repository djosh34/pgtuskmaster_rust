# HA State Semantics

The HA engine is exposed inside the `ha` section of `GET /state`.

The most important fields are:

- `ha.publication.authority`
- `ha.publication.fence_cutoff`
- `ha.role`
- `ha.planned_commands`
- `ha.world`

That split is deliberate:

- `publication.authority` answers who operators should currently treat as primary
- `role` answers what this node is trying to do locally
- `planned_commands` answers what concrete work is queued next
- `world` exposes the raw derived worldview that produced those results

## `ha.publication.authority`

`authority` is a tagged enum:

- `primary`
- `no_primary`
- `unknown`

`primary` carries the current primary member plus a lease epoch.

`no_primary` carries a reason:

- `dcs_degraded`
- `lease_open`
- `recovering`
- `switchover_rejected`

`unknown` is the cold-start value before HA has produced a stronger publication.

## `ha.publication.fence_cutoff`

Present when the node must retain enough information to stop unsafe primary behavior.

It includes:

- `epoch`
- `committed_lsn`

Typical cases:

- trust degraded while this node was primary
- storage stall while this node was primary

## `ha.role`

`role` is the node's target local role:

- `leader`
- `candidate`
- `follower`
- `fail_safe`
- `demoting_for_switchover`
- `fenced`
- `idle`

These values describe desired local behavior, not only externally visible status.

## `ha.planned_commands`

`planned_commands` is the ordered reconcile plan that the worker will try to execute.

Possible command kinds include:

- `init_db`
- `base_backup`
- `pg_rewind`
- `start_primary`
- `start_replica`
- `promote`
- `demote`
- `acquire_lease`
- `release_lease`
- `ensure_required_roles`
- `publish`
- `clear_switchover`

## `ha.world`

`world` exposes the HA engine's current derived worldview:

- `local`: what this node knows about its local data directory, postgres state, process state, storage, publication, and observations
- `global`: what this node currently trusts about DCS, leases, switchover state, and peer eligibility

Use `ha.world` when you need the engine's raw interpretation inputs rather than only its operator-facing publication.
