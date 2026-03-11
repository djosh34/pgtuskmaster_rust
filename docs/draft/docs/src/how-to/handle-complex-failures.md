```md
# Handle Complex Failures

This guide shows how to diagnose and respond when a PostgreSQL cluster experiences failures that involve multiple nodes, quorum loss, or split network partitions. These scenarios require careful observation before manual intervention.

## When to Use This Guide

Use this procedure when:

- A primary failure persists beyond lease expiry and the majority does not re-establish a single primary
- `pgtm status -v` shows warnings like `degraded_trust`, `leader_mismatch`, or `insufficient_sampling`
- `pgtm primary` fails closed with "cluster has no sampled primary" or other errors after recovery window
- Network partitions isolate multiple nodes simultaneously
- You observe conflicting primary states across different nodes or operator seeds

For simple primary crashes with automatic recovery, see [Handle Primary Failure](handle-primary-failure.md). For debug-first diagnosis, see [Debug Cluster Issues](debug-cluster-issues.md).

// todo: Replace this placeholder with a real mermaid diagram during final cleanup. The current draft still contains an instruction marker, not valid mdBook content.
[diagram about DCS trust states showing FullQuorum -> quorum loss -> FailSafe/NotTrusted -> restoration path, with operator observation points and lease expiry window]

## Understand Complex Failure Modes

Complex failures differ from simple node crashes because they involve **trust degradation** in the distributed consensus store. The system prioritizes safety over availability until quorum is restored.

### DCS Trust States

The HA decision engine evaluates trust before any recovery action:

- **FullQuorum**: Healthy majority connectivity; normal failover proceeds
- **FailSafe**: DCS is reachable but insufficient fresh member information exists; system enters quiescent mode
- **NotTrusted**: DCS itself is unreachable (etcd failure)

The cluster configuration defines the timing boundaries:

```
[ha]
loop_interval_ms = 1000
lease_ttl_ms = 10000
```

- Loop evaluation: ~1 second
- Lease freshness window: 10 seconds
- Expected convergence: within a few loops plus lease expiry

Decision logic (`src/ha/decide.rs`) routes all non-FullQuorum states into FailSafe handling before phase-specific decisions execute.

## Diagnosis: Check Trust and Sampling

Run these commands on at least two nodes to detect partition or inconsistency:

```bash
# Check local node status and warnings
pgtm status -v

# Check cluster-wide primary target (fails closed for safety)
pgtm primary
```

### Expected Healthy Output

// todo: This sample `pgtm status -v` output shape is illustrative, but the requested sources do not support these exact rendered field names or lines. Replace it with a source-backed example or remove it.
After recovery, you should see:

```
$ pgtm status -v
cluster: healthy
trust: FullQuorum
primary: node-b
members: 2 healthy, 0 stale
warnings: (none)
```

If you see warnings, record them before proceeding.

### Warning Signals That Indicate Complex Failure

| Warning | Meaning | Operator Action |
|---------|---------|----------------|
| `degraded_trust=FailSafe` or `NotTrusted` | Insufficient fresh members for quorum | Wait for lease expiry or restore connectivity |
| `leader_mismatch=<member>` | Leader lease disagreement across nodes | Check partition; compare seeds |
| `insufficient_sampling=X/Y` | Could not reach all members | Network or etcd issue |
| `unreachable_node=<member>` | Specific member not responding | Isolate network fault |

If `pgtm primary` shows no target and warnings persist past lease window, do **not** manually promote nodes. This is the correct safety response.

## Step-by-Step Diagnosis

### Step 1: Identify the Failure Pattern

Use `pgtm status -v` on multiple nodes to map the failure pattern.

#### Pattern A: Majority Partition Survives, Old Primary Isolated

// todo: The exact per-node trust split described here (`two nodes FullQuorum`, isolated node `FailSafe` or `NotTrusted`) is directionally plausible but not proven by the requested sources as written. Tighten this to source-backed operator observations.
- Two nodes show `trust: FullQuorum` and agree on single primary
- One node shows `degraded_trust=FailSafe` or `trust: NotTrusted`
- Isolated node may still report itself as primary in local view

This matches the scenario `full_partition_majority_survives_old_primary_isolated` from the test suite.

**Wait for lease expiry** (10 seconds by config). The majority should converge. If `pgtm primary` still fails closed after this window, proceed to escalation.

#### Pattern B: No Quorum, Cluster-Wide

- All nodes show `trust: FailSafe` or `NotTrusted`
- `pgtm primary` fails with "cluster has no sampled primary"
- DCS (etcd) may be unreachable or lacks fresh member records

The system enters FailSafe to prevent split-brain. This matches the `no_quorum_fencing_blocks_post_cutoff_commits` scenario.

**Do not attempt promotion.** The correct response is to restore DCS connectivity or member freshness.

#### Pattern C: Stale Leader Lease After Network Heal

- One node shows `leader_mismatch` warning
- Majority exists but old primary still holds lease
- System enters `Fencing` phase then `FailSafe`

This occurs when a previously isolated primary rejoins after majority has already promoted a new primary. The decision engine (`decide_primary`) detects foreign leader record and triggers fencing.

Wait for fencing to complete and lease release. Monitor with:

```bash
pgtm status -v  # watch for "warnings: (none)"
```

### Step 2: Determine Wait vs. Intervene

Use this decision flow based on trust state and timing:

// todo: Replace this placeholder with a real mermaid diagram during final cleanup. The current draft still contains an instruction marker, not valid mdBook content.
[diagram about operator decision tree starting with trust state check, then lease expiry check, then warning persistence check, leading to either "wait" or "escalate to manual investigation"]

#### Wait When

- Trust loss is within the 10-second lease window
- `pgtm status -v` warnings are decreasing (e.g., `insufficient_sampling` resolves as members come back)
- `pgtm primary` is trending toward a single clean target
- A majority of nodes agree on one primary after transient failover

#### Escalate to Manual Investigation When

// todo: The concrete `15 seconds`, `20 seconds`, and `30 seconds` thresholds in this section are not source-backed by the requested files. Keep guidance relative to `lease_ttl_ms` unless you add evidence for exact multipliers.
- `pgtm primary` remains failed closed **more than 15 seconds** after expected lease expiry even though majority appears reachable
- `pgtm primary` returns a target **during** no-quorum conditions (this violates safety contract)
- `pgtm status -v` shows persistent `leader_mismatch` or `degraded_trust` across multiple nodes for >20 seconds
- Different operator seed configs disagree on primary target for longer than transient failover window
- A node remains in `unknown` or `fail_safe` and repeatedly fails to rejoin after restart attempts

### Step 3: Restore Trust Before Authority

If you must intervene, restore DCS trust first. Do not manually promote PostgreSQL instances.

#### If DCS is Unreachable (NotTrusted)

1. Check etcd cluster health independent of PostgreSQL nodes
2. Restore etcd connectivity
3. Wait for lease TTL to elapse
4. Re-run `pgtm status -v` until `trust: FullQuorum` appears

#### If Quorum is Insufficient (FailSafe)

1. Identify which members are stale or unreachable
// todo: "Restart stale members to refresh their member records" may be reasonable advice, but the requested sources do not establish it as the correct generic operator procedure for every complex failure. Rephrase more conservatively unless a source is added.
2. Restart stale members to refresh their member records
3. Ensure at least **two members** report fresh status within lease window (conservative quorum rule)
4. Watch `pgtm status -v` for `trust: FullQuorum`

The quorum evaluation requires at least two fresh members in any multi-member cluster, per `has_fresh_quorum` logic.

### Step 4: Verify Convergence

Once trust is restored, verify automatic recovery completed:

```bash
# Should return single primary target
pgtm primary

# Should show no warnings and consistent primary
pgtm status -v
```

If convergence does not occur within 30 seconds after trust restoration, collect debug snapshots and escalate.

## Common Scenarios and Commands

### Scenario: Isolated Primary After Partition Heals

When a minority primary rejoins the majority:

```bash
# On isolated node, expect fencing then FailSafe
pgtm status -v
# Expected: phase: FailSafe, warnings: degraded_trust=FailSafe

# Wait for lease expiry and fencing completion
sleep 15

# Should transition to replica automatically
pgtm status -v
# Expected: phase: Replica, trust: FullQuorum
```

// todo: This sequence uses unsupported exact timings and exact phase/output examples. The requested sources support fail-safe/fencing concepts, but not this exact operator transcript or guaranteed automatic transition text.
Do **not** run manual `pg_ctl promote` or `pg_rewind`. The HA worker handles this via the `Fencing` -> `WaitingDcsTrusted` -> `Bootstrapping` -> `Replica` path.

### Scenario: Majority Remains, But `pgtm primary` Fails Closed

If majority exists but `pgtm primary` fails:

```bash
# On two separate nodes, compare:
// todo: The requested sources do not support a `--seed` flag on `pgtm status -v`. Replace this with the real supported way to run the command from multiple operator seed configs.
pgtm status -v --seed node-a
pgtm status -v --seed node-b

# If these disagree on primary or warnings, you have a partition
# If they agree but `pgtm primary` still fails, wait for lease expiry + 5 seconds

# If still failing after wait, check:
// todo: The requested sources document `pgtm debug verbose` and the `/debug/snapshot` HTTP surface, but they do not establish this exact CLI command or `--target` flag. Replace with a source-backed debug capture path.
pgtm debug snapshot --target /tmp/snapshot.json
```

// todo: The exact field names `foreign_leader_record` and `lease_mismatch` are not established by the requested sources in this rendered debug workflow. Tighten this to source-backed debug concepts or cite the exact payload fields.
The debug snapshot captures `DecisionFacts` and `HaState` used by the decision engine. Inspect for `foreign_leader_record` or `lease_mismatch` conditions.

## Safety Constraints

Never perform these actions during complex failures:

- **Manual promotion** of PostgreSQL: bypasses fencing and lease safety
- **Deleting DCS keys** to "clear state": leads to split-brain risk
- **Restarting all nodes simultaneously**: destroys quorum context
- **Forcing lease acquisition** via manual etcd edits: violates FailSafe contract

The CLI contracts are intentionally conservative. `pgtm primary` fails closed precisely to prevent unsafe writes during degraded states.

## Missing Source Support

The following operational details are not yet documented in source:

- Specific metrics and alert thresholds for degraded trust
- Recommended timeout multipliers for clusters larger than three nodes
- Manual procedure for restoring etcd quorum from backup
- Step-by-step reconciliation when `pgtm primary` returns a target during no-quorum

When you encounter these gaps, prioritize safety and escalate to maintainers with debug snapshot data.

## Summary

Complex failures are trust-restoration problems, not promotion problems. Observe trust state, wait for lease expiry, and let the `FailSafe` -> `WaitingDcsTrusted` -> normal phase progression occur. Intervene only when trust is restored yet convergence fails, and then only by restoring DCS connectivity, not by manual PostgreSQL promotion.
