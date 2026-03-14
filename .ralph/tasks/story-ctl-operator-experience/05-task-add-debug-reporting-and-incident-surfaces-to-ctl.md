## Task: Add Debug Reporting And Incident Investigation Surfaces To `pgtm` <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
**Goal:** Pull the debug and incident-response workflows up into `pgtm` so operators do not need to memorize `/debug/verbose`, `?since=`, or ad-hoc archive commands during incidents. The higher-order goal is to turn the CLI into the normal place to start an investigation and leave raw debug HTTP only as the protocol substrate.

**Scope:**
- Add CLI coverage for the read-only debug surfaces that operators currently access with `curl`.
- Design the first layer of deeper inspection around `pgtm status -v`, with the table still leading the presentation.
- If separate commands are still needed after that, design the smallest possible follow-on surface for current snapshot retrieval, incremental polling with `since`, and incident archive/report workflows.
- Add human-oriented summary rendering for the high-signal debug sections while preserving raw JSON output for full fidelity.
- Rework docs that currently teach direct `/debug/verbose` polling so they can lead with the CLI and then mention the underlying API as needed.
- Keep auth and debug-disabled behavior explicit so operators can distinguish `404 debug disabled` from auth or transport failures.

**Context from research:**
- Several of the most curl-heavy docs are heavy specifically because the CLI has no debug commands today: `debug-cluster-issues`, `monitor-via-metrics`, and `handle-network-partition`.
- The runtime already exposes stable debug surfaces: `/debug/verbose`, `/debug/snapshot`, and `/debug/ui`.
- `/debug/verbose` already has a useful incremental cursor model via `since`, plus retained `changes` and `timeline` history that would benefit from a higher-level CLI wrapper.
- CloudNativePG’s `status` plus `report` split is a useful pattern: quick summary first, richer incident artifact second.
- Current design direction is to avoid inventing too many top-level nouns too early, so `status -v` should absorb as much routine inspection as practical.

**Expected outcome:**
- Operators can do common incident investigation from the CLI instead of assembling raw HTTP commands.
- The docs for network partitions, debugging, and monitoring become shorter and more coherent.
- Incident capture becomes an intentional product feature rather than a shell convention.

</description>

<acceptance_criteria>
- [x] Add CLI support for at least the stable `/debug/verbose` surface, including current-state retrieval and incremental `since` polling.
- [x] The design explicitly decides what belongs in `pgtm status -v` versus what deserves a separate debug/reporting command.
- [x] Define at least one human-readable summary mode for high-signal debug sections such as trust, leader, phase, decision, process state, and recent timeline changes.
- [x] `pgtm status -v` surfaces pginfo when debug data is available and degrades clearly when debug is disabled.
- [x] Preserve full JSON output for automation and offline archive use.
- [x] Document how the CLI handles `debug.enabled = false`, auth failures, and transport errors.
- [x] Update the debug- and incident-focused docs under `docs/src/how-to/` to lead with the new CLI paths where coverage now exists.
- [x] Tests cover normal debug retrieval, incremental polling, disabled-debug responses, and representative output rendering.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Execution Plan

### Product decisions and command surface
- Keep routine investigation anchored on `pgtm status -v`.
- Expand `status -v` so it remains the first command operators run during an incident:
  - keep the existing cluster table as the primary shape,
  - add a concise per-node debug summary sourced from `/debug/verbose`,
  - make debug availability explicit instead of silently dropping missing detail.
- Do not force every debug workflow into `status -v`. Add the smallest shallow follow-on surface for raw inspection:
  - `pgtm debug verbose` for one-node current `/debug/verbose` retrieval,
  - `pgtm debug verbose --since <sequence>` for incremental polling.
- Do not make `debug report` a required deliverable for this task unless execution proves there is already a trivial, testable way to emit an intentional artifact without adding a second discovery/archive subsystem. The task acceptance criteria are satisfied by a strong `status -v` plus `debug verbose` split; report/archive work can be captured as a follow-on task if it is not genuinely small.
- Do not add separate top-level nouns for `snapshot`, `incident`, or `report` unless implementation proves the `debug` namespace cannot stay coherent.
- Keep the global output contract consistent with the rest of `pgtm`:
  - default output is human-oriented,
  - `--json` preserves full-fidelity machine-readable payloads,
  - no extra output-format matrix.

### Explicit design split: `status -v` versus `debug`
- `pgtm status -v` should answer: what is each node doing right now, is debug available, and what high-signal incident indicators should I look at first?
- `pgtm status -v` should not try to print full `changes` and `timeline` bodies inline. That will drown out the cluster table and make watch mode noisy.
- `pgtm debug verbose` should be the raw/current retrieval surface:
  - fetch the stable `/debug/verbose` payload from the selected API base URL,
  - default human output should summarize the high-signal sections plus recent retained history,
  - `--json` should print the full payload with no lossy projection.
- `pgtm debug verbose --since <sequence>` should preserve the API cursor contract directly and return the same payload shape with filtered `changes` and `timeline`.
- `pgtm debug report` is a follow-on design point, not a default implementation commitment for this task. Only implement it if it remains genuinely small after `status -v` and `debug verbose` are complete.
- `/debug/snapshot` stays secondary in this task unless implementation of `debug report` concretely needs it; the acceptance criteria require `/debug/verbose` coverage first.

### CLI parsing and entrypoint changes
- Preserve the current `pgtm` default-to-`status` shape in [`src/cli/args.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/args.rs). Do not do a broad parser rewrite unless execution proves it is necessary.
- Add the smallest extension that keeps existing `pgtm`, `pgtm status`, `pgtm --json`, `pgtm -v`, and `pgtm --watch` behavior stable while introducing a shallow `debug` command tree.
- Keep `--json` global if that remains simplest. Keep `-v/--verbose` and `--watch` rooted in the existing `status` path unless there is a compelling reason to re-scope them.
- Introduce only the new command-specific args actually needed for this task:
  - `DebugArgs` with a `verbose` subcommand,
  - `DebugVerboseArgs` with `since`,
  - optional `report` args only if `debug report` remains genuinely in scope after implementation review.
- Update [`src/cli/mod.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/mod.rs) dispatch so:
  - `status` continues to resolve operator context once and uses cluster sampling,
  - `debug verbose` uses one API client against the configured base URL,
  - any `debug report` work is explicitly gated behind the later implementation-scope check above.

### Expand the CLI debug client from a tiny projection to the full stable payload
- Do not duplicate the full verbose schema in an unrelated CLI-only struct tree if it can be avoided. [`src/debug_api/view.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/debug_api/view.rs) already defines the response contract on the API side.
- Prefer extracting or relocating the stable `/debug/verbose` serde model into a shared module that both the API renderer and the CLI client can use, then derive both `Serialize` and `Deserialize` there. That reduces schema drift risk between the server and the CLI.
- The shared model still needs to cover the full stable `/debug/verbose` contract documented in [`docs/src/reference/debug-api.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/reference/debug-api.md):
  - `meta`,
  - `config`,
  - `pginfo`,
  - `dcs`,
  - `process`,
  - `ha`,
  - `api`,
  - `debug`,
  - `changes`,
  - `timeline`.
- Add a client method that accepts an optional `since` cursor instead of hard-coding `/debug/verbose` without query parameters.
- Keep auth behavior aligned with the stable API: read token access, same base URL/TLS handling as the other CLI calls.
- Do not introduce permissive deserialization that hides shape drift. Model the current documented fields explicitly so tests catch schema mismatches.

### Fix the current silent-debug-failure behavior in cluster status
- The current `fetch_optional_debug()` path in [`src/cli/status.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/status.rs) silently converts 401, 403, transport, and decode failures into missing debug data. That must change.
- Replace the plain `Option<DebugVerboseResponse>` enrichment with a richer per-node debug observation state, for example:
  - `Available(payload)`,
  - `Disabled`,
  - `Unavailable { reason }`.
- Preserve the current non-fatal behavior for debug-disabled or temporary debug unavailability when the stable `/ha/state` sample succeeded, but surface it explicitly in both human and JSON output.
- Differentiate at least these cases:
  - `404 Not Found`: debug disabled,
  - `401` or `403`: auth failure,
  - transport error: API reachable for status seed/member sample versus debug endpoint failed,
  - decode error or unexpected response shape,
  - `503`: debug subsystem not ready.
- Keep the main `/ha/state` command path strict as it is today: seed failures still fail the whole command; peer failures still degrade the cluster sample.

### `pgtm status -v` data model and rendering changes
- Extend [`src/cli/status.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/status.rs) and [`src/cli/output.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/output.rs) so verbose status can render more than the current `pginfo summary`, `readiness`, and `process state`.
- Keep the table-first shape, but add a compact detail block after the table when `-v` is set. That block should summarize the high-signal debug sections without dumping raw JSON:
  - DCS trust and leader,
  - HA phase, decision, and decision detail,
  - PostgreSQL variant, SQL health, readiness, and summary,
  - process worker state and last outcome/running job when useful,
  - recent change/timeline excerpts with bounded length.
- Keep the table columns narrow enough to remain readable in terminals. Do not attempt to cram full `decision_detail`, `last_outcome`, or history messages into table cells.
- For each node in verbose mode, surface an explicit debug status marker so operators can tell the difference between:
  - debug available,
  - debug disabled,
  - debug auth failure,
  - debug transport/decode failure.
- JSON status output should preserve the richer debug-observation state so automation can inspect exact reasons instead of inferring from nulls.

### `pgtm debug verbose` human and JSON output design
- Implement a dedicated renderer for the one-node debug command rather than reusing the cluster status renderer.
- Default human output should be structured for incident use:
  - header with node/member identity, cluster, scope, and current `meta.sequence`,
  - compact summary sections for `pginfo`, `dcs`, `ha`, `process`, and debug retention metadata,
  - bounded recent `changes` and `timeline` excerpts in newest-first or clearly labeled chronological order.
- `--json` must output the full stable payload exactly as deserialized, including all fields and full retained event arrays.
- `--since` should not alter the human summary semantics except that history sections show only filtered entries and the header makes the cursor clear.
- If watch mode is kept after implementation review, it should live on `pgtm debug verbose --watch` rather than inventing a second incremental-poll command.

### `pgtm debug report` artifact design
- Treat incident capture as a follow-on design point, not the default implementation target for this task.
- Only implement `pgtm debug report` if, during execution, it remains small after `status -v` and `debug verbose` are finished and verified.
- If it does stay in scope, the first implementation must remain deliberately simple:
  - gather `/debug/verbose` payloads only,
  - include metadata describing which node each payload came from and when,
  - write a deterministic JSON artifact to disk.
- Prefer a single JSON document over zip/tar packaging unless the codebase already has an obvious archive helper.
- If cluster-wide collection is too large for this task, do not ship an ambiguous partial cluster archive. Defer it cleanly instead.

### Shared internal models and refactors
- Reuse the existing sampled-cluster snapshot in [`src/cli/status.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/status.rs) for any cluster-wide report or status enrichment work rather than adding a second peer-sampling path.
- Add dedicated view types for:
  - per-node debug observation state,
  - human summary sections/excerpts,
  - report artifact payloads.
- Keep raw API payload types separate from rendered view types so the CLI can preserve exact JSON while still offering opinionated human summaries.
- Do not thread raw strings everywhere. Use typed enums/structs for debug availability and report scope so error handling stays explicit and lint-clean.

### Testing plan
- Extend parser tests in [`src/cli/args.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/args.rs) for:
  - `pgtm debug verbose`,
  - `pgtm debug verbose --since <n>`,
  - `pgtm debug report`,
  - any status-arg refactor needed to keep `status -v` and `status --watch` stable.
- Expand client tests in [`src/cli/client.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/client.rs) for:
  - `/debug/verbose` success with full payload decoding,
  - `?since=` query emission,
  - auth header behavior on debug reads,
  - representative API status and decode failures.
- Extend status assembly tests in [`src/cli/status.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/status.rs) for:
  - debug-disabled nodes,
  - debug auth failures,
  - debug transport/decode failures,
  - richer verbose summaries and JSON state preservation.
- Extend renderer tests in [`src/cli/output.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/output.rs) for:
  - table plus detail-block output in `status -v`,
  - explicit debug-status wording,
  - debug verbose human output,
  - report artifact JSON serialization only if report stays in scope.
- Add binary tests in [`tests/cli_binary.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/cli_binary.rs) for:
  - `pgtm status -v` against fake `/ha/state` plus `/debug/verbose`,
  - `pgtm debug verbose`,
  - `pgtm debug verbose --since`,
  - debug-disabled `404`,
  - auth failure `401/403`,
  - representative transport failure,
  - report creation and output path handling only if report stays in scope.
- If the task introduces report behavior, add at least one real-binary or harness-backed test that exercises the new command against an actual running node instead of mock-only coverage.

### Documentation work
- The user requested an `update-docs` skill, but no such skill is available in this session. Execution should therefore update docs directly in-repo.
- Update [`docs/src/reference/pgtm-cli.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/reference/pgtm-cli.md) to document:
  - what `status -v` now includes,
  - the new `debug verbose` command,
  - `debug report` only if it remains in scope,
  - JSON versus human behavior,
  - `--since`,
  - incident-report examples only if report ships.
- Keep [`docs/src/reference/debug-api.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/reference/debug-api.md) API-first because it is the protocol reference. Add a concise note near the top pointing operators to the CLI workflow (`pgtm status -v`, `pgtm debug verbose`) rather than turning the API reference into a how-to.
- Rewrite the curl-heavy operator guides to lead with CLI workflows where coverage now exists:
  - [`docs/src/how-to/debug-cluster-issues.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/how-to/debug-cluster-issues.md),
  - [`docs/src/how-to/handle-network-partition.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/how-to/handle-network-partition.md),
  - [`docs/src/how-to/monitor-via-metrics.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/how-to/monitor-via-metrics.md).
- Review tutorial and overview pages that currently teach direct `/debug/verbose` polling and shorten or remove stale curl-first sections:
  - [`docs/src/tutorial/debug-api-usage.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/tutorial/debug-api-usage.md),
  - [`docs/src/tutorial/observing-failover.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/tutorial/observing-failover.md),
  - [`docs/src/tutorial/single-node-setup.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/tutorial/single-node-setup.md),
  - [`docs/src/how-to/overview.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/how-to/overview.md),
  - [`docs/src/reference/overview.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/reference/overview.md).
- Rebuild generated docs rather than editing built output by hand.

### Required verification and finish gates for the execution pass
- After implementation, run all required gates with no skipping:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- Only after those succeed and docs are updated should the execution pass:
  - mark `<passes>true</passes>`,
  - run `/bin/bash .ralph/task_switch.sh`,
  - commit all files including `.ralph`,
  - push with `git push`.

NOW EXECUTE
