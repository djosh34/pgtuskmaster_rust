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


This is greenfield project with 0 users. 
We don't have legacy at all. If you find legacy, remove it.
Never plan changes thinking of backwards compatibility.
You are free and encouraged to make large code/schema changes, if that will improve the codebase.


## Cross application applicable learnings
- ... (add here)
