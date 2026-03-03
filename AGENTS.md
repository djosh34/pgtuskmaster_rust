## Progress Log

On startup, read your previous progress
```bash
/bin/bash .ralph/progress_read.sh "<codex>"
```

Append to the progress log — it is your working memory across context windows.
Please write to it as if it is your diary. All updates, confusions, thinking, progress, anything you personally want to write.
```bash
/bin/bash .ralph/progress_append.sh "<codex>" << 'EOF_APPEND_PROGRESS_LOG'
- what you did
- what happened
- should do next, after quitting immediately due to context limit
EOF_APPEND_PROGRESS_LOG
```

Please quit immediately if you feel you are filling up your own context too much.

Please do not use unwraps, panics, expects anywhere. The linter gives out error with good reason.
All errors must be properly handled. Please add-bug with that skill, if you find any, and remove any linter-ignore/exceptions for it.

No test must be optional, especially not tests against real binaries. Instead install them if needed.
Skipping tests is one of the worst things you can do, giving extremely false confidence. Create bug immediately when spotted with add-bug skill.


## Cross application applicable learnings
- Config defaulting is safer when required fields stay required in parse structs; optional-only defaults plus strict `deny_unknown_fields` avoids silent typo drift.
- Keep `target/` ignored in Rust repos to avoid accidental large artifact commits during task-level `git add -A` workflows.
- For contract-only skeleton tasks, avoid `pub(crate) use ...` re-export fanout in `mod.rs`; direct module paths keep clippy `unused_imports` clean while preserving minimal visibility.
- Avoid running multiple top-level Cargo build/test commands in parallel within the same workspace; package-cache lock contention can surface misleading archive/object-file errors.
- For port-allocation tests, keep listener reservations alive for the whole assertion window; dropping reservations early makes legitimate OS port reuse look like a false collision.
- Parallel `make` targets that each invoke Cargo can intermittently fail at link time with missing `*.rcgu.o` artifacts; rerun required gates sequentially for trustworthy pass/fail evidence.
- When state caches use `BTreeMap<MemberId, ...>`, ensure `MemberId` derives `Ord`/`PartialOrd`; otherwise key operations fail at compile-time deep inside worker logic.
- Clippy `large_enum_variant` is likely for watcher update enums that carry full runtime config payloads; boxing only the heavy variant preserves API shape while satisfying `-D warnings`.
- To keep strict runtime clippy denies active under `--all-features`, prefer crate-root `cfg_attr(not(test), deny(...))` and isolate panic/expect allowances in `src/test_harness/mod.rs` instead of feature-gating deny policy off globally.
- For real `pg_ctl` process-worker tests, keep `NamespaceGuard` alive for the entire job lifecycle; dropping it early can race cleanup against running postgres and cause non-deterministic follow-up job outcomes.
- When using `PortReservation` to pick ports for child processes, ensure `drop(reservation)` happens immediately before spawning/starting the child that must bind those ports; holding the reservation through spawn/start guarantees bind failure once real binaries are installed.
- If `cargo test` fails with “failed to build archive … failed to open object file: No such file or directory”, treat it as a potentially corrupted/stale `target/` artifact set; a clean rebuild (`cargo clean`) has been sufficient to restore deterministic passes.
- In process-worker real-job tests, keep at least one `StateSubscriber<ProcessState>` alive for the full helper lifecycle; if all subscribers are dropped, publisher writes fail with `state channel is closed` and unrelated job assertions become misleading.
- For PG replica setup tests, avoid batching `ALTER SYSTEM` commands in one `batch_execute` call; use simple connectivity checks plus bounded retry polling because standby startup can briefly report `SqlStatus::Unreachable` before stabilizing.
- Full-suite `cargo test --all-targets` can expose postgres readiness races not seen in isolated test reruns; add a short bounded connectivity probe loop before first SQL work in real-PG fixture tests.
- In async real-binary tests, wrapping assertions in a `Result` flow and then running explicit shutdown cleanup after the main result helps avoid panic chains while preserving failure context when both test logic and teardown fail.
- In async Rust tests, avoid relying on underscore-only bindings to keep critical resources alive; a fixture struct that owns subscriber + context + namespace guard makes lifetime intent explicit and reduces brittle publish-channel failures.
- For HA integration tests, drive multi-worker progression with an explicit ordered cycle (`dcs step -> ha step -> process step -> ha step`) so process outcomes are visible to HA deterministically without relying on scheduler timing.
- `tokio::net::TcpListener` does not always provide a `try_accept()` helper; use a tiny `tokio::time::timeout(...)` around `accept()` to keep `step_once()` non-blocking for contract tests.
- If a public config struct like `RuntimeConfig` contains schema subtypes, re-export those schema structs publicly (or you’ll be unable to construct configs in `tests/` integration crates).
- `etcd-client` endpoint setup can be lazy; an unreachable endpoint may still let `Client::connect(...)` return `Ok`, so unreachable-path tests should assert first I/O operation failure (`put/get`) rather than constructor failure alone.
- If test namespaces are RAII-cleaned via `NamespaceGuard`, failure-time artifacts under namespace paths can disappear before diagnosis; write timeline/debug artifacts to a stable workspace path (for example `.ralph/evidence/...`) before teardown.
- For long-lived bug tasks, first verify the reported failure string still exists in current code/logs; if absent and stress + full gates pass, close as stale-with-evidence instead of forcing speculative code changes.
- When making test-harness binary lookup helpers fallible, grep for `require_*bin` usages across non-harness test modules (`pginfo`, `process`, `dcs`) before coding; signature fallout is wider than `src/test_harness` alone.
- For DCS watch-path bug reports, add paired regressions at both `refresh_from_etcd_watch` (assert `had_errors`) and `step_once` (assert faulted/not-trusted publication) so stale-vs-fixed decisions are backed by explicit contract tests.
- In real `pg_ctl promote` tests bootstrapped from a standalone primary, expect either success or `EarlyExit(code=1)` ("not in standby"); treat only that specific failure as acceptable instead of blanket `JobOutcome::Failure`.
- When converting tests from `.expect(...)` to `?`, make the test `Result` error type match the called API’s error (for example `DecideError` for `decide(...)`) to avoid unnecessary `From` glue and compile churn.
- For stale bug reports about panic behavior, still add a direct missing-path contract test on the fallible helper; it turns “already fixed” claims into durable regression evidence.
- `new_state_channel(...)` starts at `Version(0)`; in contract tests, a single successful `publish(...)` should assert `Version(1)`, while untouched channels should remain at `Version(0)`.
- In one-shot TLS tests, `tokio-rustls` may surface `UnexpectedEof` when the server closes without `close_notify`; treat it as acceptable only if full HTTP response bytes were already received and parsed.
- For mTLS contract tests in `step_once` workers, prefer asserting end-to-end request rejection over handshake-only failure checks, because post-handshake worker policy can still close unauthorized clients deterministically.
- When logging gate runs with `... | tee ...`, always enforce `set -o pipefail`; otherwise `tee` can hide a failing `make` exit status and produce a false green task.
- If acceptance criteria ask for log phrase checks that may not exist in native tool output, still archive explicit grep artifacts (including "not found") so pass/fail evidence remains auditable without inventing output.
- If real-binary policy is centralized in harness helpers, audit HA/e2e fixtures for leftover direct `.tools/...` path checks; those bypasses silently diverge from enforcement env behavior unless routed through the same helper.
- For etcd-backed DCS adapters, bootstrap with `get(prefix)` and then create `watch(prefix)` from `header.revision + 1`; if watch responses are canceled/compacted, mark unhealthy and force a full reconnect+resnapshot cycle.
- For strict conninfo typing, keep `PgConnInfo` in HA/process job specs and render only at command boundary; parser-level `UnsupportedKey` checks catch typo drift (for example `sslmdoe`) before any process job is dispatched.
- Even when `make test` (`cargo test --all-targets`) already exercises BDD test binaries, still run `make test-bdd` separately when task policy requires it so evidence logs map 1:1 to acceptance commands.
- In `story-full-verification` task files, final completion metadata is expected as `<status>done</status> <passes>true</passes> <passing>true</passing>`; keep all three tags aligned to avoid downstream task-state ambiguity.
- For full-suite regression tasks, using `CARGO_BUILD_JOBS=1` for each serial `make` gate can reduce intermittent Cargo archive/object race noise while preserving deterministic pass/fail evidence.
- For closure-only remediation tasks where all bug files are already done, still archive a task-local `bug-inventory.log` plus `bug-pending-status.log`; this makes "no active bugs" auditable instead of implicit.
- Before implementing new HA real-integration coverage, run the existing `ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix` with `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1`; if it already proves leader-key writes, process-driven postgres effects, and HA phase progression, treat the task as stale-satisfied and finish with evidence instead of duplicating tests.
- For encapsulation refactors, introducing a narrow worker-facing trait (for example `DcsHaWriter`) with a blanket impl over `DcsStore` lets HA switch to typed methods while existing concrete stores/test doubles keep working via coercion to `Box<dyn DcsHaWriter>` without forcing API/DCS-worker raw-store rewrites.
