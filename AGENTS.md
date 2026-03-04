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
- ... (add here)
- When moving `pg_basebackup` / `pg_rewind` to use configured non-`postgres` role usernames, real HA e2e must ensure those roles exist on the elected primary *before* starting clone nodes (e.g. `replicator` with `LOGIN REPLICATION`; `rewinder` often needs `SUPERUSER` for `pg_rewind`).
- If you override Postgres auth via managed `hba_file=...`, remember replication clients (`pg_basebackup`, `replication` DSNs) do **not** match `database=all`; include explicit `host replication <user> ...` rules or basebackup will fail with auth errors.
- When reading secrets from files for env vars (e.g. `PGPASSWORD`), trim trailing `\n`/`\r` (`trim_end_matches(['\n', '\r'])`) so newline-terminated files don’t break auth and clippy stays clean.
- When running shell commands that include markdown backticks (for example searching for `` `make test` `` in tasks), always wrap the regex in single quotes; backticks inside double quotes trigger shell command substitution and can accidentally run `make test`.
- This workspace’s `rg` build may not include PCRE2; avoid `rg -P` and prefer Rust-regex-compatible patterns.
- `mdbook init docs` creates `docs/book/` immediately and `mdbook build` writes static output there; ignore `/docs/book/` at repo root to prevent accidental commits.
- On Linux `aarch64`, mdBook upstream releases may only provide a `*-unknown-linux-musl` asset; pin/verify the correct archive per arch instead of assuming a `*-gnu` build exists.
- `mdbook-mermaid` must match the mdBook major line: newer `mdbook-mermaid` releases (e.g. 0.16+) target mdBook 0.5 and can fail under mdBook 0.4 with a preprocessor JSON parse error; for mdBook v0.4.40, pin `mdbook-mermaid` v0.14.0 (depends on mdBook crate v0.4.36) and ensure the preprocessor is on `PATH` during `mdbook build`.
- Policy guard tests for post-start behavior should avoid over-broad helper-name bans (for example `post_switchover(`) and instead ban precise forbidden coordination tokens while adding explicit assertions that allowed admin API/SQL action tokens remain unblocked.
- When runtime startup may use `pg_basebackup` into harness-created data directories, enforce `0700` permissions on those directories (`prepare_pgdata_dir`) before startup; otherwise postgres can fail with `data directory ... has invalid permissions` and the node API never binds.
- When adding fields to shared config structs (`ApiConfig`), always run `make check --all-targets` equivalent immediately because examples and contract fixtures outside `src/` are frequent compile-break points.
- `ConfigError::Validation` uses `field: &'static str`; for actionable missing-field errors in config parsing, prefer `*V2Input` wrapper structs with `Option<T>` leaves + explicit validation instead of relying on serde “missing field” text.
- For new API surface that depends on internal snapshots, prefer keeping response projection logic in controller/worker unit tests and use BDD tests for black-box route/auth and DCS mutation assertions when integration tests cannot construct crate-private snapshot types directly.
- When expanding a harness from single-instance to clustered resources, keep the single-instance API as compatibility wrappers and add additive cluster APIs (`*_cluster`) so existing real-binary fixtures keep compiling while the new topology migrates incrementally.
- In `#[tokio::test(flavor = "current_thread")]` e2e fixtures, `ApiWorkerCtx::run` is not `Send` for `tokio::spawn`; drive API/debug workers with explicit `step_once` pumps inside request helpers (or use a dedicated local task set) instead of spawning them on the multithreaded spawn path.
- Adding `reqwest`/modern rustls dependencies can cause TLS tests to panic with missing process crypto provider; install a default provider (`rustls::crypto::ring::default_provider().install_default()`) in TLS test helpers before `ClientConfig`/`ServerConfig` builders.
- In rustls 0.23, `WebPkiClientVerifier::builder(...)` uses the process-default `CryptoProvider` and can panic when multiple providers are enabled; prefer `WebPkiClientVerifier::builder_with_provider(..., rustls::crypto::ring::default_provider().into())`, and use `*_with_provider` config builders to avoid relying on global provider selection.
- For `cargo test --all-targets`, CLI integration tests should not assume `CARGO_BIN_EXE_<name>` is always set; add a fallback to `target/debug/<bin>` derived from `current_exe()` to keep binary smoke tests portable.
- In long real-binary HA e2e runs, `...has been running for over 60 seconds` is a cargo test heartbeat, not an automatic hang signal; add explicit per-operation and whole-scenario timeouts in the fixture so true stalls fail deterministically with actionable error context.
- For HA leader handling, split semantics: treat unhealthy leader metadata as unavailable for replica/candidate follow decisions, but still treat any conflicting non-self leader record as a split-brain signal for `Primary` fencing; coupling both checks regresses matrix fencing paths.
- On this workspace mount, intermittent linker failures (`cannot find ... .rcgu.o`) can appear even with low parallelism; `cargo clean` before reruns and `CARGO_BUILD_JOBS=1` for validation gates reduces flake risk.
- Real-binary HA scenario sequencing matters: after certain failover transitions, adding a planned switchover in the same matrix can be flaky/stall; keep failover/fencing proof in a dedicated unassisted scenario and keep matrix coverage focused on switchover + no-quorum invariants.
- For recurring object/archive disappearance during test/link on this mount, `CARGO_INCREMENTAL=0` alongside `CARGO_BUILD_JOBS=1` further reduces flake compared to jobs throttling alone.
- When `make test` and `make test-long` run long real-binary HA scenarios in the same lib test invocation, intermittent connection-refused flakes can come from concurrent real e2e interference; `RUST_TEST_THREADS=1` stabilizes gate runs.
- Note: `make test` hard-fails when `RUST_TEST_THREADS=1` (Makefile guard). For stability in required gates, prefer `CARGO_BUILD_JOBS=1` and leave `RUST_TEST_THREADS` unset; for local debugging, use `cargo test -- --test-threads 1` instead of `make test`.
- `make test` is wrapped in `/usr/bin/timeout 120s` and that wall-clock includes compilation; if compilation is required, forcing `CARGO_BUILD_JOBS=1` can push the run over the timeout. Prefer warming builds with `cargo test --all-targets --no-run` (or avoid throttling build jobs) before relying on `make test` as the 120s gate.
- `.ralph/evidence/` is ignored by `.ralph/evidence/.gitignore`; to commit evidence logs for a task, use `git add -f .ralph/evidence/<task-dir>/...`.
- In stress HA tests that sample node API state sequentially, transient dual-primary readings can be polling artifacts; keep hard split-brain assertions tied to deterministic write-id integrity and dedicated dual-primary window checks instead of raw sampled maxima alone.
- For user systemd units tracked in-repo, avoid hardcoded absolute home paths in `ExecStart`/`WorkingDirectory`; prefer `%h/...` specifiers and verify post-fix journal errors using `journalctl --since "$(systemctl --user show <unit> -p ActiveEnterTimestamp --value)"` so historical failures do not create false regressions.
- When introducing a new required field on shared runtime structs like `BinaryPaths`, update non-`src/` examples (`examples/`) in the same patch and run `make check` immediately; example binaries compile under `--all-targets` and are easy to miss.
- In `#[tokio::test(flavor = "current_thread")]` real-e2e fixtures, if startup paths perform blocking/synchronous probes (for example `probe_dcs_cache` via `EtcdDcsStore::connect`), proxy listeners running on the same tokio runtime can starve and cause false DCS connect timeouts; run long-lived proxy listeners on dedicated runtime threads.
- When extracting reusable test certificate builders into shared harness modules, keep `rcgen`/rustls-heavy helpers behind `#[cfg(test)]` in those modules so they stay usable across unit tests without promoting `rcgen` from `dev-dependencies` to runtime dependencies.
- In long `#[tokio::test(flavor = "current_thread")]` HA no-quorum loops, `CliApiClient`/reqwest transport can intermittently fail with connection-reset/send errors; keep CLI/API-client as primary path but add a direct TCP `/ha/state` fallback on transport errors for deterministic e2e polling.
- Some no-quorum states can stall `CliApiClient::get_ha_state()` longer than expected; wrap the await in `tokio::time::timeout` and treat timeouts like transport failures so the TCP `/ha/state` fallback still runs.
- In no-quorum e2e, avoid wrapping a poll helper that spawns tasks in `tokio::time::timeout` unless you also abort/join those spawned tasks; leaked local tasks can accumulate and blow up suite runtime. Prefer passing a small per-request step timeout (e.g. 3s) into the poll helper itself.
- For `pg_ctl stop -m immediate -w`, if the `pg_ctl` process times out, killing only `pg_ctl` can leave Postgres running. Use `postmaster.pid` to find the postmaster PID and send signals to guarantee teardown (prevents port collisions + long `make test` stalls).
- Keep “strict all-nodes fail-safe observed” in a focused test; for fencing/regression invariants, prefer a cutoff based on quorum-loss time so `make test` stays under the hard 120s wall-clock.
- In no-quorum fencing e2e, Postgres can be intentionally stopped as part of fencing; any post-fencing SQL/table integrity probes should be best-effort (or try multiple nodes) to avoid false failures from expected connection-refused.
- If a test harness submodule is only used by in-crate unit tests (e.g. `src/ha/e2e_*.rs` behind `#[cfg(test)]`), gate the harness module behind `#[cfg(test)]` in `src/test_harness/mod.rs` to avoid `cargo clippy --lib -D warnings` unused-import/re-export failures.
- For real process-worker PostgreSQL start/stop fixtures, keep namespace child paths short (especially `socket_dir`, e.g. `sock`) to avoid Unix socket path-length startup flakes that surface as `pg_ctl` early-exit code 1 during full-suite runs.
- When a test harness reserves ports by binding temporary listeners, releasing a port and then binding later in a different task/thread can race with other `:0` binds in parallel tests; prefer passing the already-bound `TcpListener` into the component (e.g. proxy listeners) instead of releasing and rebinding.
- If real-binary tests start failing with `No space left on device` / etcd or initdb early-exit, check root filesystem free space (often `/tmp/pgtuskmaster-rust` accumulates) and delete old `/tmp/pgtuskmaster-*` test directories before retrying.
- Under `cargo clippy -D warnings`, avoid `assert!(false, ...)` patterns (triggers `clippy::assertions_on_constants`); prefer `#[test] -> Result<(), E>` and `return Err(...)` for unreachable branches.
- If a type becomes part of a public config surface, its defining module must be public (not just the type), otherwise integration tests/examples cannot name the type path (e.g. exposing `PgSslMode` required `pub mod pginfo`).
