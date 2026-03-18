---
name: list-active-tasks
description: Refresh the Ralph task summary, read the current task summary and choose-task rules, then produce a table based only on those files. Use when asked to report active backlog blockers or prioritize remaining work.
---

# List Active Ralph Tasks

## Goal

Produce a table using only the Ralph-generated task summary and the task-selection rules:

1. Refresh the summary by running `.ralph/update_task_summary.sh`.
2. Read `.ralph/current_tasks.md`.
3. Read `.ralph/ralph-choose-task.md`.
4. Base the result only on those two files after the refresh step. Do not inspect task files directly.
5. Render a table for the currently active tasks visible in `.ralph/current_tasks.md`.
6. Order the table from chosen first to chosen last using the selection precedence and gating rules from `.ralph/ralph-choose-task.md`.
7. Put tasks Ralph would choose earliest at the top and tasks Ralph would choose later at the bottom.
8. Use `.ralph/ralph-choose-task.md` only to interpret or explain ordering, priority, blocked-task handling, and bug-first selection rules when they are relevant to the table.
9. Do not mention implementation details used to refresh or gather the data. Do not mention bash.

## Output

Return only a concise table unless the user explicitly asks for extra explanation.

The default table order is mandatory: chosen first to chosen last.

Recommended columns:

- Task
- Path
- Passes
- Priority
- Blocked By
- Notes
