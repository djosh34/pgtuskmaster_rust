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
- When adding fields to shared config structs (`ApiConfig`), always run `make check --all-targets` equivalent immediately because examples and contract fixtures outside `src/` are frequent compile-break points.
- For new API surface that depends on internal snapshots, prefer keeping response projection logic in controller/worker unit tests and use BDD tests for black-box route/auth and DCS mutation assertions when integration tests cannot construct crate-private snapshot types directly.
- When expanding a harness from single-instance to clustered resources, keep the single-instance API as compatibility wrappers and add additive cluster APIs (`*_cluster`) so existing real-binary fixtures keep compiling while the new topology migrates incrementally.
