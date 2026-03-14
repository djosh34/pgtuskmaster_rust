---
name: add-meta-task
description: Create a meta-task (recurring, never-done task). Uses /add-task-as-agent but with meta-task rules. Triggers on "add meta-task", "create meta-task", "new meta-task", "/add-meta-task".
---

## What is a meta-task?

A meta-task is a recurring verification/audit task that is NEVER finished. Every time it is picked up, the engineer must do a FRESH pass. It stays in the backlog forever.

## How to create

Use the `/add-task-as-agent` skill to create the task, with these **mandatory differences**:

### 1. Header uses `<passes>meta-task</passes>` — NEVER `true` or `false`

```markdown
## Task: [Title] <status>not_started</status> <passes>meta-task</passes>
```

### 2. Warning line immediately AFTER the `## Task:` line

```markdown
## Task: [Title] <status>not_started</status> <passes>meta-task</passes>
NEVER TICK OFF THIS TASK. ALWAYS KEEP <passes>meta-task</passes>. This is a recurring deep verification task.
```

### 3. Description must include these rules

- State clearly that this is a **RECURRING META-TASK**
- State that the engineer must do a **FRESH verification** every time
- State: **NEVER set this task's passes to anything other than meta-task**
- Include an `## Exploration` section template for engineers to log their findings with dates

### 4. Acceptance criteria must end with

```markdown
- THIS TASK STAYS AS meta-task FOREVER
```

### 5. Checkboxes are NEVER ticked off

Acceptance criteria checkboxes exist to describe what the engineer must verify each run, but they must **NEVER be ticked off** (`- [x]`). They always stay as `- [ ]`.

## Everything else

Follow `/add-task-as-agent` for file location, naming, structure, parallelization instructions, and all other conventions.
