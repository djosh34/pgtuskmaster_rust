---
name: ralph-log
description: Check whether Ralph is running and whether its logs are actively moving. Use when the user asks if Ralph is running, if it is stuck, if logs are still moving, or to inspect recent Ralph service output.
---

## Steps

Execute these steps in order. Use Bash for all commands.

### 1. Check service status first

Use the Ralph launcher status command:

```bash
/bin/bash .ralph/ralph.sh --status
```

Report clearly whether `ralph-worker.service` is active/running, inactive, or not found.

### 2. If Ralph is running, inspect recent logs

Read the most recent unit logs:

```bash
journalctl --user -u ralph-worker.service -n 50 --output=short-iso --no-pager
```

Summarize what the worker appears to be doing right now.

### 3. Verify that logs are moving

Compare two short snapshots a few seconds apart:

```bash
journalctl --user -u ralph-worker.service -n 5 --output=short-iso --no-pager
sleep 5
journalctl --user -u ralph-worker.service -n 5 --output=short-iso --no-pager
```

Treat logs as moving only if there is fresh output in the second snapshot that was not present in the first one. Use timestamps and changed lines in the explanation.

### 4. Report precisely

Always answer these two points explicitly:

1. Is Ralph running?
2. Are the logs still moving?

If Ralph is not running, say that directly and skip the "moving logs" claim.
