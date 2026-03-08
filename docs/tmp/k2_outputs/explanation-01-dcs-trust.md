# Why DCS trust and coordination shape the cluster

The DCS is shared coordination state, not passive storage. Before pgtuskmaster treats the DCS as authoritative, the worker must both publish its own live member record and observe that the store is healthy. This two-part gating, failure logging, and continuation with unhealthy state creates a trust model that directly determines HA behavior, API design, and operational posture.

## Trust as a precondition for authority

Trust requires two independent signals. First, the worker must successfully publish its local member state. If publication fails, trust falls to NotTrusted immediately. Second, the worker must drain watch events, apply them to its cache, and refresh without error. Any failure in write, drain, or refresh marks the store unhealthy, sets WorkerStatus::Faulted in the published DcsState, and forces DcsTrust::NotTrusted. Only when both succeed does the system derive FullQuorum or FailSafe trust.

This design treats trust as an input to HA decisions, not as a separate advisory metric. The HA state machine cannot ignore trust and proceed with normal phase handling; trust overrides the entire decision. If the DCS is not trusted, the worker does not have authority to act on external state, even if the data appears intact. This prevents split-brain scenarios where a partitioned node might mistakenly assume leadership based on stale or incomplete information.

## How trust changes fail-safe behavior

When trust is not FullQuorum, the HA decision logic enters a fail-safe mode with different outcomes depending on local Postgres role. If Postgres is primary, the next phase is EnterFailSafe with release_leader_lease set to false. This keeps the lease held while blocking further promotion actions, giving operators time to assess the situation. If Postgres is not primary, the phase becomes FailSafe with NoChange, which freezes the replica in place and avoids triggering unnecessary elections or failovers.

[diagram about trust gating and lease ownership]

The conditional lease release, the NoChange path for replicas, and the fact that the worker continues to run while logging failures together enforce a conservative, observable failure mode. The system degrades gracefully rather than panicking or making aggressive recovery attempts that could amplify an outage.

## Consequences for APIs, tests, and operators

The HTTP API exposes HA state and switchover requests through the DCS-backed model, not through direct leader steering. Switchover requests are written into DCS scope paths and become part of the shared state that the worker consumes. The API reports HA state from snapshots of the internal model, not from direct inspection of Postgres or the DCS. This means clients cannot force a primary assignment; they can only request a change that the HA logic will evaluate once trust is restored.

The e2e policy test enforces this design by forbidding direct DCS writes or internal worker calls after startup. Tests may observe state via GET /ha/state and may issue admin switchover requests, but they may not circumvent the trust mechanism. This policy mirrors production expectations: operators interact with the system through its APIs, not by manually editing DCS keys or calling internal functions.

This separation of concerns, clean API surface, and hands-off test posture ensure that trust remains the single source of truth for coordination authority.
