---
name: continue-ralph
description: Continue the Ralph loop. Commits all changes, switches task, and restarts Ralph. Triggers on "continue ralph", "continue with loop", "continue loop", "resume ralph".
---

## Steps

Execute these steps in order. Use Bash for all commands.

### 1. Check if Ralph is already running

```bash
systemctl --user is-active ralph-pgtuskmaster.service
```

- If **active**: stop it first before continuing:
  ```bash
  systemctl --user stop ralph-pgtuskmaster.service
  ```
- If **inactive**: proceed (this is the expected state)

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

### 5. Start Ralph with timeout

**CRITICAL**: Use `timeout --signal=QUIT 5s` to prevent hanging. The ralph script attaches to journalctl which blocks forever. We use `--signal=QUIT` (not the default SIGTERM) because ralph.sh traps SIGTERM to *stop* the service, but traps SIGQUIT to *detach* (leave ralph running in background).

```bash
timeout --signal=QUIT 5s /bin/bash .ralph/ralph.sh || true
```

The `|| true` is needed because timeout exits with code 124+signal when it kills the process, which is expected and fine.

### 6. Report

Tell the user that Ralph has been continued and the task has been switched.
