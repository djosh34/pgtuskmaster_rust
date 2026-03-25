---
name: add-task-as-agent
description: Create a task when the AGENT (Claude) needs to create one. Agents should always use THIS skill, not add-task-as-user.
---

## Purpose

This skill creates **focused, tasks** from completed research. Use your extensive research/subagent explore findings to define clear, concrete tasks.

## Prerequisites

- Research/exploration phase has identified what needs to be done
- You understand the scope and can break it into independent pieces

## Where to create

- Tasks go in the same story dir as the research task: `.ralph/tasks/story-storyname/`
- Use descriptive slugs that reflect the goal: `task-convert-config-parsing.md`

## Task file format

```markdown
## Task: [Clear Goal Description] <status>not_started</status> <passes>false</passes>

<description>
**Goal:** [multiple sentences stating the objective]
[also include the higher order goal of this task]
[Complete Discussed goal, things in-scope, out of scope, all decisions made]
[Must include ALL things discussed within the chat/context. You must verify that the task alone, stands on its own, without having any additional context beside this repo only (e.g. you can refer to files)]

**Scope:**
- [Exactly what files/modules/areas are involved]
- [Exactly what specific changes are needed]
- [If Applicable, additional find instructions/to find more or less files based on current codebase]

**Context from research:**
- [which files to edit, which functions to modify, etc]
- [Patterns to follow, examples from codebase]

**Expected outcome:**
- [What should be true when done]

</description>

[Include all code needed for task]
[Include all code for each struct/enum for task]

<acceptance_criteria>
- [ ]  Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
```
