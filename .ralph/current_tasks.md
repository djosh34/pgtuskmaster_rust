# Current Tasks Summary

Generated: Thu Mar 19 01:44:36 AM CET 2026

# Task `.ralph/tasks/bugs/bug-api-worker-reload-certificate-tests-leak.md`

```
## Bug: API worker reload certificate tests leak resources under nextest <status>not_started</status> <passes>false</passes>

<description>
`make test` currently passes but nextest reports leaky tests in `pgtuskmaster_rust::api::worker::tests`:
`reload_certificates_returns_error_when_postmaster_pid_is_stale`,
```

==============

# Task `.ralph/tasks/bugs/bug-ha-long-runs-leak-docker-networks-and-exhaust-address-pools.md`

```
## Bug: HA long runs leak Docker networks and exhaust address pools <status>not_started</status> <passes>false</passes>

<description>
`make test-long` can fail before scenario execution because Docker cannot allocate another compose network:
```

==============

# Task `.ralph/tasks/bugs/bug-ha-rewind-failure-old-primary-rejoins-flakes.md`

```
## Bug: HA rewind failure rejoin scenario flakes between no-primary recovery and write convergence cleanup <status>not_started</status> <passes>false</passes>

<description>
`make test-long` currently fails in `pgtuskmaster_rust::ha::ha_rejoin_and_restart_recovery::rewind_failure_old_primary_rejoins`.
Observed failure modes:
```

==============

# Task `.ralph/tasks/bugs/bug-ha-write-convergence-healthy-test-observes-extra-write.md`

```
## Bug: HA write convergence healthy test observes extra write <status>not_started</status> <passes>false</passes>

<description>
`cargo nextest run --test ha --profile default --no-tests fail` failed in `support::invariants::write_convergence::tests::one_primary_and_two_replicas_are_determined_healthy`.
The failure observed every member at count `4` on `public.write_convergence_invariant` row `1` even though the test expected all members to converge to `3` before the 250ms deadline.
```

==============

# Task `.ralph/tasks/bugs/bug-make-test-nextest-build-fails-missing-rlib-archive.md`

```
## Bug: make test nextest build fails with missing rlib archive <status>not_started</status> <passes>false</passes>

<description>
`make test` currently fails during the `cargo nextest run --workspace --all-targets --profile default --no-tests fail` build step before any tests execute.
```

==============

# Task `.ralph/tasks/bugs/bug-nextest-incremental-linking-can-miss-ha-test-object-files.md`

```
## Bug: Nextest incremental linking can miss ha test object files <status>not_started</status> <passes>false</passes>

<description>
`cargo nextest run --workspace --all-targets --profile default --no-tests fail` failed during verification while linking test `ha`.
```

==============

# Task `.ralph/tasks/bugs/bug-process-postmaster-reload-sighup-test-times-out-with-no-signal-log.md`

```
## Bug: process postmaster reload SIGHUP test times out with no signal log <status>not_started</status> <passes>false</passes>

<description>
`make test` currently fails in `process::postmaster::tests::reload_managed_postmaster_sends_sighup`.
```

==============

# Task `.ralph/tasks/bugs/cli-output-error-flattening.md`

```
## Bug: Preserve typed JSON output errors <status>not_started</status> <passes>false</passes>

<description>
`src/cli/output.rs` converts `serde_json::Error` into a formatted `String` before it reaches the CLI error boundary. That flattens the source error too early and makes it impossible to preserve structured handling in `CliError`.
```

==============

# Task `.ralph/tasks/bugs/docker-harness-typed-errors.md`

```
## Bug: Type Docker harness failures instead of matching stderr strings <status>not_started</status> <passes>false</passes>

<description>
The Docker test harness in tests/ha/support/docker still infers control flow from stderr substrings and emits generic HarnessError::message failures for distinct cases like compose network races, missing inspect fields, empty Ryuk container IDs, and unexpected registration acknowledgements.
This makes failures harder to classify and easy to mask. Explore the existing error types first, then replace these stringly paths with typed variants or structured errors where the caller can branch without parsing messages.
```

==============

# Task `.ralph/tasks/bugs/ha-worker-stringified-errors.md`

```
## Bug: HA worker still flattens recoverable errors into strings <status>not_started</status> <passes>false</passes>

<description>
`src/ha/startup.rs` and `src/ha/state.rs` already carry `WorkerError` through the HA runtime boundary, but `src/ha/worker.rs` still collapses several distinct failure modes into `WorkerError::Message(format!(...))`.
```

==============

# Task `.ralph/tasks/bugs/ha-world-test-helper-still-builds-removed-runtime-wrapper.md`

```
## Bug: HA world test helper still builds removed runtime wrapper <status>not_started</status> <passes>false</passes>

<description>
`cargo nextest run support::config::tests::resolve_configured_executable_rejects_relative_path` exposed a compile failure in `tests/ha/support/world/mod.rs`.
The test helper `test_harness_with_write_convergence` still constructs `HarnessShared { runtime: HarnessRuntime { ... } }`, but `HarnessRuntime` is no longer the live shape of `HarnessShared`.
```

==============

# Task `.ralph/tasks/bugs/namespace-env-notunicode-is-silently-dropped.md`

```
## Bug: Namespace env var NotUnicode is silently dropped <status>not_started</status> <passes>false</passes>

<description>
`src/dev_support/namespace.rs` treats `std::env::VarError::NotUnicode` from `PGTM_TEST_KEEP_NAMESPACE` as `false`, which silently discards an actual error.
```

==============

# Task `.ralph/tasks/bugs/postmaster-typed-error-flattening.md`

```
## Bug: Preserve typed postmaster errors <status>not_started</status> <passes>false</passes>

<description>
`src/process/postmaster.rs` still converts several known failure shapes into `String` payloads before they reach `ManagedPostmasterError`. That flattens typed sources like `io::Error`, `ParseIntError`, and `TryFromIntError` into formatted text, which makes matching and refactoring harder.
```

==============

# Task `.ralph/tasks/bugs/process-jobs-typed-errors.md`

```
## Bug: Process job errors still flatten typed failures into strings <status>not_started</status> <passes>false</passes>

<description>
`src/process/jobs.rs` still uses string payloads for several `ProcessError` variants, and it converts structured errors to `String` via `to_string()` in the secret-resolution path.
```

==============

# Task `.ralph/tasks/bugs/process-typed-errors.md`

```
## Bug: Refine stringly process errors <status>not_started</status> <passes>false</passes>

<description>
The `src/process` module still uses several stringly error variants and `format!`-built `InvalidSpec` paths where the failure shape is known at the call site. This makes error handling harder to match on and hides typed context in strings.
```

==============

# Task `.ralph/tasks/bugs/reload-certificates-signal-log-timeout.md`

```
## Bug: Reload certificates HTTPS test times out waiting for signal log <status>not_started</status> <passes>false</passes>

<description>
`make test` currently fails in `pgtuskmaster_rust::api::worker::tests::reload_certificates_succeeds_for_https_transport_and_signals_postgres`.
The observed error is `signal log /tmp/pgtm-api-worker-reload-success-533849-1773863709231/signal.log was not written in time`.
```

==============

# Task `.ralph/tasks/bugs/test-child-cleanup-ignores-process-termination-errors.md`

```
## Bug: Test child cleanup ignores process termination errors <status>not_started</status> <passes>false</passes>

<description>
The `Drop` cleanup for the fake postgres child in `src/api/worker.rs` ignores failures from `child.kill()` and `child.wait()` with `let _ = ...`.
This swallows cleanup failures and can hide broken test behavior or leaked subprocesses.
```

==============

# Task `.ralph/tasks/bugs/verbose-error-boundary-cleanup.md`

```
## Bug: Reduce verbose error handling to typed boundaries <status>not_started</status> <passes>false</passes>

<description>
The codebase still has several early-error-flattening patterns:
```

