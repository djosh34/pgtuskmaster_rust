---
## Bug: Harden make gates against hangs and silent passes <status>done</status> <passes>true</passes>

<description>
`make test`, `make test-long`, `make lint`, and `make check` currently have uneven timeout behavior and incomplete pass assertions.

Observed issues from audit:
- `make test-long` has no timeout wrapper around `cargo test` executions, so one stalled real-binary test can block forever.
- `make test` has a timeout only around the final `cargo test` run, but not around preflight `cargo test -- --list`.
- `make lint` and `make check` have no timeout bounds for docs scripts / `cargo clippy` / `cargo check`.
- docs no-code guard scans only selected docs subtrees and only fences that begin with ` ``` ` at column 1, which can miss forbidden code blocks and create false confidence.
- Gate evidence logs are mostly raw stdout without normalized per-step start/end timestamps, exit codes, durations, or timeout forensics.

Please explore and research the codebase first, then implement a robust, fail-closed fix set.
</description>

<acceptance_criteria>
- [x] `make test` bounds both preflight and execution phases with explicit timeouts, and fails with clear diagnostics on timeout.
- [x] `make test-long` bounds each preflight and per-test execution with explicit timeouts; a stuck ultra-long test cannot hang forever.
- [x] `make lint` and `make check` run under bounded timeouts (or equivalent watchdog) with deterministic non-zero failure on timeout.
- [x] docs architecture no-code guard covers intended docs roots and fence patterns without easy bypasses (leading whitespace / moved docs directories).
- [x] Gate scripts/targets emit structured evidence for each step: command, start/end UTC, duration, exit status, and timeout marker when applicable.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<implementation_plan>
## Execution Plan (Verified Draft 2)

### Phase 0 — Scope confirmation (read-only)
- [x] Re-read current gate definitions in `Makefile` (`check`, `test`, `test-long`, `lint`, `docs-lint`) and confirm which phases are unbounded today.
- [x] Re-read current docs guard at `tools/docs-architecture-no-code-guard.sh` and confirm bypass vectors:
  - hardcoded roots miss large parts of `docs/src/`
  - fence regex only matches column-1 ``` fences
  - info string parsing treats `lang extra=...` as “language”
- [x] Confirm current ultra-long test split semantics:
  - `ULTRA_LONG_TESTS` list is canonical and validated against `cargo test -- --list`
  - `make test` skips ultra-long tests in one invocation
  - `make test-long` runs each ultra-long test with `-- --exact` (but is currently unbounded)

### Phase 1 — Add a reusable timeout + evidence step runner (new tool)
- [x] Add a new script `tools/gate-step.sh` (or `tools/gate-evidence.sh`) with the following contract:
  - Inputs (flags): `--gate <name>`, `--step <name>`, `--run-id <id>`, `--evidence-dir <path>`, `--timeout-bin <path>`, `--timeout-secs <n>`, `--kill-after-secs <n>`, `--` command argv.
  - Behavior:
    - Creates evidence dirs: `<evidence-dir>/<gate>/steps/`.
    - Writes one per-step log file: `<evidence-dir>/<gate>/steps/<NN>-<step>.log` (captures combined stdout/stderr) while still streaming output to the console.
    - Appends a structured record to `<evidence-dir>/<gate>/steps.jsonl` for each step with:
      - `gate`, `step`, `argv` (JSON array), `start_utc`, `end_utc`, `duration_ms`, `exit_code`, `timed_out`, `timeout_secs`, `kill_after_secs`, `log_path`.
    - Classifies timeout exit codes (`timeout` commonly returns `124`; killed-after may return `137`) as `timed_out=true` and prints a clear timeout diagnostic to stderr.
    - Fails closed: any non-zero exit is propagated (and recorded), including failures to create/write evidence files.
  - Implementation constraints:
    - No `jq` dependency (pure bash + coreutils).
    - Avoid `set -e` footguns around pipelines: preserve and propagate the real command exit code (`PIPESTATUS`) and ensure `pipefail` is enabled.
   - Reliability hardening:
     - Run-id collisions: `--run-id` must include a collision-resistant suffix (at least milliseconds + PID, or similar), not just second-resolution UTC.
     - Step numbering: derive `<NN>` from the current number of records in `steps.jsonl` for the gate (sequential within a gate run).

### Phase 2 — Makefile: add explicit timeouts and step evidence for every gate phase
- [x] Make shell semantics explicit (the current file already uses bash-isms like `pipefail` and `[[ ... ]]`):
  - [x] Set `SHELL := /usr/bin/env bash` near the top of `Makefile`.
- [x] In `Makefile`, make all gate targets depend on `ensure-timeout`:
  - [x] `check: guard-makeflags ensure-timeout`
  - [x] `docs-lint: guard-makeflags ensure-timeout ensure-node`
  - [x] `lint: guard-makeflags ensure-timeout docs-lint`
  - [x] `test-long: guard-makeflags ensure-timeout`
- [x] Introduce explicit per-phase timeout constants (do not allow external override, matching existing gate policy style):
  - [x] `TEST_PREFLIGHT_TIMEOUT_SECS` (for `cargo test -- --list` in both `test` and `test-long`)
  - [x] `TEST_LONG_PER_TEST_TIMEOUT_SECS` (per ultra-long test execution)
  - [x] `CHECK_TIMEOUT_SECS`
  - [x] `LINT_DOCS_TIMEOUT_SECS` (node + docs guard)
  - [x] `LINT_CLIPPY_TIMEOUT_SECS` (per clippy invocation)
  - [x] `*_KILL_AFTER_SECS` variants as needed (short vs long phases)
- [x] Add run metadata variables for evidence output:
  - [x] `GATE_RUN_ID := $(shell date -u +%Y%m%dT%H%M%SZ)-$(shell printf '%s' "$$PPID")-$(shell printf '%s' "$$RANDOM")`
  - [x] `GATE_EVIDENCE_DIR := $(CURDIR)/.ralph/evidence/gates/$(GATE_RUN_ID)`
- [x] Rewire each gate into explicit steps executed via the step runner:
  - **`make check`**
    - [x] step `check.cargo_check`: `cargo check --all-targets` under `CHECK_TIMEOUT_SECS`.
  - **`make test`**
    - [x] step `test.preflight_list`: `cargo test --all-targets -- --list` under `TEST_PREFLIGHT_TIMEOUT_SECS`.
    - [x] step `test.preflight_validate_ultra_long`: validate `ULTRA_LONG_TESTS` exists, is unique, and each token matches exactly one listed test (keep existing logic but run it as its own step so it’s evidence-backed).
    - [x] step `test.preflight_validate_default_nonempty`: compute `non_ultra_count = (all_test_names from list) - ULTRA_LONG_TESTS` and fail closed if `non_ultra_count == 0` (prevents “default suite is empty but green” drift).
      - Use only `: test` lines from `-- --list` output (ignore `Doc-tests ...` banners) to avoid false counts.
    - [x] step `test.exec_default_suite`: existing `cargo test --all-targets -- $(ULTRA_LONG_SKIP_ARGS)` under `TEST_TIMEOUT_SECS` (keep existing timeout for the execution step).
  - **`make test-long`**
    - [x] step `test_long.preflight_list`: `cargo test --all-targets -- --list` under `TEST_PREFLIGHT_TIMEOUT_SECS`.
    - [x] step `test_long.preflight_validate_ultra_long`: keep existing duplicate/exists/uniqueness checks (evidence-backed).
    - [x] steps `test_long.exec.<test_name>`: for each `t in $(ULTRA_LONG_TESTS)` run:
      - `cargo test --all-targets "$$t" -- --exact`
      - under `TEST_LONG_PER_TEST_TIMEOUT_SECS` so a single hung test can’t stall forever.
  - **`make docs-lint`**
    - [x] step `docs_lint.mermaid`: `node ./tools/docs-mermaid-lint.mjs` under `LINT_DOCS_TIMEOUT_SECS`.
    - [x] step `docs_lint.no_code_guard`: `./tools/docs-architecture-no-code-guard.sh` under `LINT_DOCS_TIMEOUT_SECS`.
  - **`make lint`**
    - [x] steps `lint.clippy.<pass>`: each of the 4 clippy invocations under `LINT_CLIPPY_TIMEOUT_SECS`.
- [x] Ensure evidence directory creation does not break local workflows:
  - Evidence must be written under `.ralph/evidence/` (already gitignored by default).
  - The Makefile should still stream normal command output; step runner adds structured sidecar evidence.

### Phase 3 — Docs architecture “no-code” guard: cover moved docs and fence bypasses
- [x] Update `tools/docs-architecture-no-code-guard.sh` to determine docs source root robustly:
  - [x] Parse `docs/book.toml` for `[book] src = "..."` (default to `docs/src` if absent).
  - [x] Scan `*.md` recursively under the resolved src root (not hardcoded subtrees).
  - [x] Fail closed if the scan file list is empty.
- [x] Harden policy + parsing (eliminate practical bypasses without blocking intended contributor snippets):
  - [x] Make allowed languages depend on file path:
    - Default (non-contributor docs): allow only `mermaid|bash|console|toml|text`.
    - Contributor docs (`docs/src/contributors/**`): additionally allow `rust` (per the contributor “code snippet policy”).
  - [x] Detect fences with markdown-legal prefixes:
    - allow leading whitespace (indent) and blockquote prefix (`>`), and support both backticks and tildes.
    - track marker type and length (``` vs ````; ~~~ vs ~~~~) and only close on a compatible closing fence.
  - [x] Parse language as the *first token only* of the info string (so ` ```bash title="x"` is treated as `bash`).
  - [x] Require explicit language on opening fence; unlabeled fences remain a hard error.
  - [x] Flag HTML code blocks (`<pre><code ...>`) as violations (at least in non-contributor docs), since they bypass markdown fence detection.
- [x] Keep diagnostics stable:
  - [x] Deterministic file ordering (`LC_ALL=C sort`).
  - [x] Error messages include `file:line` and the offending language token.

### Phase 4 — Verification (must be 100% green)
- [x] Run the full required gate set and ensure they all pass:
  - [x] `make check`
  - [x] `make test`
  - [x] `make test-long`
  - [x] `make lint`
- [x] Confirm timeout behavior is evidence-backed:
  - The structured evidence exists at `.ralph/evidence/gates/<run-id>/.../steps.jsonl`.
  - Each gate phase produces a start/end record with duration and exit code.

### Phase 5 — Task wrap-up
- [x] Tick off acceptance criteria checkboxes based on green runs.
- [x] Keep `<passes>false</passes>` unless all four gates are green.
- [x] Only after all gates pass: set `<status>done</status> <passes>true</passes>` and follow the repo’s task-switch + commit + push protocol.
</implementation_plan>

NOW EXECUTE
