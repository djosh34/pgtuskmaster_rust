## Plan: Capture Deleted Tracked File Regression For `git_current_lines.sh`

### Current state already verified

- `.ralph/git_current_lines.sh` already routes `git ls-files -z` through `tracked_existing_paths()` and filters each tracked path with `[ -f "$path" ]` before `wc -l` sees it.
- A throwaway git repo repro with one committed `src/main.rs`, one committed `tests/basic.rs`, and then a deleted tracked `src/main.rs` now prints clean totals:
  - `src/: 0 lines across 0 existing git-tracked files`
  - `tests/: 2 lines across 1 existing git-tracked files`
  - `total: 2 lines across 1 existing git-tracked files`
- `git diff -- .ralph/git_current_lines.sh` is empty, so this behavior is already present in the checked-out branch rather than an uncommitted local fix.

### Why execution is not safe yet

- The task requires strict red-green TDD with one red test first.
- A faithful regression test for the reported `wc: ... No such file or directory` bug would be green at `HEAD`, because the script already avoids passing deleted tracked files into `wc`.
- Reverting the working script just to manufacture a red phase would violate the task's actual product goal, and broadening the task to a different failure mode would change the scope without evidence.

### Execution contract

- This task will proceed by adding retroactive regression coverage for already-correct behavior at `HEAD`.
- The red-green requirement cannot be satisfied literally for the originally reported deleted-file bug without first breaking working code, so execution should preserve the current script behavior and land the missing coverage instead.
- If the new regression test exposes a different real failure while being written or manually verified, execution must stop, switch the plan back to `TO BE VERIFIED`, and rewrite the task around that concrete failing case before further edits.

1. Lock the execution policy for the already-fixed behavior.
   - Treat the task as a coverage gap: the script behavior is already correct, but it lacks an automated regression test for deleted tracked files.
   - Do not revert or weaken `.ralph/git_current_lines.sh` just to manufacture a red phase.

2. Add one public-behavior regression test around the script.
   - Create a focused integration test under `tests/` that shells out to `.ralph/git_current_lines.sh` with `std::process::Command`.
   - Follow the existing integration-test style from `tests/cli_binary.rs`: return `Result<(), String>`, decode stdout/stderr explicitly, and create a uniquely named temp directory under `std::env::temp_dir()` instead of adding a new test helper crate.
   - Build a temporary git repo fixture inside the test, commit tracked `src/` and `tests/` files, delete one tracked file, run the script, and assert:
     - exit status is success,
     - stderr does not contain `No such file or directory`,
     - stdout reports counts only for the still-existing tracked files.

3. Change the script only if the new test exposes a real remaining hole.
   - Keep the existing `tracked_existing_paths()` ownership boundary if it still proves to be the right minimal shape.
   - Do not introduce new wrappers, enums, or a second counting path; reuse the current `count_files()` / `count_lines()` flow unless the test demonstrates a concrete defect.

4. Manually verify after the test-driven change set.
   - Re-run the deleted-tracked-file repro from a throwaway repo after the test passes.
   - If manual verification still shows a broken path, capture that exact path with the next red test before making another script change.

5. Run the required repo gates in order.
   - `make check`
   - `make lint`
   - `make test`
   - `make test-long`

### Guardrails

- Keep the task scoped to `.ralph/git_current_lines.sh` and one script-focused regression test.
- Reuse standard-library command execution and temp-path patterns already used in repo tests; do not add a new shell-testing framework.
- There is no existing repo test that already builds a temporary git repo fixture, so the execution turn should keep that setup local to the new regression test instead of inventing shared helpers prematurely.
- Do not silence stderr or weaken assertions just to make the test pass; the point is to preserve the current clean failure-free behavior in dirty worktrees.
- If execution finds a real still-failing case that differs from the original deleted-file bug, switch the task back through `TO BE VERIFIED` with that exact contract mismatch written down before continuing.

NOW EXECUTE
