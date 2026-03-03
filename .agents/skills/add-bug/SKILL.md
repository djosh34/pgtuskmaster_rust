---
name: add-bug
description: Create a bug task file in .ralph/tasks/bugs/. Triggers on "add bug", "create bug", "new bug", "/add-bug".
---

## Where to create

- `mkdir -p .ralph/tasks/bugs/`
- Write the bug file: `.ralph/tasks/bugs/bug-slug.md`
- Never modify `.ralph/current_tasks.md` directly

## Bug file format

```markdown
---
## Bug: Bug Title <status>not_started</status> <passes>false</passes> <priority>optional: medium|high|ultra high</priority>

<description>
What is broken and how it was detected.
Tell the agent to explore and research the codebase first, then fix.
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
- No numbering in file names or titles
- If there are too many failures, group related ones into a single bug file
