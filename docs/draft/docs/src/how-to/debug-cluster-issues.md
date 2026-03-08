// todo: remove placeholder diagram labels and keep only real markdown/mermaid/ascii content that is valid in mdBook.
// todo: verify serialized enum values against the API wire format. Existing published docs use lower_snake_case values such as `full_quorum`, `fail_safe`, and `not_trusted`, so do not describe CamelCase API values unless source proves it.
# Debug Cluster Issues

This guide shows you how to diagnose PostgreSQL high-availability cluster problems using pgtuskmaster debug endpoints and interpret the key signals that indicate system health.

## What you will need

- A running pgtuskmaster cluster with `[debug] enabled = true` in runtime configuration
- Network access to cluster nodes on the API port (default 8080)
- curl or similar HTTP client
- A text editor or JSON viewer to inspect responses

## Step 1: Check /debug/snapshot for a system overview

The `/debug/snapshot` endpoint returns a lightweight JSON snapshot of current system state.

```bash
curl http://node-a:8080/debug/snapshot
```

**What to observe:**

- `meta.app_lifecycle` shows if the node is Running, Starting, or Faulted
- `meta.sequence` provides a monotonic counter for change ordering
- `changes[]` lists recent domain updates with version numbers
- `timeline[]` records diagnostic messages from each domain

The snapshot gives you a quick health status and reveals which subsystems have changed recently.

## Step 2: Check /ha/state for high-availability status

The `/ha/state` endpoint exposes HA decision logic and cluster membership.

```bash
curl http://node-a:8080/ha/state
```

**Key fields to interpret:**

- `dcs_trust`: One of `FullQuorum`, `FailSafe`, or `NotTrusted`
- `ha_phase`: Current phase (e.g., Primary, Replica, WaitForPostgres)
- `ha_decision`: Last decision label (e.g., `wait_for_dcs_trust`, `follow_leader`)
- `leader`: Current leader member ID, if any
- `member_count`: Total members in DCS cache

**DCS trust states explained:**
```
etcd_healthy? ──NO──> NotTrusted (etcd unreachable)
     │
    YES
     │
     └─> self_in_cache? ──NO──> FailSafe (local record missing)
            │
           YES
            │
            └─> leader_fresh? ──NO──> FailSafe (stale leader)
                 │
                YES
                 │
                 └─> enough_fresh_members? ──NO──> FailSafe (insufficient quorum)
                      │
                     YES
                      │
                      └─> FullQuorum (normal operation)
```

- **FullQuorum**: etcd is healthy, local member record is fresh, and leader metadata is current. Normal operations can proceed.
- **FailSafe**: etcd is healthy but the local node cannot confirm a full quorum. This occurs when your member record is stale, the leader record is stale, or there are too few fresh members. The system enters a protective mode.
- **NotTrusted**: etcd is unreachable. The node cannot trust DCS state and will not make leadership decisions.

## Step 3: Check /debug/verbose for detailed diagnostics

For deeper investigation, use `/debug/verbose` to see complete subsystem state with change history.

```bash
# Full verbose output
curl http://node-a:8080/debug/verbose

# Only changes after sequence 42
curl http://node-a:8080/debug/verbose?since=42
```

**What to inspect:**

**Config section**
- `version` and `updated_at_ms`: confirm config reloads
- `cluster_name`, `member_id`, `scope`: verify node identity
- `tls_enabled`, `debug_enabled`: check security and debug settings

**PgInfo section**
- `variant`: Unknown, Primary, or Replica
- `sql`: Healthy, Unknown, or Unreachable
- `readiness`: Ready, NotReady, or Unknown
- `timeline`: PostgreSQL timeline ID
- `summary`: one-line status including LSN positions

**DCS section**
- `trust`: DCS trust state
- `member_count` and `leader`: cluster membership view
- `has_switchover_request`: indicates pending switchover

**HA section**
- `phase`: current HA phase
- `decision`: last decision label
- `decision_detail`: optional details (e.g., `leader_member_id=node-b`)
- `planned_actions`: number of queued actions

**Process section**
- `state`: Idle or Running
- `running_job_id`: active job, if any
- `last_outcome`: result of last completed job

**Timeline and Changes sections**
- `timeline[]`: chronological diagnostic messages
- `changes[]`: state version updates by domain

## Step 4: Interpret HA decisions from debug data

HA decisions appear in both `/ha/state` and the verbose `ha.decision` field.
```
PostgreSQL state + DCS state → DecisionEngine → ha.decision label → Planned actions
```

Decision labels and their meaning:

- `wait_for_postgres`: PostgreSQL is not reachable or not ready. Check `pginfo.sql` and `pginfo.readiness`.
- `wait_for_dcs_trust`: DCS trust is not FullQuorum. Check `dcs.trust` and etcd health.
- `attempt_leadership`: Node is attempting to become leader. Verify `leader` field after a few seconds.
- `follow_leader`: Node will follow a remote leader. Check `ha.decision_detail` for `leader_member_id`.
- `become_primary`: Node is promoting itself to primary. Monitor `pginfo.variant` and `pginfo.timeline`.
- `step_down`: Node is stepping down (switchover or foreign leader detected).
- `recover_replica`: Node is recovering using rewind, basebackup, or bootstrap.
- `fence_node`: Fencing operation in progress.
- `release_leader_lease`: Releasing leadership lease (fencing complete or postgres unreachable).
- `enter_fail_safe`: Entering fail-safe mode. Check `dcs.trust` and `process.state`.

## Step 5: Correlate timeline messages with symptoms

The `timeline[]` array records diagnostic messages in temporal order.

**Common patterns:**

- Sequence of `pginfo` updates showing SQL status transitions from `Unknown` → `Healthy`
- `ha` decision changes from `wait_for_dcs_trust` → `attempt_leadership` → `become_primary`
- `process` job outcomes such as `Success`, `Failure`, or `Timeout`
- `dcs` trust changes from `NotTrusted` → `FailSafe` → `FullQuorum` as etcd stabilizes

When debugging an incident:
1. Note the `meta.sequence` at symptom onset
2. Request `/debug/verbose?since=<sequence>` to limit history
3. Scan `timeline[]` for error messages or state transitions near the incident time
4. Cross-reference `changes[]` to identify which domains updated

## Expected outcomes

After following these steps you should:
- Know the DCS trust state and why it is that value
- Understand the current HA phase and decision for each node
- Identify whether PostgreSQL is reachable and in which role (primary/replica)
- Detect if a process job is running or failed
- Have a timeline of diagnostic events leading to the current state

## What to do next

If you identify a problem:
- **DCS NotTrusted**: Check etcd connectivity and health
- **DCS FailSafe**: Verify member freshness and network partitions
- **PostgreSQL Unreachable**: Check PostgreSQL process status and logs
- **Unexpected HA decision**: Review `timeline[]` and `changes[]` for triggering events
- **Split-brain risk**: Monitor multiple nodes showing `Primary` phase simultaneously

For deeper investigation, consult the HTTP API reference for all endpoint details and the failure modes explanation for theoretical background.
