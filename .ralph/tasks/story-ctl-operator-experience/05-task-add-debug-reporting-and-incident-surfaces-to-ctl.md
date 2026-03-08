## Task: Add Debug Reporting And Incident Investigation Surfaces To `pgtm` <status>not_started</status> <passes>false</passes>

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
- [ ] Add CLI support for at least the stable `/debug/verbose` surface, including current-state retrieval and incremental `since` polling.
- [ ] The design explicitly decides what belongs in `pgtm status -v` versus what deserves a separate debug/reporting command.
- [ ] Define at least one human-readable summary mode for high-signal debug sections such as trust, leader, phase, decision, process state, and recent timeline changes.
- [ ] `pgtm status -v` surfaces pginfo when debug data is available and degrades clearly when debug is disabled.
- [ ] Preserve full JSON output for automation and offline archive use.
- [ ] Document how the CLI handles `debug.enabled = false`, auth failures, and transport errors.
- [ ] Update the debug- and incident-focused docs under `docs/src/how-to/` to lead with the new CLI paths where coverage now exists.
- [ ] Tests cover normal debug retrieval, incremental polling, disabled-debug responses, and representative output rendering.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
