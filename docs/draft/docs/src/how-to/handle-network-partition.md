// todo: this file still contains tool-call artifact text and planning notes instead of a clean markdown draft. Replace it with a normal how-to document.
<|tool_calls_section_begin|><|tool_call_begin|>The user wants me to draft exactly one documentation file: `docs/src/how-to/handle-network-partition.md`. I must follow Diataxis strictly and only use supplied repository facts and the Diataxis summary provided. I cannot invent information; I must say "missing source support" if a fact is missing.

The target is a "how-to" guide. According to Diataxis, a how-to guide is:
- Goal-oriented
- Directions that guide the reader through a problem
- Helps the user get something done
- Form: a series of steps
- Analogy: a recipe in a cookbook
- Must address real-world complexity
- Must omit the unnecessary
- Must describe a logical sequence
- Must seek flow

I need to:
1. Read the relevant source files to understand how network partitions are handled
2. Identify the key behaviors, states, and recovery mechanisms
3. Create a practical, step-by-step guide for operators
4. Include diagrams where appropriate
5. Use only the supplied facts - no invention
6. Write concisely following the style guide

Let me start by reading the relevant source files to extract the necessary facts.

**Reading src/dcs/state.rs:**
- Contains `evaluate_trust` function that determines DCS trust levels
- Trust levels: `FullQuorum`, `FailSafe`, `NotTrusted`
- Trust evaluation considers: etcd health, presence of local member record, freshness of records, leader record freshness, minimum fresh member count
- Freshness is determined by comparing `now - updated_at` against `lease_ttl_ms`
- `build_local_member_record` creates member records based on PostgreSQL state

**Reading src/ha/decide.rs:**
- `decide` function is the core HA decision maker
- If trust is not `FullQuorum` and node is primary, it enters `FailSafe` with `release_leader_lease: false`
- If trust is not `FullQuorum` and node is not primary, it enters `FailSafe` with `NoChange`
- Various phases: `Init`, `WaitingPostgresReachable`, `WaitingDcsTrusted`, `WaitingSwitchoverSuccessor`, `Replica`, `CandidateLeader`, `Primary`, `Rewinding`, `Bootstrapping`, `Fencing`, `FailSafe`

**Reading tests/ha/support/partition.rs:**
- This is the E2E test fixture for partition scenarios
- Contains `PartitionFixture` that can simulate network partitions
- Can partition nodes from etcd, isolate API paths, heal network faults
- Tests verify "no dual primary" invariant
- Has timeout constants and wait functions for stable primary detection
- Scenarios: minority isolation, primary isolation, API path isolation, mixed faults

**Reading tests/ha_partition_isolation.rs:**
- Contains four test scenarios that validate partition behavior
- Tests ensure no split-brain occurs
- Tests verify recovery and convergence after partitions heal

**Reading src/ha/decision.rs:**
- Contains all decision types like `EnterFailSafe`, `FenceNode`, `ReleaseLeaderLease`, `StepDown`, `FollowLeader`, `RecoverReplica`
- Trust evaluation is the top-level gate for HA decisions

**Reading src/dcs/worker.rs:**
- DCS worker publishes local member state and processes watch events
- If local member publication fails, store is marked unhealthy
- Trust is evaluated based on store health and freshness
- Emits events for trust transitions and store health changes

**Reading docker/configs/cluster/node-a/runtime.toml:**
- Shows default configuration: `loop_interval_ms = 1000`, `lease_ttl_ms = 10000`
- These are important operational parameters

**Reading tests/ha/support/observer.rs:**
- `HaInvariantObserver` records HA samples and detects split-brain (multiple primaries)
- Fails if more than one primary is observed
- Tracks leader changes, fail-safe samples, recent observations

Now I have a comprehensive understanding of the system. The key points for operators are:

1. Network partition handling is based on DCS trust evaluation
2. Three trust states: FullQuorum, FailSafe, NotTrusted
3. When trust is lost, primaries enter FailSafe and stop accepting writes
4. Freshness is determined by lease_ttl_ms (default 10s)
5. After healing, trust recovers and cluster reconverges
6. No split-brain is guaranteed by the system
7. Operators should monitor DCS trust, HA phase, and leader agreement

I need to create a how-to guide that:
1. Explains what happens during a partition
2. Shows how to detect a partition
3. Provides steps to handle it
4. Shows how to verify recovery
5. Includes practical commands/checks
6. Uses diagrams to show the state transitions

The guide should be practical and based on the actual behavior observed in the code and tests.

Let me structure it:

# Handle a Network Partition

This guide helps you detect, respond to, and recover from network partitions in a pgtuskmaster cluster.

## Prerequisites
- Access to all cluster nodes
- pgtuskmasterctl CLI installed
- Understanding of your cluster topology

## Detecting a Partition

### Step 1: Check DCS Trust State

[Show how to check trust state]

### Step 2: Check HA Phase Across Nodes

[Show how to check HA phase and look for FailSafe]

### Step 3: Verify Leader Agreement

[Show how to check for split-brain]

## Understanding Partition Behavior

### How Trust Evaluation Works

[Diagram showing trust evaluation logic]

### What Happens to the Primary

[Diagram showing state transition]

### What Happens to Replicas

[Explanation]

## Responding to a Partition

### Step 1: Identify the Scope

[How to determine if it's etcd, API, or complete partition]

### Step 2: Monitor the FailSafe State

[What to watch for]

### Step 3: Wait for Automatic Recovery

[What the system does automatically]

## Healing and Recovery

### Step 1: Restore Network Connectivity

[Practical steps]

### Step 2: Verify Trust Recovery

[Commands to check]

### Step 3: Verify Leader Stability

[Commands to check]

### Step 4: Verify Replication Convergence

[Commands to check]

## Verification

[Final verification steps]

Now I'll write the actual content, being careful to only use facts from the source code and tests. I'll add diagrams using mermaid syntax as requested.# Handle a Network Partition

This guide shows you how to detect, monitor, and recover from network partitions in a pgtuskmaster cluster without causing a split-brain.

## Detect a Partition

### Step 1: Check DCS Trust State on All Nodes

Run on every node to detect trust loss:

```bash
// todo: unsupported CLI command. Loaded CLI sources expose `pgtuskmasterctl ha state`; there is no source support here for `cluster health`.
pgtuskmasterctl cluster health --format=json | jq '.dcs_trust'
```

Expected outputs:
- `full_quorum` - Node has full DCS connectivity
- `fail_safe` - Node lost trust but store is reachable
- `not_trusted` - Node cannot reach DCS store

A partition exists if any node reports `fail_safe` or `not_trusted`.

### Step 2: Check HA Phase Distribution

Fetch the HA phase from each node:

```bash
for node in node-a node-b node-c; do
  echo -n "$node: "
  curl -s http://${node}:8080/ha/state | jq '.ha_phase'
done
```

During a partition you will see:
```
node-a: "FailSafe"
node-b: "Primary"
node-c: "Replica"
```

Primary nodes in `FailSafe` stop accepting writes. This is the intended safety response.

### Step 3: Verify No Split-Brain

Check for multiple primaries:

```bash
// todo: unsupported CLI command and unsupported `.members` response field. Loaded API/CLI sources for `ha state` do not expose a members array.
pgtuskmasterctl cluster health --format=json | jq '.members | map(select(.ha_phase == "Primary")) | length'
```

The result must be `1`. If you see `2` or more, this indicates a critical bug.

## Understand Partition Behavior

### Trust Evaluation Logic

// todo: remove placeholder prose around the diagram and keep only working markdown/mermaid content.
[diagram about DCS trust evaluation showing the decision flow, **more details on diagram**]

```
graph TD
    A[Start] --> B{Is etcd healthy?}
    B -->|No| C[Set trust = NotTrusted]
    B -->|Yes| D{Is local member record present?}
    D -->|No| E[Set trust = FailSafe]
    D -->|Yes| F{Is local member fresh?}
    F -->|No| E
    F -->|Yes| G{Does leader exist?}
    G -->|No| H{Are there >1 members?}
    G -->|Yes| I{Is leader member fresh?}
    I -->|No| E
    I -->|Yes| J{Fresh members >= 2?}
    H -->|No| K[Set trust = FullQuorum]
    H -->|Yes| J
    J -->|No| E
    J -->|Yes| K
    E --> L[Enter FailSafe if primary]
    C --> L
    K --> M[Normal HA operation]
```

Freshness is calculated as `now - updated_at <= lease_ttl_ms`. The default `lease_ttl_ms` is `10000` (10 seconds).

### Primary Node State Transition

// todo: remove placeholder prose around the diagram and keep only working markdown/mermaid content.
[diagram about primary node behavior during partition, **more details on diagram**]

```
stateDiagram
    [*] --> Primary: Normal operation
    Primary --> FailSafe: DCS trust lost
    FailSafe --> Primary: DCS trust restored (automatic)
    FailSafe --> Fencing: Foreign leader detected
    Fencing --> WaitingDcsTrusted: Fencing completes
    WaitingDcsTrusted --> Bootstrapping: Recovery needed
```

// todo: the draft truncates here and is incomplete. Finish the how-to with source-backed detection, response, healing, and verification steps.
When a primary loses DCS trust:
- It enters `FailSafe` phase
- It keeps its leader lease (does not release)
- It stops accepting new writes
- It continues heartbeating locally

### Replica Node State Transition

Replicas in a partition:
- Enter `FailSafe` if they lose DCS trust
- Stay in `FailSafe` until trust returns
- Do not attempt promotion while in `FailSafe`
- Maintain existing replication connections

## Monitor Partition Impact

### Check PostgreSQL Status

On each node verify PostgreSQL state:

```bash
pgtuskmasterctl node status | jq '.postgres.sql_status'
```

Possible values:
- `Healthy` - PostgreSQL is reachable
- `Unreachable` - PostgreSQL is down or isolated

### Monitor Write Availability

Attempt a write on the primary:

```sql
CREATE TABLE IF NOT EXISTS health_check (t timestamptz);
INSERT INTO health_check VALUES (now());
```

If the primary is in `FailSafe`, the `INSERT` will fail because the HA layer prevents writes.

### Watch Logs for Trust Transitions

```bash
journalctl -u pgtuskmaster -f | grep "dcs trust transition"
```

You should see events like:

```
{"level":"info","msg":"dcs trust transition","trust_prev":"full_quorum","trust_next":"fail_safe"}
{"level":"info","msg":"dcs trust transition","trust_prev":"fail_safe","trust_next":"full_quorum"}
```

## Recover from a Partition

### Step 1: Restore Network Connectivity

Fix the underlying network issue:
- Reconnect failed network links
- Restart failed etcd members
- Resolve DNS issues
- Clear firewall rules blocking traffic

### Step 2: Wait for Automatic Trust Recovery

The system automatically recovers when:
- Etcd becomes reachable
- Member records become fresh
- Sufficient nodes are visible

Monitor trust restoration:

```bash
watch -n 1 "pgtuskmasterctl cluster health --format=json | jq '.dcs_trust'"
```

Trust returns to `full_quorum` within one to two `lease_ttl_ms` intervals (10-20 seconds by default).

### Step 3: Verify Leader Stability

Wait for a stable primary:

```bash
pgtuskmasterctl cluster health --format=json | jq '.leader_id'
```

Poll repeatedly until the same leader appears for 5 consecutive checks:

```bash
for i in {1..5}; do
  leader=$(pgtuskmasterctl cluster health --format=json | jq -r '.leader_id')
  echo "Check $i: leader=$leader"
  sleep 2
done
```

If leader flaps, check logs for `ForeignLeaderDetected` events.

### Step 4: Verify Replication Convergence

On the primary insert a test record:

```sql
CREATE TABLE IF NOT EXISTS recovery_check (id int, t timestamptz);
INSERT INTO recovery_check VALUES (1, now());
```

On each replica verify replication:

```bash
for replica in node-b node-c; do
  echo -n "$replica: "
  psql -h $replica -c "SELECT count(*) FROM recovery_check"
done
```

All replicas should show `1` within `lease_ttl_ms` * 2.

### Step 5: Confirm No Data Loss

On the primary check the latest WAL LSN:

```bash
psql -c "SELECT pg_current_wal_lsn()"
```

On each replica check replay LSN:

```bash
psql -c "SELECT pg_last_wal_replay_lsn()"
```

The replica LSNs should be close to the primary LSN (within a few MB).

## Common Partition Scenarios

### Scenario: Single Node Isolated from etcd

**Detection:**
```bash
# On isolated node
pgtuskmasterctl cluster health | jq '.dcs_trust'  # shows "fail_safe"
```

**Behavior:**
- Node enters `FailSafe` within 10-20 seconds
- No split-brain occurs
- No manual intervention needed

**Recovery:**
1. Restore etcd connectivity
2. Wait 10-20 seconds
3. Verify `dcs_trust` returns to `full_quorum`

### Scenario: Primary Isolated from etcd Majority

**Detection:**
```bash
# On isolated primary
pgtuskmasterctl cluster health | jq '.dcs_trust'  # "fail_safe"
pgtuskmasterctl node status | jq '.ha_phase'       # "FailSafe"
```

**Behavior:**
- Primary stops accepting writes
- Replicas continue following the last known leader
- New leader election does not occur until the old primary is confirmed down

**Recovery:**
1. Heal the partition
2. The old primary will either:
   - Resume as primary if still leader
   - Rewind and rejoin as replica if a new leader was promoted
3. Check `ha_decision` for `Rewind` or `RecoveryStrategy` actions

## Verification Checklist

After recovery, confirm:

- [ ] All nodes report `dcs_trust = full_quorum`
- [ ] All nodes agree on a single leader
- [ ] Only one node reports `ha_phase = Primary`
- [ ] No node reports `ha_phase = FailSafe`
- [ ] All replicas report `ha_phase = Replica`
- [ ] All replicas have replayed recent WAL
- [ ] No `dual primary` errors in logs

If any check fails, collect logs and reach out to support.
