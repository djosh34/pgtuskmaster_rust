---
name: add-task-as-user
description: Create a task when the USER requests it. Triggers on "add task", "create task", "new task", "/add-task-as-user".
---

Only do research the codebase if asked, otherwise, instead include as part of the task description that the agent reading it must do the exploration itself.
Also, as always recommend using subagents in parallel for exploration -> which creates detailed plan -> implementation in parallel.


## Where to create

- Read `.ralph/current_tasks.md` to see existing stories/tasks
- Find the appropriate story dir, or `mkdir -p` a new one: `.ralph/tasks/story-storyname/`
- Write the task file: `.ralph/tasks/story-storyname/task-slug.md`
- Never modify `.ralph/current_tasks.md` directly

## Task file format

```markdown
---
## Task: Task Title <status>not_started</status> <passes>false</passes>

<description>
What needs to be done (high level, not step-by-step).
Tell the agent to explore and research the codebase first, then implement.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test` — full regular suite passes (including BDD/ignored coverage now grouped in this target)
</acceptance_criteria>
```

## Rules

- `<passes>`: always `false` when creating
- `<priority>`: only add if user explicitly requests (values: ultra high, high, medium, low)
- optional numbering in file names or titles
- Tasks should be high level — tell the agent *what* to achieve, not *how*
- For large tasks: the agent should first PLAN (explore/grep/glob), then use parallel subagents (via the Task tool) to implement all changes concurrently.
