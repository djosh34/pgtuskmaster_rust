On startup, read your previous progress
```bash
/bin/bash .ralph/progress_read.sh "<codex>"
```

Append to the progress log — it is your working memory across context windows.
Please write very often.
```bash
/bin/bash .ralph/progress_append.sh "<codex>" << 'EOF_APPEND_PROGRESS_LOG'
- what you did
- what happened
- should do next, after quitting immediately due to context limit
EOF_APPEND_PROGRESS_LOG
```



apply skill improve-code-boundaries on one of these:

- src/
- tests/

when done: 
- make lint
- make test
- DO NOT RUN! make test-long
- commit (including .ralph changes)
- push
- bash .ralph/task_switch.sh
