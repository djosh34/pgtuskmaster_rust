## Task: Align doc file order and names with the rendered site structure <status>completed</status> <passes>true</passes>

<description>
Make the docs source tree easier to navigate by aligning file names and ordering conventions with the rendered website structure.

The agent must explore the current docs tree, `SUMMARY.md`, and rendered navigation intent first, then implement the following fixed product decisions:
- docs source file naming should help a contributor understand the rendered order without guessing
- file names and ordering conventions should match the website structure closely enough that the source tree is not fighting the book navigation
- the source layout should become easier to navigate for humans working in the repo, not only for mdBook
- this should be done without adding ugly clutter or arbitrary complexity that makes file paths worse

This task exists because docs should be easier to work on directly in the repo, and the source tree should reflect the reading order instead of obscuring it.

The agent should use parallel subagents after exploration if that materially helps with doc-tree cleanup and link updates.
</description>

<acceptance_criteria>
- [x] Docs source naming and ordering conventions are aligned with the rendered site structure
- [x] `docs/src/SUMMARY.md` and the source tree no longer fight each other in obvious ordering/naming ways
- [x] The docs source tree is easier to navigate directly from the filesystem
- [x] Links and references remain correct after any renames or reordering
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make test-long` — passes cleanly
- [x] `make lint` — passes cleanly
</acceptance_criteria>

## Detailed Execution Plan (Draft 1, 2026-03-07)

### 1. Planning baseline from the required exploration

- The current rendered navigation is chapter-first and mostly coherent in `docs/src/SUMMARY.md`, but the source tree still contains several obvious fights against that structure.
- The concrete mismatches already confirmed during planning are:
  - `Start Here` in `docs/src/SUMMARY.md` points at `docs/src/introduction.md` instead of a chapter-local landing page such as `docs/src/start-here/index.md`.
  - `Glossary And Key Terms` renders under `Interfaces` in `docs/src/SUMMARY.md`, but the file actually lives at `docs/src/concepts/glossary.md`.
  - The source tree currently contains empty directories `docs/src/architecture`, `docs/src/operations`, `docs/src/testing`, and `docs/src/verification`, which look like stale or abandoned naming experiments and make the filesystem harder to trust.
- The rest of the top-level chapter directories already map acceptably to the rendered book intent without forcing ugly numeric prefixes or excessively long path names.
- Skeptical review conclusion: do not rename `operator/` or `assurance/` in this task. Those directory names are slightly shorter than the rendered labels, but they are still semantically obvious and do not create the same chapter-placement confusion as `introduction.md` and `concepts/glossary.md`.
- Non-goal: do not introduce numbered filename prefixes such as `01-...` or `10-...`. That would make the tree more obviously ordered, but it would also add the kind of path clutter this task explicitly wants to avoid.

### 2. Fixed structural decisions to execute later

- Move `docs/src/introduction.md` to `docs/src/start-here/index.md` so the chapter landing page lives with the rest of the Start Here material.
- Move `docs/src/concepts/glossary.md` to `docs/src/interfaces/glossary.md` so the file path matches the rendered placement under Interfaces.
- Remove the now-unused `docs/src/concepts/` directory if it becomes empty after the glossary move.
- Remove the stale empty directories:
  - `docs/src/architecture`
  - `docs/src/operations`
  - `docs/src/testing`
  - `docs/src/verification`
- Keep the remaining chapter directory names as:
  - `start-here`
  - `quick-start`
  - `operator`
  - `lifecycle`
  - `assurance`
  - `interfaces`
  - `contributors`
- Keep existing leaf-page filenames unless execution finds a concrete naming conflict. The currently proven source-tree confusion is about chapter placement and stale directories, not about every individual page title.

### 3. Files and references expected to change during `NOW EXECUTE`

- Structural moves:
  - `docs/src/introduction.md` -> `docs/src/start-here/index.md`
  - `docs/src/concepts/glossary.md` -> `docs/src/interfaces/glossary.md`
- Navigation updates:
  - `docs/src/SUMMARY.md`
- Docs pages likely to need path updates after those moves:
  - `docs/src/start-here/docs-map.md`
  - `docs/src/interfaces/index.md`
  - `docs/src/lifecycle/index.md`
  - any other markdown file found by a repository-wide link/reference search for `introduction.md` or `concepts/glossary.md`
- Navigation parity verification:
  - confirm every live markdown page under `docs/src/` other than `docs/src/SUMMARY.md` is either linked from `docs/src/SUMMARY.md` or intentionally removed as stale during execution
- Filesystem cleanup:
  - remove any empty/stale chapter directories listed above after confirming they are still empty at execution time
- Task bookkeeping:
  - this task file
  - normal `.ralph` state files updated by the standard Ralph workflow

### 4. Exact execution sequence for the later `NOW EXECUTE` pass

- Re-open this task file and confirm the terminal marker says `NOW EXECUTE`.
- Re-check that the structural targets above still match the live tree before editing, but do not restart broad exploration unless the task file itself is stale or contradicted by new repo changes.
- Perform the two planned file moves first:
  - move `docs/src/introduction.md` into `docs/src/start-here/index.md`
  - move `docs/src/concepts/glossary.md` into `docs/src/interfaces/glossary.md`
- Update `docs/src/SUMMARY.md` immediately after the moves so the rendered navigation points at the new paths:
  - `Start Here` -> `./start-here/index.md`
  - `Glossary And Key Terms` -> `./interfaces/glossary.md`
- Run a repository-wide search for old path references and update every remaining hit that points at the moved files.
- Run a filesystem-vs-navigation parity check after the path updates:
  - compare the live markdown files under `docs/src/` against the paths referenced by `docs/src/SUMMARY.md`
  - if any unexpected orphan page remains, decide explicitly whether it should be linked, moved, or deleted before continuing
- Re-read the affected Start Here and Interfaces chapter landing pages to make sure the new file locations still read naturally and do not contain stale “go up one directory” assumptions.
- Remove the stale empty directories only after the link/path updates are complete and the directories are confirmed empty.
- Tick off the relevant acceptance criteria in this task file only after the structure, links, and cleanup are all complete.

### 5. Link-update sweep to perform during `NOW EXECUTE`

- Search for `introduction.md` across the repo and update each result deliberately rather than with a blind replace, because some references may describe the old state historically inside task files.
- Search for `concepts/glossary.md` across the repo and update each live docs reference to `interfaces/glossary.md`.
- Search for markdown links under `docs/src/` after the moves to catch any relative-path breakage caused by the new file locations.
- Search for `](./introduction.md)` and `](./concepts/glossary.md)` specifically in `docs/src/SUMMARY.md` and live docs pages so no rendered-navigation references are left behind by accident.
- Rebuild the docs navigation mentally from `docs/src/SUMMARY.md` after edits to ensure that chapter landing pages and subordinate pages now sit together in the filesystem the same way they render in the book.

### 6. Verification and closeout order for the later execution pass

- After the structural edits and link updates, run:
  - `make docs-build`
  - `make docs-hygiene`
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- If any gate fails, fix the issue and re-run the failed gate until the full required set is green.
- Only after all required gates pass:
  - mark the acceptance criteria as complete in this task file
  - set `<passes>true</passes>` in this task file
  - run `/bin/bash .ralph/task_switch.sh`
  - commit all modified files, including `.ralph` bookkeeping, with `task finished [task name]: ...`
  - push with `git push`

### 7. Skeptical review conclusions applied before promotion to `NOW EXECUTE`

- Re-checked the tree and `docs/src/SUMMARY.md`; the two planned moves remain the only high-value structural renames proven by the repo state.
- Explicitly decided not to rename `operator/`, `assurance/`, or the `quick-start/` leaf pages in this task because that would add churn without resolving a currently demonstrated navigation mismatch.
- Strengthened the execution plan so it fail-closes on docs integrity, not only on the two file moves:
  - added a required parity check between live `docs/src/` markdown files and `docs/src/SUMMARY.md`
  - added `make docs-hygiene` alongside `make docs-build` before the heavier gate suite

NOW EXECUTE
