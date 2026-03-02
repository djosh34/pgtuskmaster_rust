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


## Cross application applicable learnings
- Config defaulting is safer when required fields stay required in parse structs; optional-only defaults plus strict `deny_unknown_fields` avoids silent typo drift.
- Keep `target/` ignored in Rust repos to avoid accidental large artifact commits during task-level `git add -A` workflows.
