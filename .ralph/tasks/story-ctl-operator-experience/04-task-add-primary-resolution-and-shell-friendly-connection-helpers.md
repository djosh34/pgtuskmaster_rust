## Task: Add Primary Resolution And Shell-Friendly Connection Helpers To `pgtm` <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
**Goal:** Add carefully designed shell-oriented commands to `pgtm` that let operators resolve the current primary or replica endpoints and feed that information into `psql`, scripts, or automation without scraping human output. The higher-order goal is to support fast operator workflows like “connect me to the current primary” while still keeping the CLI output principled and parseable.

**Scope:**
- Design commands or subcommands that expose the current primary member, replica members, and connection details in a script-friendly way.
- Prefer minimal verbs and nouns such as `pgtm primary`, `pgtm replicas`, or `pgtm connect-info` over deep command nesting.
- Make DSN output the operator-first contract for now.
- Ensure any shell-friendly output has a dedicated stable mode instead of encouraging users to scrape the human table/status view.
- Reuse DCS member record data for PostgreSQL host and port instead of requiring duplicated node connection info in config.
- Keep the default DSN concise, then add an explicit `--tls` flag for TLS-expanded DSNs rather than always printing TLS connection parameters.
- Document intended use with `psql` and shell pipelines, but avoid inventing ambiguous magic that hides errors or silently picks unsafe targets.

**Context from research:**
- The user explicitly raised the idea of piping the current primary into `psql`; this is directionally useful, but only if the output contract is deliberate.
- Patroni exposes DSN/query-oriented operator helpers, which is one reason `patronictl` is more useful than plain state inspection.
- DCS member records already carry PostgreSQL host and port, which makes DSN rendering possible without extra node inventory in config.
- `pgtm` must not become a PostgreSQL client in this task. It only resolves and prints DSNs.
- Today the CLI exposes only HA API state and does not help operators bridge from member identity to a live PostgreSQL connection target.
- This should be designed after or together with config-backed contexts and cluster-wide status, because connection helpers become much more useful once the CLI knows the cluster inventory.
- Current local design direction is to keep the command tree shallow and avoid `ha`, `cluster`, or other redundant prefixes for common operator commands.
- The current design direction is to keep output modes simple here too: default human output and `--json`.
- The current design direction is to support an explicit `--tls` flag that expands DSNs with TLS settings using config-backed paths where that is safe and well-defined.
- The TLS-expanded DSN path should be driven by semantic PostgreSQL client TLS config, not by print-only field names.
- `--tls` should default to `sslmode=verify-full` for v1 instead of introducing a separate sslmode knob unless a real use case proves it necessary.

**Expected outcome:**
- Operators have a supported, scriptable way to discover the current primary or replicas without fragile text scraping.
- The CLI can support practical workflows such as targeted SQL verification, connection-string export, and incident triage.
- The product gains a concrete differentiator beyond “curl wrapper with prettier text.”

Expected command/output direction:

```bash
pgtm -c config.toml primary
pgtm -c config.toml primary --json
pgtm -c config.toml primary --tls

pgtm -c config.toml replicas
pgtm -c config.toml replicas --json
pgtm -c config.toml replicas --tls
```

Expected default output:

```text
host=node-a.db.example.com port=5432 user=postgres dbname=postgres
```

Expected `replicas` default output:

```text
host=node-b.db.example.com port=5432 user=postgres dbname=postgres
host=node-c.db.example.com port=5432 user=postgres dbname=postgres
```

Expected `--tls` output direction:

```text
host=node-a.db.example.com port=5432 user=postgres dbname=postgres sslmode=verify-full sslrootcert=/etc/pgtm/postgres-ca.pem sslcert=/etc/pgtm/postgres.crt sslkey=/run/secrets/postgres.key
```

TLS field source rules:
- Use `[pgtm.postgres_client]` when present.
- Otherwise fall back to `[pgtm.api_client]`.
- `pgtm` prints DSNs only; it does not open PostgreSQL connections.

</description>

<acceptance_criteria>
- [x] Define and implement a stable DSN-oriented contract for `pgtm primary` and `pgtm replicas`.
- [x] The chosen command names stay minimal and flat under `pgtm`, without requiring users to traverse redundant namespaces.
- [x] The implementation distinguishes clearly between human output and machine/script output so automation does not depend on table formatting.
- [x] The command output contract works with only default human output and `--json`; it does not introduce an open-ended output-format matrix yet.
- [x] `pgtm primary` renders a DSN for the current primary in human output.
- [x] `pgtm replicas` renders one replica DSN per line in human output.
- [x] The DSN rendering uses cluster state rather than duplicated node definitions in config.
- [x] The default DSN output stays concise and does not always inject optional TLS fields.
- [x] `pgtm primary --tls` and `pgtm replicas --tls` are supported.
- [x] `--tls` uses `sslmode=verify-full` for v1 unless implementation evidence forces a more configurable design.
- [x] `pgtm primary --tls` is explicit about which PostgreSQL client TLS fields can be emitted, such as `sslmode`, `sslrootcert`, `sslcert`, and `sslkey`, and it does not reuse unrelated API TLS server fields.
- [x] If `[pgtm.postgres_client]` is absent, TLS-expanded DSN printing falls back to `[pgtm.api_client]`.
- [x] The task does not add internal PostgreSQL connection support to `pgtm`.
- [x] Error handling is explicit for no leader, disagreement between nodes, incomplete inventory, or unavailable connection metadata.
- [x] Docs include at least one supported `psql` or shell-pipeline example that uses the new contract without text scraping.
- [x] Tests cover healthy primary resolution, no-leader cases, disagreement cases, and output-shape stability.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Execution Plan

### Product decisions and command contract
- Keep the command tree flat and operator-first:
  - `pgtm primary`
  - `pgtm replicas`
  - both support only the existing output split of default human output and `--json`
  - both support an explicit `--tls` flag that expands the DSN contract instead of changing the default output
- Do not add a separate `connect-info` or nested `cluster primary` surface in this task unless implementation proves `primary` and `replicas` cannot express the accepted contract cleanly.
- Treat the human default as the stable shell-oriented contract for this feature, not as decorative text:
  - `primary` prints exactly one DSN line
  - `replicas` prints one DSN per replica, one per line
  - no headers, prefixes, commentary, or table formatting in the default mode
- Keep `--json` machine-oriented and explicit. It should expose the same resolved members and DSN strings without forcing automation to parse the text lines.
- Do not turn `pgtm` into a PostgreSQL client. This task only resolves cluster members and prints DSNs.

### Extract a reusable sampled-cluster snapshot before the status renderer
- Reuse the existing cluster-wide sampling flow, but do not make connection resolution depend on the rendered `ClusterStatusView`.
- First extract the peer fan-out and aggregation work in [`src/cli/status.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/status.rs) into a reusable sampled-cluster snapshot model that both `status` and the new connection helpers consume.
- The execution path should remain:
  - seed one API node through the existing resolved operator context,
  - sample discovered peer APIs concurrently,
  - synthesize one internal sampled snapshot with both discovered member records and successful peer observations,
  - project that snapshot into either the status view or the connection-helper result.
- Keep the refactor small and local to the CLI layer, but make the intermediate snapshot explicit so connection helpers can reason about contradictions, missing observations, and DCS member connection metadata without reverse-engineering status-table concerns.

### Add explicit connection-target data structures
- Introduce a dedicated connection-helper model under the CLI surface, either in a new module such as [`src/cli/connect.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/connect.rs) or in a narrow extension of [`src/cli/status.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/status.rs).
- Model the resolved output explicitly instead of passing raw strings around. The internal types should distinguish:
  - the chosen target kind (`primary` vs `replica`)
  - the resolved member identity
  - PostgreSQL host and port from DCS member records
  - the rendered DSN string
  - whether TLS expansion was requested
- For `--json`, expose structured member identity and connection fields in addition to the final DSN string so the JSON remains useful if later automation wants fields without reparsing the DSN.

### CLI parsing changes
- Extend [`src/cli/args.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/args.rs) with shallow subcommands for `primary` and `replicas`.
- Add a shared args struct for connection helpers with:
  - `tls: bool`
- Keep `--json` global as it is today so the new commands fit the existing CLI shape cleanly.
- Keep `--watch` and `--verbose` restricted to `status`. Execution should explicitly reject `pgtm primary --watch`, `pgtm replicas --watch`, and verbose-only combinations with a config error instead of silently ignoring them.
- Update parser tests in [`src/cli/args.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/args.rs) for:
  - `pgtm primary`
  - `pgtm primary --tls`
  - `pgtm primary --json`
  - `pgtm replicas`
  - `pgtm replicas --tls`
  - invalid use of `--watch` or `-v` with the new commands

### Execution flow in the CLI entrypoint
- Extend [`src/cli/mod.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/mod.rs) so the new commands resolve the operator context once, then branch into connection-helper execution.
- Reuse the same auth and API client plumbing already built for `status` and `switchover`.
- Keep command-specific validation explicit:
  - `status` remains the only command that accepts `--watch` and `--verbose`
  - `primary` and `replicas` accept `--tls` and optionally `--json`
- Avoid late surprises in the renderer. Resolution errors such as no primary, ambiguous primary, or missing connection metadata should be raised before output rendering.

### Primary and replica resolution rules
- Resolve targets from sampled cluster state, not from a single node’s local opinion and not from static config inventory.
- Use the sampled cluster view to find candidate members:
  - primary candidates are members whose sampled role is `primary`
  - replica candidates are members whose sampled role is `replica`
- Do not collapse every degraded warning into one generic hard failure. The resolution rules need command-specific safety semantics based on the sampled snapshot.
- For `primary`, stay strict and fail closed:
  - if zero primaries are observed, return an explicit error
  - if more than one primary is observed, return an explicit disagreement error
  - if any discovered member could not be sampled or sampled members disagree on leader or membership, return an explicit degraded-resolution error rather than guessing a write target
- For `replicas`, return all sampled replicas in a stable deterministic order, ideally the same ordering already used by the cluster view after self/role sorting.
- Decide explicitly how degraded states affect `replicas`:
  - unreachable or unsampled members should not be emitted as connection targets
  - only members with sampled usable PostgreSQL host and port data are eligible
  - degraded sampling should not automatically fail `replicas` if at least one positively sampled replica remains, but the JSON response should carry enough metadata or warnings for automation to detect partial visibility
- Reuse contradiction signals from shared aggregation where possible, but introduce dedicated connection-resolution errors only where operator intent truly requires a single authoritative answer.

### DSN rendering contract
- Render libpq-style keyword/value DSNs, not URLs.
- The default DSN should stay concise:
  - `host=<postgres_host> port=<postgres_port> user=postgres dbname=postgres`
- The source of `host` and `port` is the DCS member record already exposed in stable API member payloads.
- For v1, keep `user` and `dbname` fixed to `postgres` unless the existing config model already provides a better explicit operator-facing default during execution review.
- Do not include optional TLS fields unless `--tls` is set.
- Keep DSN field ordering stable so shell diffs and tests remain simple.
- Implement proper shell-safe quoting for values that would break keyword/value DSNs if left raw. If the renderer currently assumes simple host/path strings, execution should add a dedicated libpq keyword escaping helper instead of concatenating blindly.

### TLS-expanded DSN behavior
- Drive TLS DSN expansion from the already-resolved PostgreSQL client TLS context in [`src/cli/config.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/config.rs).
- Keep the precedence already established in the config layer:
  - `[pgtm.postgres_client]` first
  - fallback to `[pgtm.api_client]` when the PostgreSQL client block is absent
- For `--tls`, emit only the PostgreSQL client TLS fields that make sense in libpq keyword/value DSNs:
  - `sslmode=verify-full`
  - `sslrootcert=<path>` when a CA cert path exists
  - `sslcert=<path>` when a client cert path exists
  - `sslkey=<path>` when a client key path exists
- Do not emit inline certificate contents or env-resolved secret values into DSNs. DSN output must only include filesystem paths when the configured material came from a path-backed source.
- If `--tls` is requested but the effective TLS client material was configured inline or from env in a way that cannot be represented as a safe DSN path, fail explicitly instead of printing a misleading partial DSN.
- Do not reuse unrelated API server TLS fields. Only the semantic client TLS settings from the `pgtm` client blocks are valid for DSN expansion.

### Config-surface follow-through needed for `--tls`
- The current resolved operator context in [`src/cli/config.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/config.rs) materializes certificate bytes for HTTP use, but DSN printing needs path-aware provenance.
- Refactor that layer so the PostgreSQL client TLS context retains enough source information to render DSN flags safely:
  - whether each field came from `[pgtm.postgres_client]` or `[pgtm.api_client]`
  - whether the source was a path-backed input versus inline/env-backed content
  - the usable path when path-backed
- Keep the HTTP client path working as it does today. The refactor should add provenance for DSN printing, not regress API client setup.
- Add focused tests around this provenance-preserving resolution so later tasks do not reopen the same ambiguity.

### Output rendering
- Extend [`src/cli/output.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/output.rs) with dedicated renderers for:
  - single DSN text output
  - multi-DSN text output
  - JSON output for the connection-helper response types
- Keep renderers small and deterministic:
  - `primary` text output should end with one newline from the binary wrapper only
  - `replicas` text output should join DSNs with `\n` and no extra blank lines
- Do not overload the status renderer with connection-helper special cases. Add separate rendering entry points so the contracts remain isolated and testable.

### Error handling expectations
- Add explicit error variants in [`src/cli/error.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/error.rs) or a narrow helper-layer error that maps into it for:
  - no sampled primary
  - multiple sampled primaries
  - leader disagreement / insufficient sampling for connection resolution
  - missing PostgreSQL host or port in member data
  - requested `--tls` DSN fields that cannot be represented safely from config provenance
- Error messages should tell operators what failed and why, for example whether the cluster view was degraded, whether multiple primaries were observed, or whether TLS material came from inline content that cannot be emitted as `sslrootcert` or `sslkey`.
- Do not silently omit broken TLS fields or silently pick one primary from a disputed cluster view.

### Test strategy
- Add unit tests near the new resolution code for:
  - healthy single primary resolution
  - healthy multi-replica resolution
  - no-primary error
  - multi-primary error
  - strict `primary` failure when sampling is incomplete or contradictory
  - degraded-but-usable `replicas` output when one replica is reachable and another is unsampled
  - missing PostgreSQL host/port metadata
  - TLS provenance cases for path-backed, inline, env-backed, and fallback-to-api-client configuration
- Extend renderer tests in [`src/cli/output.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/cli/output.rs) for:
  - exact single-line `primary` output
  - exact multi-line `replicas` output
  - JSON shape stability
  - keyword/value escaping when fields contain characters that require quoting
- Extend spawned-binary coverage in [`tests/cli_binary.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/tests/cli_binary.rs) for:
  - `pgtm primary`
  - `pgtm primary --json`
  - `pgtm primary --tls`
  - `pgtm replicas`
  - `pgtm replicas --json`
  - `pgtm replicas --tls`
  - degraded and disagreement failures via mocked peer responses
- Use the existing HA multi-node harness for at least one real-cluster workflow that proves the commands resolve the active primary and replicas from live member state rather than from mock-only assumptions.
- Keep real-binary tests mandatory. Do not skip them, and do not weaken existing cluster-status coverage while refactoring shared aggregation code.

### Documentation work
- The requested `update-docs` skill is not available in this session, so execution should update docs directly in-repo.
- Update [`docs/src/reference/pgtm-cli.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/reference/pgtm-cli.md) to document:
  - the new `primary` and `replicas` commands
  - default DSN output
  - `--json`
  - `--tls`
  - operator examples piping the result into `psql` without scraping `status`
- Update [`docs/src/reference/runtime-configuration.md`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/src/reference/runtime-configuration.md) to clarify the PostgreSQL client TLS fallback rules and the DSN-printing constraint that only path-backed TLS files can be emitted into DSNs.
- Update operator how-to pages that currently imply manual endpoint lookup or raw curl is required for “connect to the current primary” workflows, especially the cluster health and switchover guides.
- Rebuild generated docs instead of editing [`docs/book`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/docs/book) by hand.

### Execution order
- Execute the task in this order so the later command work sits on a stable base:
  1. Refactor shared cluster aggregation so connection helpers can reuse it.
  2. Extend CLI arg parsing and command dispatch for `primary` and `replicas`.
  3. Add the connection-resolution model and DSN renderer.
  4. Refactor CLI config/TLS resolution to preserve path provenance for `--tls`.
  5. Add and adjust unit tests, binary tests, and at least one HA integration path.
  6. Update docs and regenerate the built book.
  7. Run the full required verification gates.

### Completion gates
- After implementation, run and require all of:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- Only after those all pass and the docs are updated:
  - tick the completed acceptance boxes in this task file
  - set `<passes>true</passes>`
  - run `/bin/bash .ralph/task_switch.sh`
  - commit all changes including `.ralph` updates with the required `task finished [task name]: ...` message format
  - push with `git push`
  - stop immediately

NOW EXECUTE
