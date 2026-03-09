## Task: Add Cluster-Wide `pgtm status` UX With Topology And Table Output <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
**Goal:** Give operators one high-signal command, `pgtm status`, that answers “what is my cluster doing right now?” without forcing them to loop over nodes, compare JSON by hand, or infer cluster topology from several separate requests. The higher-order goal is to make PGTuskMaster’s operator CLI materially better than a collection of shell snippets while keeping the command surface minimal.

**Scope:**
- Design and implement a cluster-oriented `status` command that polls one or more configured node APIs and synthesizes a single cluster view.
- Add a human-first default output that is intentionally compact: a health indication plus a cluster table as the main presentation.
- Support only two output modes for now: default human output and `--json`.
- Use `status -v` as the main path for deeper operator detail instead of multiplying command nouns too early.
- Keep the command tree flat and minimal: no `ha` namespace and no extra grouping keywords for simple operator actions.
- Add `status --watch` because repeated observation is a first-class operator workflow.
- Consider a table or topology view that resembles the usefulness of `patronictl list` and CloudNativePG’s `kubectl cnpg status`, but adapt it to PGTuskMaster’s HA vocabulary and available signals.
- Ensure the command handles partial failures and disagreement explicitly instead of silently hiding unreachable nodes.

**Context from research:**
- Current CLI text output only reports a single node snapshot as `key=value` lines.
- Many docs still use shell loops over `curl` plus `jq` to compare leader, trust, and phase across nodes.
- Patroni provides cluster-level list/topology affordances.
- CloudNativePG provides a human-friendly status summary and a richer `--verbose` expansion path.
- The repo already has clear source-backed cluster signals to summarize: `leader`, `self_member_id`, `member_count`, `dcs_trust`, `ha_phase`, `ha_decision`, and `snapshot_sequence`.
- Current local design direction is to shorten the operator binary name to `pgtm` and make `status` the default mental entry point for operators.
- The current design direction is also that `pgtm` with no subcommand should behave like `pgtm status`.
- The current stable public API does not yet expose a full cluster member list in a stable machine-oriented payload, so this task may need to add or extend a stable API payload that `pgtm status` can consume.

**Expected outcome:**
- Operators can run one command and immediately see which node is primary, which are replicas, whether trust is degraded, and whether nodes disagree.
- The docs can replace most multi-node `curl` loops with a single ctl command.
- The product gets a clearer operator identity instead of stopping at a low-level transport wrapper.

Expected output direction:

```text
cluster: prod-eu1  health: healthy

NODE    SELF  ROLE     TRUST         PHASE    API
node-a  *     primary  full_quorum   primary  ok
node-b        replica  full_quorum   replica  ok
node-c        replica  full_quorum   replica  ok
```

With pending switchover:

```text
cluster: prod-eu1  health: healthy
switchover: pending -> node-b

NODE    SELF  ROLE     TRUST         PHASE    API
node-a  *     primary  full_quorum   primary  ok
node-b        replica  full_quorum   replica  ok
node-c        replica  full_quorum   replica  ok
```

Verbose direction:

```text
cluster: prod-eu1  health: degraded
warning: node-c unreachable

NODE    SELF  ROLE     TRUST         PHASE     LEADER  DECISION          PGINFO              READINESS  PROCESS  API
node-a  *     primary  full_quorum   primary   node-a  no_change         Primary tl=7        ready      idle     ok
node-b        replica  fail_safe     replica   node-a  enter_fail_safe   Replica tl=7        ready      idle     ok
node-c        unknown  unknown       unknown   ?       ?                 unavailable         ?          ?        down
```

JSON direction:

```json
{
  "cluster_name": "prod-eu1",
  "health": "healthy",
  "self_member_id": "node-a",
  "nodes": [
    {
      "member_id": "node-a",
      "is_self": true,
      "role": "primary",
      "trust": "full_quorum",
      "phase": "primary",
      "api": "ok"
    }
  ]
}
```

</description>

<acceptance_criteria>
- [x] Add a cluster-oriented `status` command under `src/cli/` and wire it through the operator binary entry point using the short CLI name `pgtm`.
- [x] `pgtm` with no subcommand behaves as `pgtm status`.
- [x] The default human output is compact, with the table as the main presentation and no cluttered front matter.
- [x] Switchover state is surfaced in status only when it is present and operationally relevant.
- [x] JSON status output includes the queried node identity explicitly, so it is always clear which node’s snapshot was used.
- [x] `pgtm status -v` provides deeper detail without requiring a separate debug-style noun for the common inspection path.
- [x] `pgtm status -v` uses debug data when available to surface pginfo and related local-node detail, and handles debug-disabled cases cleanly.
- [x] The command surfaces partial failures and disagreement explicitly, including unreachable nodes, leader mismatch, or multiple sampled primaries.
- [x] Only two output modes exist for now: default human output and `--json`, with stable JSON for automation and scripting.
- [x] `pgtm status --watch` is supported.
- [x] The command surface does not require an `ha` prefix or similarly redundant namespace for routine operator usage.
- [x] The implementation exposes or reuses a stable machine-readable cluster member payload so `pgtm` does not depend on unstable debug text dumps for the main status table.
- [x] The output design is documented in the CLI reference and used in the relevant how-to guides that currently loop over nodes with `curl`.
- [x] Tests cover healthy clusters, degraded trust, node disagreement, unreachable nodes, and multi-primary evidence.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Execution Plan

### Product decisions and command surface
- Make `status` the operator default entry point, not an advanced subcommand:
  - `pgtm` with no subcommand must behave as `pgtm status`.
  - `pgtm status` remains explicit and should produce the same result.
  - `status` gains the only new UX flags needed for this story: `-v, --verbose`, `--json`, and `--watch`.
- Remove the current generic `--output {json,text}` mode split for `status`. The task requirement is a human-first default plus `--json`, so execution should move the CLI to that model instead of keeping the older output switch around as a hidden third surface.
- Keep the command tree flat:
  - no `ha` namespace,
  - no `node` grouping,
  - no new `primary` or `replicas` nouns in this task.
- Treat `status -v` as the deeper operator path. Do not add another inspection command just to expose pginfo, process, or decision detail.
- Keep switchover commands intact, but update top-level parsing and help text so the mental model is now `pgtm[=status]` plus `pgtm switchover ...`.

### Core architecture for cluster-wide status
- Implement cluster-wide status as a fan-out from one seed API, not as a local shell loop and not as a debug-only scrape:
  - seed node comes from the existing resolved operator context (`--base-url` or config-backed `[pgtm].api_url` / derived target),
  - seed request returns stable cluster discovery data,
  - `pgtm status` then polls discovered peer APIs directly to build a synthesized cluster view.
- Prefer extending the existing stable `/ha/state` response with explicit cluster member discovery data instead of minting a brand-new `/ha/cluster` endpoint:
  - that keeps the stable read surface minimal,
  - preserves existing operator/test entry points that already treat `/ha/state` as the canonical observation API,
  - and avoids needless duplication between two stable endpoints that would otherwise expose overlapping HA state.
- If implementation proves `/ha/state` extension to be unworkable, only then fall back to a second stable endpoint, and document why the existing stable shape could not stay the single read surface.
- The stable discovery payload must include enough information for `pgtm` to find and name peers deterministically:
  - cluster name and scope,
  - queried node identity,
  - leader and switchover summary,
  - stable member list,
  - each member’s operator API target or advertised API URL,
  - member freshness / DCS-backed role data already known from the stable runtime state.

### Runtime and API model changes needed for discovery
- Extend the node’s stable cluster/member data so each member can advertise an operator-reachable API target.
- The current DCS member record only carries PostgreSQL connection details, so cluster-wide `pgtm status` cannot safely fan out today. Fix that explicitly rather than hiding the limitation.
- Reuse the existing `[pgtm].api_url` semantics as the node’s advertised operator API target unless code forces a stronger separation:
  - the repo already documents `[pgtm].api_url` as the operator-reachable API URL,
  - using that same value for DCS publication avoids introducing a second config knob for the same operational fact,
  - allow a safe fallback to the concrete `api.listen_addr` only when it is actually a usable client target,
  - reject or clearly degrade when a node cannot publish an operator-reachable target.
- Only add a brand-new runtime field if reusing `[pgtm].api_url` creates a concrete conflict during implementation or validation.
- Update DCS local-member publishing so every node writes its advertised API target together with its existing role/readiness/timeline data.
- Extend the stable API response types in `src/api/` to expose this cluster member payload directly and document it in the HTTP API reference.
- Keep the new stable payload intentionally boring and automation-friendly. It should be a direct source for:
  - cluster aggregation in `pgtm`,
  - future automation,
  - docs examples,
  - tests that assert cluster topology without parsing human output.

### CLI implementation breakdown
- Refactor `src/cli/args.rs` so `status` becomes a real structured command with arguments:
  - `verbose: bool`,
  - `json: bool`,
  - `watch: bool`,
  - optional watch interval if one already exists elsewhere and can be reused cleanly; otherwise keep the first version fixed/simple rather than inventing extra knobs.
- Implement the no-subcommand-to-status behavior in a way that does not make clap parsing brittle for `switchover`:
  - either represent status args at the root and normalize into `Command::Status`,
  - or pre-normalize argv before clap parsing,
  - but do not keep the current “subcommand is always required” structure.
- Introduce a dedicated cluster-status execution path in `src/cli/mod.rs` or a new `src/cli/status.rs` module that owns:
  - fetching stable discovery data from the seed node,
  - determining which peer APIs to contact,
  - polling peers concurrently,
  - optionally polling debug payloads for `-v`,
  - synthesizing a stable internal `ClusterStatusView`.
- Keep the HTTP client layer explicit:
  - add client methods for the new stable cluster endpoint,
  - add a client method for `/debug/verbose` if one does not already exist in a reusable form,
  - ensure every transport and decode failure is returned with node context instead of being swallowed.

### Synthesis and health rules
- Define a single internal aggregation model, for example `ClusterStatusView`, that is separate from raw API payloads. It should carry:
  - seed node identity,
  - cluster name / scope,
  - cluster health,
  - warnings / disagreement messages,
  - optional switchover summary,
  - ordered node rows for rendering,
  - optional verbose-only per-node details.
- Derive cluster health deterministically and document the rule in code comments and docs. At minimum treat these as degraded or worse:
  - one or more unreachable nodes,
  - leader mismatch across sampled nodes,
  - more than one sampled primary,
  - any sampled node reporting degraded trust,
  - missing advertised API targets for known members,
  - insufficient sampling to support a confident cluster view.
- Surface disagreement explicitly instead of collapsing to a “best guess”:
  - warnings for unreachable members,
  - warnings for leader disagreement,
  - warnings for multiple primaries,
  - warnings when DCS membership and API fan-out disagree.
- Prefer fail-closed summaries. If the CLI cannot prove a healthy cluster view, it should say so in the rendered output and JSON instead of silently omitting members.

### Human output design
- Replace the current newline-delimited `key=value` text renderer for `status` with a compact human-first renderer:
  - one short header line for cluster identity and health,
  - optional warning/switchover lines only when relevant,
  - a fixed-width table as the main body.
- Default table columns should stay compact and high-signal:
  - `NODE`,
  - `SELF`,
  - `ROLE`,
  - `TRUST`,
  - `PHASE`,
  - `API`.
- Verbose mode extends the same table rather than switching to a completely different document shape. Add columns only when `-v` is present, such as:
  - `LEADER`,
  - `DECISION`,
  - `PGINFO`,
  - `READINESS`,
  - `PROCESS`,
  - any debug-derived state that is reliably available.
- When debug data is disabled or unreachable in verbose mode:
  - keep the command successful if stable status data is still available,
  - render explicit placeholders or warnings instead of crashing or pretending the data exists.
- Keep row ordering stable and operator-friendly:
  - self row first if that materially helps scanning,
  - otherwise primary first, then replicas, then unknown/unreachable, with deterministic member-id tie-breaking.

### JSON output design
- `pgtm status --json` should emit the synthesized cluster view, not just the seed node’s `/ha/state`.
- Include the queried-via identity explicitly in JSON so operators and automation can always see which node seeded the cluster sample.
- JSON should include:
  - cluster identity,
  - sampled-at metadata if useful,
  - health,
  - warnings,
  - switchover state when present,
  - node rows with explicit `member_id`, `is_self`, `api_status`, and source-backed stable status fields,
  - verbose-only fields when `-v --json` is used.
- Keep JSON schema stable and machine-oriented. Avoid copying human-formatted strings like entire warning paragraphs into required data fields when structured fields can carry the same information.

### Watch mode
- Implement `status --watch` as repeated cluster aggregation, not as a wrapper around the previous single-node renderer.
- Human watch mode should redraw the full human status view each interval so operators can observe topology changes directly.
- JSON watch mode should emit a full JSON document per tick with a clean separator strategy. Keep it explicit and testable rather than inventing an ad hoc streaming format.
- Handle Ctrl-C cleanly and ensure watch mode does not leave partial terminal junk or suppressed errors.
- Reuse the same aggregation path for one-shot and watch mode so behavior stays identical apart from repetition.

### Test strategy
- Update parser tests in `src/cli/args.rs` for:
  - `pgtm` with no subcommand,
  - `pgtm -v`,
  - `pgtm --json`,
  - `pgtm --watch`,
  - `pgtm status ...`,
  - `pgtm switchover ...` still parsing correctly.
- Add unit tests for the new aggregation layer that cover:
  - healthy three-node cluster,
  - degraded trust on one node,
  - unreachable node,
  - leader disagreement,
  - multi-primary evidence,
  - missing peer API advertisement,
  - verbose debug-disabled behavior.
- Expand client tests in `src/cli/client.rs` for the new stable cluster endpoint and any debug verbose fetch helper.
- Expand renderer tests in `src/cli/output.rs` or the new status-rendering module for:
  - compact table output,
  - verbose table output,
  - warning lines appearing only when needed,
  - JSON schema shape including `queried_via`.
- Extend `tests/cli_binary.rs` with real binary coverage for:
  - bare `pgtm` defaulting to status,
  - `pgtm --json`,
  - human output header/table expectations,
  - config- or flag-seeded status against a mock multi-node cluster surface,
  - error mapping when the seed node is unreachable or when discovered peers fail.
- Use the existing HA multi-node harness for the task’s real cluster scenarios rather than inventing a parallel test framework:
  - add an end-to-end status workflow that exercises real cluster sampling,
  - cover healthy topology,
  - cover degraded / partial failure,
  - cover switchover visibility in status,
  - cover disagreement or split-brain evidence if the harness can force it without destabilizing unrelated tests.
- If adding the peer API advertisement field changes DCS member shape, extend DCS and API tests so that serialization/deserialization and API docs stay source-backed.

### Documentation work
- The requested `update-docs` skill is not available in this session, so execution should update docs directly in-repo.
- Update `docs/src/reference/pgtm-cli.md` to reflect the new operator model:
  - synopsis should show bare `pgtm`,
  - document `status`, `status -v`, `status --json`, and `status --watch`,
  - remove stale `--output text/json` wording,
  - document default human table output and the no-subcommand behavior.
- Update `docs/src/reference/http-api.md` to document the new stable cluster payload and any new member fields or endpoint.
- Rewrite operator how-to guides that currently tell users to loop over nodes manually:
  - `docs/src/how-to/check-cluster-health.md`,
  - `docs/src/how-to/perform-switchover.md`,
  - any tutorial/reference pages that still describe `status` as single-node key/value output.
- Remove stale prose and examples that present `pgtm` as just a thin node-local wrapper once cluster-wide status exists.
- Rebuild generated docs instead of hand-editing `docs/book`.

### Verification and execution order
- Execute code changes in this order so the refactor stays coherent:
  1. Add runtime/member/API discovery support for peer API targets and stable cluster payloads.
  2. Add CLI client methods and aggregation model.
  3. Refactor argument parsing for default-status, `--json`, `-v`, and `--watch`.
  4. Replace the old status text renderer with the new table renderer and JSON schema.
  5. Add/adjust unit, binary, and HA integration tests.
  6. Update docs and regenerate built docs.
- After implementation, verify there are no stale references to:
  - `--output text`,
  - `status` key/value text shape,
  - manual multi-node `curl` loops where `pgtm status` should now be the primary path.
- Required gates for task completion:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- Only after all required checks and docs updates pass:
  - tick the truly completed acceptance boxes,
  - set `<passes>true</passes>`,
  - run `/bin/bash .ralph/task_switch.sh`,
  - commit all tracked changes including `.ralph`,
  - push,
  - stop immediately.

NOW EXECUTE
