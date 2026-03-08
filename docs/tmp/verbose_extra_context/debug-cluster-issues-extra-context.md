# Extra context for docs/src/how-to/debug-cluster-issues.md

The repository already documents the debug endpoints in the HTTP API reference, but there is no goal-oriented troubleshooting guide. This extra context should help K2 stay factual about what the debug surfaces expose and what operators can infer from them.

Debug endpoint availability and scope:

- Debug endpoints are available only when `debug.enabled` is true in runtime configuration.
- The node-a, node-b, and node-c cluster configs currently set `[debug] enabled = true`.
- `src/debug_api/view.rs` shows the verbose payload sections: `meta`, `config`, `pginfo`, `dcs`, `process`, `ha`, `api`, `debug`, `changes`, and `timeline`.
- The advertised debug endpoints are `/debug/snapshot`, `/debug/verbose`, and `/debug/ui`.

How to interpret DCS trust using source-backed language:

- `src/dcs/state.rs` defines three trust states: `FullQuorum`, `FailSafe`, and `NotTrusted`.
- `NotTrusted` is returned immediately when etcd is unhealthy.
- `FailSafe` is returned when etcd is healthy but local/leader/member freshness is not good enough for normal trust, including cases where self is missing from cache, a member record is stale, or there are too few fresh members.
- `FullQuorum` is returned only when etcd is healthy and the freshness checks succeed.

How to interpret HA observations using source-backed language:

- `src/ha/decision.rs` defines decision labels such as `wait_for_postgres`, `wait_for_dcs_trust`, `attempt_leadership`, `follow_leader`, `become_primary`, `step_down`, `recover_replica`, `fence_node`, `release_leader_lease`, and `enter_fail_safe`.
- `src/api/controller.rs` maps runtime HA phase and decision state into the `/ha/state` API response.
- The debug verbose payload also includes HA phase, tick, decision, optional decision detail, and planned action count.

Testing evidence relevant to troubleshooting:

- `tests/ha/support/observer.rs` records API states and SQL roles to detect dual-primary windows and fail-safe sampling.
- The observer treats more than one primary as a split-brain incident and records recent samples with leader identity and HA phase.
- `tests/ha/support/multi_node.rs` repeatedly calls `/ha/state` as the canonical post-start observation path and uses switchover requests plus network fault injection in realistic scenarios.

Performance and limits guidance that is safe to state:

- The source shows no explicit rate limiter for debug endpoints.
- The payload is assembled from an in-memory `SystemSnapshot` plus stored change/timeline history, so response cost scales with the size of the retained debug history and the amount of JSON returned.
- It is safe to tell operators that `since=<sequence>` on `/debug/verbose` narrows the returned change/timeline history and is therefore the lighter-weight option when investigating a live system repeatedly.
- It is not safe to claim a documented production-safe polling rate, because the repository does not define one.

Practical troubleshooting boundaries:

- A how-to should focus on reading `/ha/state`, `/debug/snapshot`, and `/debug/verbose` together.
- The doc may connect `dcs.trust`, `ha.phase`, `ha.decision`, and `timeline` messages to incident diagnosis.
- The doc should not invent debug fields or claim UI screenshots/resources that are not present in source.
