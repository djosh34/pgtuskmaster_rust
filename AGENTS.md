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
