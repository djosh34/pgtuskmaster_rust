## Bug: git_current_lines.sh fails after deleting tracked files <status>completed</status> <passes>true</passes> <priority>medium</priority>

<description>
`bash .ralph/git_current_lines.sh` emits `wc: ... No such file or directory` once a reduce-loop slice deletes tracked files before commit. This was detected on 2026-03-24 while validating the worker-startup-boundary reduction after deleting `src/api/startup.rs`, `src/ha/startup.rs`, `src/pginfo/startup.rs`, and `src/process/startup.rs`.

The script is supposed to report the current tracked line totals for `src/` and `tests/`, but in a dirty worktree with deletions it still feeds removed paths into `wc`, which makes the measurement step noisy and unreliable for the reduction loop.
</description>

Plan: `.ralph/tasks/bugs/git-current-lines-script-fails-after-deleting-tracked-files_plans/01-capture-deleted-tracked-file-regression.md`

<mandatory_red_green_tdd>
Use Red-Green TDD to solve the problem.
You must make ONE test, and then make ONE test green at the time.

Then verify if bug still holds. If yes, create new Red test, and continue with Red-Green TDD until it does work.
</mandatory_red_green_tdd>

<acceptance_criteria>
- [x] I created a Red unit and/or integration test that captures the bug
- [x] I made the test green by fixing
- [x] I manually verified the bug, and created a new Red test if not working still
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
