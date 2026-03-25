---
name: ralph-log
description: Check whether Ralph is running and whether its logs are actively moving. Use when the user asks if Ralph is running, if it is stuck, if logs are still moving, or to inspect recent Ralph service output.
---

## Steps

Execute these steps in order. Use Bash for all commands.

### 1. Report what you are doing at each step

Before each major action, briefly say what you are checking. At minimum, report these items as you do them:

- Checking Ralph service status
- Reading recent Ralph logs
- Running the 5-second log movement check
- If needed, running each 30-second retry and which retry number it is
- Declaring either that logs are moving or that Ralph appears stuck

Do not stay silent while waiting through the retries.

### 2. Check service status first

Use the Ralph launcher status command:

```bash
/bin/bash .ralph/ralph.sh --status
```

Report clearly whether `ralph-worker.service` is active/running, inactive, or not found.

### 3. If Ralph is running, inspect recent logs

Read the most recent unit logs:

```bash
journalctl --user -u ralph-worker.service -n 50 --output=short-iso --no-pager
```

Summarize what the worker appears to be doing right now.

### 4. Verify that logs are moving

Start with one short comparison:

```bash
journalctl --user -u ralph-worker.service -n 5 --output=short-iso --no-pager
sleep 5
journalctl --user -u ralph-worker.service -n 5 --output=short-iso --no-pager
```

Treat logs as moving immediately if there is fresh output in the second snapshot that was not present in the first one. Use timestamps and changed lines in the explanation.

If nothing changed after 5 seconds, do not call it stuck yet. Instead, wait 30 seconds and re-check up to 5 times total. Report each retry before you do it, for example "No new lines after 5 seconds, waiting 30 seconds for retry 1 of 5."

Use this command shape for each retry:

```bash
journalctl --user -u ralph-worker.service -n 5 --output=short-iso --no-pager
sleep 30
journalctl --user -u ralph-worker.service -n 5 --output=short-iso --no-pager
```

Stop as soon as you observe fresh output and report that logs are moving.

Only call Ralph stuck if there is still no fresh output after the initial 5-second check plus all 5 of the 30-second retries.

### 5. Report precisely

Always answer these two points explicitly:

1. Is Ralph running?
2. Are the logs still moving?

If Ralph is not running, say that directly and skip the "moving logs" claim.

If Ralph is running but there was no fresh output during the full wait window, say that Ralph appears stuck based on no log movement across the initial 5 seconds plus 5 additional 30-second waits.
