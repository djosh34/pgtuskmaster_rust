## Progress Log

On startup, read your previous progress
```bash
/bin/bash .ralph/progress_read.sh "<codex>"
```

Append to the progress log — it is your working memory across context windows.
Please write to it as if it is your diary, and write very often. All updates, confusions, thinking, progress, anything you personally want to write.
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

We STRONGLY advice against using 'mut', and MOST of the time it can be replaced by pure and functional patterns.
If you find a place that has a lot of 'mut' and potentially could be refactored, please add a bug using the add-bug skill, that will refactor that entire code block

Also never swallow/ignore any errors. That is a huge anti-pattern, and must be reported as add-bug task.

This is greenfield project with 0 users. 
We don't have legacy at all. If you find any legacy code/docs, remove it.
No backwards compatibility allowed!
You are free and encouraged to make large code/schema changes, if that will improve the codebase.

Never run `cargo test` in this repo.
For validation, prefer make targets
If you need a focused local test while developing, use `cargo nextest run ...`, not `cargo test`.

## Cross application applicable learnings
- `git commit` triggers a post-commit hook that builds the mdBook and publishes `dist/` to the separate `pgtuskmaster-docs` repo; expect an extra docs publish step and ensure your environment has the needed Node/mdBook dependencies configured.
