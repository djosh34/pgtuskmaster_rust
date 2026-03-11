## Verbose extra context for `docs/src/how-to/handle-complex-failures.md`

This file exists to give K2 raw factual material that connects the HA decision code, DCS trust model, existing operator CLI contracts, and the greenfield Docker HA scenarios that exercise complex failure combinations.

### Scope this doc should cover

The requested target is a how-to page for handling complex failures. In this repo, "complex" means more than a single clean primary crash. The strongest source-backed examples currently come from the greenfield HA wrappers that combine:

- quorum loss
- majority/minority partitions
- mixed API, DCS, and postgres-path isolation
- fencing windows
- repeated leadership churn
- concurrent SQL during failover

The runtime and CLI contracts are conservative. The how-to should therefore distinguish:

- conditions where operators should observe and wait for automatic recovery
- conditions where operators should stop trusting the cluster-wide answer and escalate investigation

### The two most important source-backed timing knobs

From `docker/configs/cluster/node-a/runtime.toml`:

- `[ha] loop_interval_ms = 1000`
- `[ha] lease_ttl_ms = 10000`

Implications:

- The HA loop reevaluates roughly once per second.
- Freshness and trust degradation are bounded by a 10 second lease TTL in the Docker cluster config K2 is documenting.
- Docs should not promise instant failover or immediate trust restoration.
- If the system is healthy, operator-visible convergence should usually happen on the scale of a few HA loops plus lease expiry, not sub-second.

### DCS trust model that operators need to understand

From `src/dcs/state.rs`:

- `DcsTrust` has exactly three states: `FullQuorum`, `FailSafe`, `NotTrusted`.
- `NotTrusted` is used when etcd itself is unhealthy.
- `FailSafe` is used when etcd is healthy but the runtime does not have enough fresh member information to trust normal HA decisions.

The exact `evaluate_trust(...)` rules in source are:

1. If `etcd_healthy` is false, return `DcsTrust::NotTrusted`.
2. If the local member is missing from the DCS cache, return `DcsTrust::FailSafe`.
3. If the local member record is stale relative to `ha.lease_ttl_ms`, return `DcsTrust::FailSafe`.
4. If the observed view does not have fresh quorum, return `DcsTrust::FailSafe`.
5. Otherwise return `DcsTrust::FullQuorum`.

The helper `has_fresh_quorum(...)` is intentionally conservative:

- if the observed member set length is `<= 1`, exactly one fresh member is trusted
- otherwise the cache requires at least two fresh members

This means the doc must not claim that a lone survivor in a multi-member cluster is healthy just because it is still reachable.

### HA decision behavior during degraded trust

From `src/ha/decide.rs`, the decision engine starts with a trust gate before ordinary phase-specific logic:

- If trust is not `FullQuorum` and local postgres is primary, the node enters `FailSafe` with `HaDecision::EnterFailSafe { release_leader_lease: false }`.
- If trust is not `FullQuorum` and local postgres is not primary, the node enters `FailSafe` with `HaDecision::NoChange`.

Operational meaning:

- complex failure handling starts with trust, not with a generic "try to elect something anyway" rule
- if trust is degraded, the system prioritizes safety over availability
- the operator should expect `FailSafe` and `WaitForDcsTrust` style behavior instead of aggressive self-healing

Other decision details that matter for the doc:

- `WaitingDcsTrusted` can move to `FollowLeader`, `AttemptLeadership`, `RecoverReplica`, or stay waiting depending on trust and follow target.
- `Primary` can step down and fence when a foreign leader record appears.
- `Fencing` transitions to `ReleaseLeaderLease` on success and `EnterFailSafe { release_leader_lease: false }` on failure.
- `FailSafe` is not a one-shot terminal decision. It reevaluates:
  - if fencing is still running, it holds
  - if local postgres is primary, it routes through `decide_primary(...)`
  - if the node still believes it is leader, it can emit `ReleaseLeaderLease`
  - otherwise it waits for DCS trust to recover

The doc should therefore frame complex failures as trust-restoration and authority-restoration problems, not as simple restart problems.

### Operator-visible CLI signals already documented elsewhere

From `docs/src/reference/pgtm-cli.md`, `pgtm primary` is intentionally strict and fails closed when it cannot form an authoritative write-target answer. The documented failure conditions include:

- no sampled primary
- multiple sampled primaries
- incomplete peer sampling
- leader disagreement or membership disagreement across sampled nodes
- missing PostgreSQL host or port metadata

This is critical for the how-to:

- a failing `pgtm primary` during a complex fault is often the correct safety signal
- the operator should not treat "no answer" as automatically broken behavior
- the operator should treat a clean single-target answer as stronger evidence than a raw local node guess

The greenfield HA cucumber steps reinforce the same contract:

- `Then there is no operator-visible primary across N online node[s]`
- `Then pgtm primary points to "<member>"`

Those steps are effectively executable operator contracts.

### Existing docs that already define part of the operator posture

Existing docs already say:

- `docs/src/how-to/handle-primary-failure.md`
  - most ordinary primary failures do not require manual intervention
  - operators should confirm recovery by checking `pgtm status -v` for one primary, healthy trust, and no warning lines
  - operators should escalate if multi-primary views persist or if the majority side does not recover after lease expiry
- `docs/src/how-to/debug-cluster-issues.md`
  - `pgtm status -v` is the first operator entrypoint
  - operators should compare answers from more than one seed when they suspect disagreement
- `docs/src/reference/dcs-state-model.md`
  - explains what `FailSafe` and `NotTrusted` mean
- `docs/src/reference/ha-decisions.md`
  - documents `EnterFailSafe`, `ReleaseLeaderLease`, and other decisions that appear in debug output

The new how-to should not restate all of those pages. It should connect them into an operational procedure for compound failures.

### Source-backed examples from the greenfield HA wrappers

#### Majority partition where the old primary is isolated

Wrapper:

- `cucumber_tests/ha/features/full_partition_majority_survives_old_primary_isolated/full_partition_majority_survives_old_primary_isolated.rs`

Scenario intent:

- isolate the old primary onto the one-node minority across etcd, API, and postgres paths
- keep the healthy two-node majority talking to each other
- wait for exactly one majority-side primary

Recent trustworthy failure evidence from `make test-long`:

- wrapper: `ha_full_partition_majority_survives_old_primary_isolated`
- failing step: `Then exactly one primary exists across 2 running nodes as "majority_primary"`
- observed error: `cluster has no sampled primary`
- warnings included insufficient sampling and an unreachable isolated node, while the majority pair remained visible

This is the kind of signal the how-to should explain:

- if the majority remains observable but `pgtm primary` still cannot form an answer after lease expiry, the operator should stop assuming auto-recovery
- this is not the same as a temporary failover blip

#### No-quorum fencing contract

Wrapper:

- `cucumber_tests/ha/features/no_quorum_fencing_blocks_post_cutoff_commits/no_quorum_fencing_blocks_post_cutoff_commits.rs`

Scenario intent:

- start a concurrent workload
- remove DCS quorum majority while workload is active
- determine the fencing cutoff from observed workload results
- verify post-cutoff commits are rejected or bounded

Recent trustworthy failure evidence from `make test-long`:

- wrapper: `ha_no_quorum_fencing_blocks_post_cutoff_commits`
- failing step: `Then there is no operator-visible primary across 3 online node`
- observed error: `expected pgtm primary via 'node-a' to fail, but it returned targets: node-b`

This is exactly the kind of condition where docs should tell operators to escalate:

- when quorum is gone, an operator-visible primary answer is not the healthy expected outcome
- this is different from ordinary auto-recovery

### Practical operator-visible warning strings that deserve to be named in the doc

The greenfield suite and observer code repeatedly surface these warning shapes:

- `degraded_trust=...`
- `leader_mismatch=...`
- `insufficient_sampling=sampled X/Y`
- `unreachable_node=...`
- `expected pgtm primary via '<seed>' to fail, but it returned targets: ...`
- `cluster has no sampled primary`

These warnings all indicate that the cluster-wide answer is degraded or contradictory.

Good source-backed operator guidance:

- if `pgtm status -v` shows one primary, healthy trust, and no warning lines, the operator can usually wait for auto-recovery to finish
- if warnings like `leader_mismatch`, `insufficient_sampling`, or sustained `degraded_trust` persist past the expected lease window, the operator should stop relying on automatic convergence and move into diagnosis mode
- if a no-quorum condition still yields an operator-visible primary, treat that as a safety incident, not a reassuring sign

### When docs should say "wait" versus "intervene"

Use this source-backed distinction:

Wait and observe when:

- trust loss is recent and still within the expected lease-expiry window
- `pgtm status -v` is trending toward one primary and warnings are disappearing
- restarted replicas are catching up and authority is converging

Move to manual investigation when:

- `pgtm primary` keeps failing closed long after the expected lease window even though a majority should be healthy
- `pgtm primary` still returns a target during no-quorum conditions
- `pgtm status -v` keeps showing `leader_mismatch`, `insufficient_sampling`, or persistent degraded trust
- different operator seed configs disagree for longer than a transient failover window
- a restarted node remains in `unknown`, `fail_safe`, or repeatedly restarts instead of rejoining cleanly

### Facts K2 should not overstate

- The current source does not justify a blanket claim that every complex failure auto-recovers.
- The current test corpus explicitly contains trustworthy failing advanced scenarios.
- The docs should explain the intended observation and response flow without promising that all advanced scenarios are already bug-free.
- The docs should stay aligned with the conservative CLI contract: "no authoritative answer" is often safer than a misleading answer.
