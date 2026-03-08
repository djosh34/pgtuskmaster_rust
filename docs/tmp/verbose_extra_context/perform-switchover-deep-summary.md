# Perform Switchover Deep Summary

This file gathers only source-backed context for `docs/src/how-to/perform-switchover.md`.

## Operator entry points

- The API controller accepts a switchover request through `post_switchover(scope, store, input)` in `src/api/controller.rs`.
- The request body is a JSON object with one field, `requested_by`.
- Blank or whitespace-only `requested_by` values are rejected with a bad request error.
- A successful request serializes `SwitchoverRequest { requested_by }` and writes it to the DCS key `/<scope>/switchover`.
- Clearing a switchover uses `DELETE /ha/switchover`, which delegates to `DcsHaWriter::clear_switchover`.
- Both request and clear return `AcceptedResponse { accepted: true }` when the DCS write/delete succeeds.

## CLI surface and transport behavior

- The user-facing CLI syntax comes from `src/cli/args.rs`.
- The command tree is `pgtuskmasterctl ha switchover request --requested-by <ID>` and `pgtuskmasterctl ha switchover clear`.
- `requested_by` is required for the request subcommand.
- The CLI supports `--base-url`, `--read-token`, `--admin-token`, `--timeout-ms`, and `--output`.
- Runtime help output confirms the top-level flags and command structure.
- The HTTP client implementation lives in `src/cli/client.rs`.
- `delete_switchover()` sends `DELETE /ha/switchover` and expects HTTP 202.
- `post_switchover(requested_by)` sends `POST /switchover` with JSON body `{"requested_by":"..."}` and expects HTTP 202.
- Read operations can use either the read token or the admin token, but admin operations use only the admin token when provided.
- Non-expected HTTP status codes are surfaced as CLI API-status errors with the response body included.

## HA state and trust conditions

- `src/dcs/state.rs` defines three trust states: `FullQuorum`, `FailSafe`, and `NotTrusted`.
- Trust evaluation returns `NotTrusted` immediately when the DCS backend is unhealthy.
- Trust evaluation returns `FailSafe` when the local member is missing, stale, the leader record is stale, or the cluster has more than one member but fewer than two fresh records.
- Only `FullQuorum` allows the normal HA state machine to proceed.
- `src/ha/decide.rs` begins by checking trust.
- If trust is not `FullQuorum` and PostgreSQL is primary, the node enters `FailSafe` with `EnterFailSafe { release_leader_lease: false }`.
- If trust is not `FullQuorum` and PostgreSQL is not primary, the node also enters `FailSafe`, but with `NoChange`.
- This means switchover should be documented as requiring a healthy, trusted cluster view rather than something that works during `FailSafe` or `NotTrusted`.

## Switchover decision mechanics

- `src/dcs/state.rs` stores the switchover intent as `SwitchoverRequest { requested_by: MemberId }` in the DCS cache.
- `src/ha/decision.rs` exposes `switchover_requested_by` in `DecisionFacts`.
- In `src/ha/decide.rs`, a primary node that sees `switchover_requested_by.is_some()` while it is leader transitions to `HaPhase::WaitingSwitchoverSuccessor`.
- The paired decision is `HaDecision::StepDown(StepDownPlan { reason: Switchover, release_leader_lease: true, clear_switchover: true, fence: false })`.
- The current leader therefore releases the leader lease and clears the switchover marker as part of the switchover step-down path.
- While in `WaitingSwitchoverSuccessor`, the former leader waits until some other leader record appears and then follows that leader as a replica.
- In replica phase, a node with an active leader record equal to itself can become primary with `BecomePrimary { promote: true }`.
- If a switchover request exists but the node is already a replica and also the active leader, the code returns `NoChange`.
- The source files examined here do not add any explicit replica-lag threshold gating to switchover acceptance.
- The key documented guardrail that is directly source-backed is DCS trust: non-`FullQuorum` trust pushes the cluster into fail-safe handling instead of normal switchover progression.

## Observable state for verification

- `get_ha_state()` in `src/api/controller.rs` exposes `leader`, `switchover_requested_by`, `member_count`, `dcs_trust`, `ha_phase`, `ha_tick`, and `ha_decision`.
- This makes `/ha/state` the canonical source-backed way to watch switchover progress.
- A switchover can therefore be verified by observing:
- no current switchover before the request,
- `switchover_requested_by` appearing after the request,
- HA phase/decision movement through step-down and follower states,
- and a new `leader` after convergence.

## Test-backed operational expectations

- `tests/ha/support/multi_node.rs` provides the strongest end-to-end switchover evidence.
- The helper `request_switchover_via_cli()` retries the CLI request across every node API endpoint because the former primary may be transiently unavailable while replicas are still healthy enough to accept the operator request.
- That helper uses JSON output and requires the decoded response to contain `accepted: true`.
- The higher-level helper `request_switchover_until_stable_primary_changes()` retries switchover attempts, waits for a stable primary, and falls back to looser primary-change detection if stable convergence takes too long.
- The test harness therefore treats primary change plus stable observation as the success criterion, not just API acceptance.
- `tests/ha/support/observer.rs` enforces a no-dual-primary invariant during sampled observation windows.
- The observer records HA API states and SQL roles and fails if more than one primary is observed.
- That makes "single primary throughout the transition" a source-backed verification point for the how-to.

## Example configuration facts from docker config

- `docker/configs/cluster/node-a/runtime.toml` sets `cluster.member_id = "node-a"`.
- The example scope is `docker-cluster`.
- The HA loop interval is `1000` ms.
- The lease TTL is `10000` ms.
- API auth in this specific example is disabled.
- These values are examples from the docker config, not general defaults for every deployment.
