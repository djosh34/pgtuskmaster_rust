# Current Tasks Summary

Generated: Tue Mar 24 02:09:27 PM CET 2026

# Task `.ralph/tasks/bugs/git-current-lines-script-fails-after-deleting-tracked-files.md`

```
## Bug: git_current_lines.sh fails after deleting tracked files <status>not_started</status> <passes>false</passes> <priority>medium</priority>

<description>
`bash .ralph/git_current_lines.sh` emits `wc: ... No such file or directory` once a reduce-loop slice deletes tracked files before commit. This was detected on 2026-03-24 while validating the worker-startup-boundary reduction after deleting `src/api/startup.rs`, `src/ha/startup.rs`, `src/pginfo/startup.rs`, and `src/process/startup.rs`.
```

==============

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/05-reduce-loop.md`

```
## Task: Reduce Code Loop <status>in_progress</status> <passes>false</passes> <priority>high</priority>

<description>
Your only goal is to reduce code and clean up the codebase.
```

