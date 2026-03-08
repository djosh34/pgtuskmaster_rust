# Verbose extra context for docs/src/reference/ha-decisions.md

This note is intentionally exhaustive and based only on the requested sources plus targeted searches.

## Why this page belongs in reference

- `src/ha/decision.rs` defines the decision enum and its payload types.
- `src/api/controller.rs` maps those internal variants into API response variants.
- `docs/src/reference/http-api.md` already exposes `ha_decision` in `GET /ha/state`, but it does not currently explain each variant in one place.
- `docs/src/explanation/architecture.md` explains the phase machine conceptually, but not as a reference catalog.

## Decision catalog

- `HaDecision::NoChange`
- `HaDecision::WaitForPostgres { start_requested, leader_member_id }`
- `HaDecision::WaitForDcsTrust`
- `HaDecision::AttemptLeadership`
- `HaDecision::FollowLeader { leader_member_id }`
- `HaDecision::BecomePrimary { promote }`
- `HaDecision::StepDown(StepDownPlan)`
- `HaDecision::RecoverReplica { strategy }`
- `HaDecision::FenceNode`
- `HaDecision::ReleaseLeaderLease { reason }`
- `HaDecision::EnterFailSafe { release_leader_lease }`

## Related payload types

- `StepDownPlan` contains:
- `reason`
- `release_leader_lease`
- `clear_switchover`
- `fence`
- `StepDownReason` variants:
- `Switchover`
- `ForeignLeaderDetected { leader_member_id }`
- `RecoveryStrategy` variants:
- `Rewind { leader_member_id }`
- `BaseBackup { leader_member_id }`
- `Bootstrap`
- `LeaseReleaseReason` variants:
- `FencingComplete`
- `PostgresUnreachable`

## API projection facts

- `src/api/controller.rs` maps each decision to a corresponding `HaDecisionResponse`.
- That mapping is the source of the JSON contract returned from `GET /ha/state`.
- The reference page should preserve the exact JSON-facing structure:
- `wait_for_postgres` has `start_requested` and optional `leader_member_id`
- `follow_leader` carries `leader_member_id`
- `become_primary` carries `promote`
- `step_down` expands to `reason`, `release_leader_lease`, `clear_switchover`, `fence`
- `recover_replica` carries nested `strategy`
- `release_leader_lease` carries nested `reason`
- `enter_fail_safe` carries `release_leader_lease`

## Phase machine context

- `src/ha/state.rs` defines the HA phases:
- `Init`
- `WaitingPostgresReachable`
- `WaitingDcsTrusted`
- `WaitingSwitchoverSuccessor`
- `Replica`
- `CandidateLeader`
- `Primary`
- `Rewinding`
- `Bootstrapping`
- `Fencing`
- `FailSafe`
- Decisions are emitted by the phase handlers in `src/ha/decide.rs`.
- `docs/src/explanation/architecture.md` already explains that the system is phase-driven and trust-gated; this page should cross-link that explanation rather than re-explaining the whole architecture.

## Global trust gate

- At the top of `decide_phase`, if `facts.trust` is not `FullQuorum`, normal HA progression is bypassed.
- If local PostgreSQL is primary while trust is degraded, the outcome is `HaPhase::FailSafe` with `HaDecision::EnterFailSafe { release_leader_lease: false }`.
- If local PostgreSQL is not primary while trust is degraded, the outcome is `HaPhase::FailSafe` with `HaDecision::NoChange`.
- That means many decision variants are only reachable while trust is `FullQuorum`.

## Important per-variant behavior cues from the decision logic

- `WaitForPostgres` is the startup or recovery hold state while PostgreSQL is unreachable or not ready to continue. It can include `start_requested = true` when the worker is allowed to ask the process layer to start Postgres.
- `WaitForDcsTrust` is a distinct hold state used when PostgreSQL is reachable enough to continue but DCS trust has not yet stabilized for normal HA decisions.
- `AttemptLeadership` is used when a node should try to acquire leadership rather than follow another member.
- `FollowLeader` carries the selected leader member id and corresponds to replica-follow behavior.
- `BecomePrimary { promote }` differentiates between becoming primary without promotion and promotion-based takeover. The `promote` boolean matters and should not be flattened away in docs.
- `StepDown` is not a single reason string. It is a structured plan with side-effect booleans that tell the apply/lower layers whether to release the lease, clear switchover, and fence.
- `RecoverReplica` is also structured. Recovery can happen by rewind, base backup, or bootstrap.
- `FenceNode` is the hard safety action when the node should stop serving writable primary behavior.
- `ReleaseLeaderLease` is narrower than step-down and has its own reason enum.
- `EnterFailSafe` is the trust-degradation decision. Its `release_leader_lease` boolean indicates whether fail-safe entry should also release the current leader lease.

## Lowering to effect plans

- Targeted search in `src/ha/lower.rs` shows the decisions are later lowered into effect plans.
- `NoChange` and `WaitForDcsTrust` lower to no-op plans.
- `WaitForPostgres` can carry a postgres-start effect when `start_requested` is true.
- `AttemptLeadership` lowers to lease-acquisition effects.
- `FollowLeader` lowers to replication-follow effects.
- `RecoverReplica` lowers to recovery effects keyed by strategy.
- `FenceNode` and `EnterFailSafe` both influence safety effects.
- `ReleaseLeaderLease` lowers to a dedicated release effect.
- `StepDown` lowers to a combination of safety, replication, lease, and switchover effects based on the booleans in `StepDownPlan`.
- This is useful reference context because the decision names are not arbitrary labels; they are the compact control surface for downstream effect dispatch.

## Relationship to `GET /ha/state`

- The HA state response currently includes:
- cluster identity
- DCS trust
- HA phase
- HA tick
- HA decision
- snapshot sequence
- That means operators already see one decision at a time through `/ha/state`.
- This page should define the full decision vocabulary so operators can interpret that field without reading Rust enums.

## Important documentation boundaries

- Keep this page dry and technical.
- Do not turn it into a tutorial or incident playbook.
- Cross-link the architecture page for flow explanations and the HTTP API page for endpoint/auth details.
- The main value of this page is authoritative variant-by-variant semantics and field definitions.
