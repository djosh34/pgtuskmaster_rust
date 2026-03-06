## Task: Add Structured File Sink Support (Backlog) <status>done</status> <passes>true</passes>

<description>
**Goal:** Extend the unified logging subsystem with optional structured file sink support after the base structured-ingestion task is complete.

**Scope:**
- Implement additional sink modes in the single existing logging config/setup path:
  - structured file output sink(s)
- Keep default behavior unchanged (`stderr` JSONL remains default).
- Ensure sink selection/routing is fully config-driven and composable.

**Context from research:**
- This is intentionally deferred from the base unified logging task:
  - `.ralph/tasks/story-rust-system-harness/38-task-unified-structured-logging-and-postgres-binary-ingestion.md`

**Expected outcome:**
- Operators can route unified structured logs to file without introducing a second logging bootstrap path.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] File sink support exists and is configurable through the single logging config entrypoint
- [x] Default sink remains JSONL to `stderr`
- [x] Existing structured log schema and source attribution keys remain stable
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] BDD features pass (covered by `make test`).
</acceptance_criteria>

## Plan (Draft)

### Deep skeptical reality check (done; no re-implementation needed)
- ✅ This task’s core scope (structured JSONL file sink + config-driven wiring) is **already implemented**:
  - `src/logging/mod.rs`:
    - `JsonlFileSink` exists (append/truncate + parent dir creation).
    - `logging::bootstrap(&RuntimeConfig)` wires `logging.sinks.file` and composes sinks (stderr/file) via fanout.
    - Unit tests already cover `JsonlFileSink` + `FanoutSink`.
  - `src/config/schema.rs`: `LoggingSinksConfig { stderr, file }` + `FileSinkConfig { enabled, path, mode }` exist.
  - `src/config/defaults.rs`: defaults keep behavior unchanged (`stderr.enabled=true`, `file.enabled=false`, `mode=append`).
  - `src/config/parser.rs`: validates non-empty `logging.sinks.file.path` and requires `path` when `file.enabled=true` (has tests).
  - `src/runtime/node.rs`: calls `crate::logging::bootstrap(&cfg)` exactly once (single entrypoint).

Conclusion: this “backlog” task was likely created before the implementation landed; remaining work is (1) add missing bootstrap-level tests for sink selection/wiring (small + low risk), (2) run full verification gates, then (3) close the task with evidence.

### Execution plan (no more exploration; do exactly in order)

#### 1) Add bootstrap-level tests (fills current coverage gap)
- [ ] Add unit tests in `src/logging/mod.rs` that exercise `bootstrap(...)` wiring outcomes directly (today only sink-level + parser-level tests exist).
  - [ ] `bootstrap_file_enabled_without_path_returns_misconfigured`
  - [ ] `bootstrap_file_enabled_with_path_writes_jsonl`
  - [ ] `bootstrap_with_stderr_and_file_still_writes_file` (verifies fanout path without asserting stderr)
  - [ ] (Optional) `bootstrap_with_all_sinks_disabled_is_non_fatal` (documents NullSink behavior)

Notes for these tests:
- Keep JSON schema stable by asserting parsed JSON fields (e.g. `message`, `severity_text`) instead of raw-string equality.
- Avoid `unwrap`/`expect`/`panic` in tests too (repo policy); prefer `Result<(), Box<dyn Error>>` and `matches!`.
- Use the existing temp-path pattern (pid + timestamp) and best-effort cleanup (`let _ = remove_dir_all(...)`).

### Verification (mandatory; no skips)
- [ ] Run `make check`
- [ ] Run `make test`
- [ ] Run `make test-long`
- [ ] Run `make lint`

### Task closure (only after all verification passes)
- [ ] Update this task file:
  - [ ] Set `<status>done</status>` and `<passes>true</passes>`
  - [ ] Add `<passes>true</passes>` (required by task runner conventions in this repo)
  - [ ] Tick all acceptance criteria checkboxes
  - [ ] Record brief evidence of the `make ...` commands that pass (copy/paste the final summary lines)
- [ ] Run `/bin/bash .ralph/task_switch.sh`
- [ ] `git add -A`
- [ ] `git commit -m "task finished 39-task-logging-file-sink-backlog: structured JSONL file sink support"`
- [ ] `git push`
- [ ] Add any learnings/surprises to `AGENTS.md`

NOW EXECUTE

<execution_report>
- Code change: added bootstrap-level unit tests that exercise `logging::bootstrap(...)` sink selection/wiring directly (`src/logging/mod.rs`).
- `make check`: pass
  - `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 5.97s`
- `make test`: pass
  - `test result: ok. 234 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 63.06s`
- `make test-long`: pass
  - `test ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix ... ok`
  - `test ha::e2e_multi_node::e2e_multi_node_stress_planned_switchover_concurrent_sql ... ok`
  - `test ha::e2e_multi_node::e2e_multi_node_stress_unassisted_failover_concurrent_sql ... ok`
  - `test ha::e2e_multi_node::e2e_no_quorum_fencing_blocks_post_cutoff_commits_and_preserves_integrity ... ok`
- `make lint`: pass
</execution_report>
