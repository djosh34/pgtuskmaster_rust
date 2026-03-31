## Task: Remove `requested_by` And Add Optional `switchover_to` <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Surgically remove the current `requested_by` field from the switchover model across the entire repository and replace it with an operator-meaningful request shape that optionally accepts `switchover_to`. The higher-order goal is to ensure the public switchover API and CLI only ask operators for information that actually affects HA behavior, while preserving the current generic switchover behavior when no explicit target is requested.

This task is not allowed to leave `requested_by` behind in any form. When done, the repository must read as if `requested_by` never existed in the switchover model at all.

**Scope:**
- Remove `requested_by` completely from the public API, CLI, DCS switchover record, HA facts naming, rendered status fields, docs, tests, prompts, drafts, temporary docs artifacts, and any other repository content. Do not keep compatibility aliases, deprecated fields, transitional mentions, or explanatory leftovers that preserve the old name in code or repository text.
- Introduce a new switchover request model with optional `switchover_to`.
- When `switchover_to` is present, validate that the requested member is a valid cluster member and is eligible according to the chosen HA rules. If it is not valid or not available, the API must reject the request and the CLI must surface that failure clearly.
- When `switchover_to` is omitted, preserve the current behavior: request a generic planned switchover and let the existing successor-selection behavior continue to choose an eligible successor automatically.
- Rename or redesign any HA/API state fields that currently expose `switchover_requested_by` so they reflect real semantics, such as pending target information or a generic pending switchover indicator.
- Remove any status, docs, or tests that teach operators to think in terms of `requested_by`.

**Context from research:**
- The current switchover payload is only `SwitchoverRequest { requested_by: MemberId }` in `src/dcs/state.rs`.
- The API accepts only `requested_by` in `src/api/controller.rs`.
- The CLI requires `ha switchover request --requested-by ...` in `src/cli/args.rs` and `src/cli/client.rs`.
- The HA decision path propagates that field as `switchover_requested_by`, but current decision logic only checks whether it is present. The value itself does not control switchover destination.
- Successor choice currently comes from `available_primary_member_id` / `follow_target(...)` in `src/ha/decision.rs` and `src/ha/decide.rs`.
- Live testing confirmed this behavior: submitting a generic switchover request changed the primary, but not deterministically to the member named in `requested_by`; a later generic request changed the primary again.
- Tests do currently use `requested_by`, but only for shallow concerns:
- parser shape and required-argument behavior
- API payload shape and validation of non-empty input
- DCS write shape assertions
- e2e helper argv/log construction
- No research evidence found so far shows tests depending on `requested_by` as meaningful switchover semantics or target-selection behavior.
- That means `requested_by` is not just “used for tests”. It is wired through production API, CLI, HA state, DCS storage, docs, and tests, but it appears to be semantically useless for actual operator control.

**Expected outcome:**
- `requested_by` is gone from the repository entirely in relation to switchover behavior and operator-facing surfaces.
- Operators can request `switchover_to=<member>` when they want a targeted switchover.
- Operators can omit `switchover_to` and still get today’s generic planned switchover behavior.
- Invalid or unavailable `switchover_to` values are rejected explicitly by the API and CLI.
- Docs and tests reflect the real control model rather than a misleading placeholder field.

</description>

<acceptance_criteria>
- [ ] `requested_by` is removed from `src/api/controller.rs`, `src/api/worker.rs`, `src/api/mod.rs`, `src/cli/args.rs`, `src/cli/client.rs`, `src/cli/output.rs`, `src/dcs/state.rs`, `src/ha/decision.rs`, `src/ha/decide.rs`, and any other production switchover-path files that still expose it
- [ ] `requested_by` is removed from operator-facing docs, including `docs/src/how-to/check-cluster-health.md`, and from draft docs, prompt files, temporary docs artifacts, and any generated repository content that still mentions it
- [ ] `DecisionFacts` and related HA naming are updated so no production logic or state surface still refers to `switchover_requested_by`
- [ ] A new request model with optional `switchover_to` exists in the API, CLI, and DCS storage format
- [ ] When `switchover_to` is omitted, switchover behavior remains equivalent to the current generic planned switchover behavior
- [ ] When `switchover_to` is provided, the API validates the target and rejects unknown, invalid, or ineligible targets with a clear error response
- [ ] The CLI exposes the optional target cleanly and surfaces API validation failures clearly
- [ ] HA state and related status output expose only semantically real switchover metadata, such as pending target or generic pending switchover intent
- [ ] Tests are updated so they no longer mention or depend on `requested_by`, and they cover both generic switchover and explicit `switchover_to` behavior
- [ ] Repository-wide verification proves there are zero remaining `requested_by` references anywhere in the repo after the change, including `src/`, `tests/`, `docs/`, `docs/draft/`, `docs/tmp/`, `.ralph/`, and any other tracked content
- [ ] The final repository state reads as if the switchover field name `requested_by` never existed
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
