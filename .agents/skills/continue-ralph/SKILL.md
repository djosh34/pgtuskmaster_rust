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

### 2. Stage all and clean up unsafe files

Stage everything first:

```bash
git add -A
```

Then unstage anything that should NOT be committed:

```bash
git rm --cached -r --ignore-unmatch log/ *.log
git rm --cached --ignore-unmatch .env .env.* *credentials* *secret* *.pem *.key
```

Review `git diff --cached --stat` — if you see anything else suspicious (large binaries, temp files, etc.), `git rm --cached` those too and tell the user.

### 3. Commit changes

```bash
git commit -m "auto-commit before ralph continue" || true
```

The `|| true` handles the case where there's nothing to commit.

### 4. Run task switch

```bash
/bin/bash .ralph/task_switch.sh
```

### 5. Start Ralph without attaching

Use the start-only flag so the command returns immediately after starting the service:

```bash
/bin/bash .ralph/ralph.sh --start
```

### 6. Report

Tell the user that Ralph has been continued and the task has been switched.
