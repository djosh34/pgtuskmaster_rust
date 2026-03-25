---
name: add-bug
description: Create a bug task file in .ralph/tasks/bugs/. Triggers on "add bug", "create bug", "new bug", "/add-bug".
---

## Where to create

- `mkdir -p .ralph/tasks/bugs/`
- Write the bug file: `.ralph/tasks/bugs/bug-slug.md`

## Bug file format

```markdown
## Bug: Bug Title <status>not_started</status> <passes>false</passes> <priority>optional: medium|high|ultra high</priority>

<description>
[What is broken and how it was detected.]
</description>

<mandatory_red_green_tdd>
Use Red-Green TDD to solve the problem.
You must make ONE test, and then make ONE test green at the time.

Then verify if bug still holds. If yes, create new Red test, and continue with Red-Green TDD until it does work.
</mandatory_red_green_tdd>

<acceptance_criteria>
- [ ] I created a Red unit and/or integration test that captures the bug
- [ ] I made the test green by fixing
- [ ] I manually verified the bug, and created a new Red test if not working still
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
```