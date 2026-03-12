# HA State Semantics

This page describes the HA-specific fields exposed through `GET /ha/state`.

The current API no longer publishes the old `ha_phase` and `ha_decision` enums. The stable contract is now:

- `dcs_trust`: whether the node trusts the DCS view enough to run normal HA
- `authority`: the operator-facing primary-authority projection this node is publishing
- `fence_cutoff`: the lease epoch and committed LSN cutoff used when the node must stop unsafe primary behavior
- `ha_role`: the local role the node is trying to hold right now
- `planned_actions`: the ordered reconcile actions the node intends to execute next

That split matters:

- `authority` answers "who should operators currently treat as primary?"
- `ha_role` answers "what is this node trying to do locally?"
- `planned_actions` answers "what concrete work is queued from that choice?"

## Authority

`authority` is a tagged enum.

### `primary`

```text
{
  "kind": "primary",
  "member_id": "node-a",
  "epoch": {
    "holder": "node-a",
    "generation": 7
  }
}
```

Use this when the node is publishing a concrete primary authority view.

The embedded lease epoch matters operationally:

- `holder` is the member id for the published authority
- `generation` distinguishes successive leader leases for the same member

### `no_primary`

```text
{
  "kind": "no_primary",
  "reason": {
    "kind": "recovering"
  }
}
```

This means the node is deliberately not projecting any primary authority.

`reason.kind` can be:

- `dcs_degraded`: DCS trust is not good enough for normal HA
- `lease_open`: no trusted primary lease is currently held
- `recovering`: the node is still converging or restarting and cannot safely publish a primary
- `switchover_rejected`: a targeted switchover request was invalid, with a blocker payload

When `reason.kind` is `switchover_rejected`, the blocker is:

- `target_missing`
- `target_ineligible`, with one of:
  - `not_ready`
  - `lagging`
  - `partitioned`
  - `api_unavailable`
  - `starting_up`

### `unknown`

```text
{
  "kind": "unknown"
}
```

This is the cold-start publication value before the HA worker has produced a stronger authority projection.

## Fence Cutoff

`fence_cutoff` is present when the node must retain enough information to stop an unsafe primary cleanly.

```text
{
  "epoch": {
    "holder": "node-a",
    "generation": 7
  },
  "committed_lsn": 12345678
}
```

This payload is emitted together with `authority.kind = "no_primary"` in safety-sensitive situations such as:

- DCS trust degradation while the local node is primary
- storage-stall fencing while the local node is primary

## Local HA Role

`ha_role` is a tagged enum describing the node's local intent.

### `leader`

```text
{
  "kind": "leader",
  "epoch": {
    "holder": "node-a",
    "generation": 7
  }
}
```

The node believes it holds the current leader lease and is acting as the leader-side role.

### `candidate`

```text
{
  "kind": "candidate",
  "candidacy": {
    "kind": "failover"
  }
}
```

`candidacy.kind` can be:

- `bootstrap`
- `failover`
- `resume_after_outage`
- `targeted_switchover`, with `member_id`

### `follower`

```text
{
  "kind": "follower",
  "goal": {
    "leader": "node-a",
    "recovery": "rewind"
  }
}
```

`goal.recovery` can be:

- `none`
- `start_streaming`
- `rewind`
- `basebackup`

### `fail_safe`

```text
{
  "kind": "fail_safe",
  "goal": {
    "kind": "primary_must_stop",
    "cutoff": {
      "epoch": {
        "holder": "node-a",
        "generation": 7
      },
      "committed_lsn": 12345678
    }
  }
}
```

`goal.kind` can be:

- `primary_must_stop`
- `replica_keep_following`, with optional `upstream`
- `wait_for_quorum`

### `demoting_for_switchover`

```text
{
  "kind": "demoting_for_switchover",
  "member_id": "node-b"
}
```

The node is stepping down specifically so the named target can take over.

### `fenced`

```text
{
  "kind": "fenced",
  "reason": "foreign_leader_detected"
}
```

`reason` can be:

- `foreign_leader_detected`
- `storage_stalled`

### `idle`

```text
{
  "kind": "idle",
  "reason": {
    "kind": "awaiting_leader"
  }
}
```

`reason.kind` can be:

- `awaiting_leader`
- `awaiting_target`, with `member_id`

## Planned Actions

`planned_actions` is the ordered reconcile plan derived from `ha_role` and `authority`.

Possible action kinds are:

- `init_db`
- `base_backup`
- `pg_rewind`
- `start_primary`
- `start_replica`
- `promote`
- `demote`
- `acquire_lease`
- `release_lease`
- `publish`
- `clear_switchover`

The action payloads carry the additional information the worker needs:

- recovery actions name the source member id
- `demote` includes the shutdown mode
- `acquire_lease` includes the candidacy kind
- `publish` embeds the authority projection that should become visible to operators

The list is ordered. The worker executes it in sequence, so this field is the closest stable explanation of what the node plans to do next without reading the debug internals.

## Relationship To Debug Output

The debug surface still uses the legacy field names `ha.phase` and `ha.decision`, but they now mean:

- `ha.phase`: the string label for `ha_role`
- `ha.decision`: the compact string form of `authority`
- `ha.decision_detail`: a debug-oriented detail string for the current role

Use `/ha/state` for automation. Use `/debug/verbose` when you need additional narrative detail while investigating a node.
