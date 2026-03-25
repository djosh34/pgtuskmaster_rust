---
name: continue-ralph
description: Continue the Ralph loop. Commits all changes, switches task, and restarts Ralph. Triggers on "continue ralph", "continue with loop", "continue loop", "resume ralph".
---

## Steps

Execute these steps in order. Use Bash for all commands.

### 1. Stop Ralph service first (mandatory)

Always stop the service before any git/task-switch/restart steps, regardless of current state:

```bash
/bin/bash .ralph/ralph.sh --stop
```

Then confirm it is inactive before continuing by checking the script-managed status output:

```bash
/bin/bash .ralph/ralph.sh --status
```

Expected status includes `Active: inactive` or `Loaded: not-found`.

### 2. Stage all and review staged changes

Stage everything first:

```bash
git add -A
```

Then review what is staged:

```bash
git status --short
git diff --cached --stat
```

Use common sense to spot anything suspicious that should NOT be committed, such as logs, temp files, generated junk, large binaries, or unrelated local artifacts. If needed, unstage exact paths with:

```bash
git restore --staged <path>
```

Tell the user if you had to unstage anything suspicious.

### 3. Commit changes

```bash
git commit -m "auto-commit before ralph continue" || true
```

The `|| true` handles the case where there's nothing to commit.

### 4. Run task switch

```bash
/bin/bash .ralph/task_switch.sh
```

### 5. Delete the STOP file and start Ralph without attaching

Delete `.ralph/STOP` first so Ralph can actually resume iterations:

```bash
rm -f .ralph/STOP
```

Use the start-only flag so the command returns immediately after starting the service:

```bash
/bin/bash .ralph/ralph.sh --start
```

### 6. Verify Ralph is actually running and logs are moving

After restarting, confirm the service is active:

```bash
/bin/bash .ralph/ralph.sh --status
```

Then check whether fresh logs are appearing by comparing two short snapshots a few seconds apart:

```bash
journalctl --user -u ralph-worker.service -n 5 --output=short-iso --no-pager
sleep 5
journalctl --user -u ralph-worker.service -n 5 --output=short-iso --no-pager
```

Treat logs as moving only if the second snapshot contains fresh output that was not present in the first one.

### 7. Report

Tell the user that Ralph has been continued and the task has been switched. Also report:
- whether `ralph-worker.service` is running
- whether the logs appear to be moving
