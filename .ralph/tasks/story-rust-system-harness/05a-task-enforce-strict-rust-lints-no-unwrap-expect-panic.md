---
## Task: Enforce strict Rust lint policy and forbid unwrap expect panic in runtime code <status>done</status> <passes>true</passes> <passing>true</passing> <priority>ultra_high</priority>

<description>
**Goal:** Install and enforce strict Rust linting with explicit denial of `unwrap`, `expect`, and panic-prone patterns in runtime code.

**Scope:**
- Add repository-level clippy lint configuration for strict Rust style and correctness (`clippy.toml` and/or crate-level `#![deny(...)]` as appropriate).
- Update lint entrypoints (`Makefile` and any lint scripts/config) so CI/local lint always enforces the same deny set.
- Refactor existing runtime code paths that currently rely on `unwrap`/`expect`/manual panic patterns to typed error handling.
- Keep tests pragmatic: allow targeted test-only exceptions only where justified and explicitly scoped.

**Context from research:**
- Current lint flow is `cargo clippy --all-targets --all-features -- -D warnings`.
- No repository-level clippy config currently enforces `clippy::unwrap_used` or `clippy::expect_used`.
- Existing `unwrap`/`expect` calls remain in the codebase and are not currently blocked by lint policy.

**Expected outcome:**
- `make lint` fails on newly introduced `unwrap`/`expect`/runtime panic usage and passes with compliant error handling.
- The repository has a clear, documented, reproducible strict lint baseline that can be run locally and in CI.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Exhaustive file/module checklist is completed:
- [x] `Makefile` lint target enforces strict clippy denies consistently.
- [x] Lint configuration file(s) are added/updated (for example `clippy.toml` and/or crate attributes) with explicit deny policy.
- [x] Runtime crate entrypoints are updated to deny `unwrap`/`expect`/panic patterns where appropriate.
- [x] Existing runtime violations are replaced with typed error handling (no `unwrap` introduced).
- [x] Any test-only lint allowances are narrowly scoped and documented inline.
- [x] Documentation is updated with lint rationale and local usage notes.
- [x] `make check` — passes cleanly
- [x] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [x] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [x] `make test-bdd` — all BDD features pass
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan

1. Baseline and scope lock
- [x] Re-read this task and confirm it is the active item from `.ralph/current_task.txt`.
- [x] Capture baseline with `make check`, `make test`, `make test-bdd`, and `make lint` (record failures before any edits).
- [x] Confirm current lint wiring in `Makefile` and current crate root lint attributes in `src/lib.rs`.

2. Exhaustive violation inventory (runtime vs test-only)
- [x] Generate a full `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!` inventory across `src/` and `tests/`.
- [x] Classify each hit as:
  - [x] Runtime path (compiled in non-test library/binary targets).
  - [x] Unit-test block (`#[cfg(test)]`) inside `src/*`.
  - [x] Integration test (`tests/*`).
  - [x] Test-harness code (`src/test_harness/*`, enabled by `feature = "test-harness"`).
- [x] Keep a module-by-module checklist in this task file while executing:
  - [x] `src/lib.rs`
  - [x] `Makefile`
  - [x] `clippy.toml` (add/update only if needed)
  - [x] Runtime modules touched by lint violations (if any are found outside test-only code)
  - [x] Test-only modules requiring explicit scoped lint allowance comments (if needed)
  - [x] Documentation file chosen for lint policy notes (`RUST_SYSTEM_HARNESS_PLAN.md` or equivalent)

3. Define enforcement model before refactor
- [x] Use crate-level deny policy in `src/lib.rs` for runtime code paths (at minimum: `clippy::unwrap_used`, `clippy::expect_used`; plus panic-prone lints where workable).
- [x] Ensure deny scope is active for runtime builds even when running `--all-features` and does not leak into broad test-only code:
  - [x] Use `cfg_attr(not(test), deny(...))` at crate root so non-test runtime targets always enforce denies.
  - [x] Add narrow module-level `#![allow(...)]` in `src/test_harness/mod.rs` (with inline rationale) so `feature = "test-harness"` helpers do not force global relaxations.
  - [x] Avoid feature-gating the crate-level deny itself (for example, do not disable deny via `feature = "test-harness"` conditions).
- [x] Decide whether `clippy.toml` is needed (for shared repository policy such as `disallowed-methods`), and add it only if it improves reproducibility.

4. Update lint entrypoint(s) for deterministic local/CI behavior
- [x] Update `Makefile` `lint` target so strict deny policy is always enforced when running `make lint`.
- [x] Keep command behavior explicit and stable (no hidden env requirements).
- [x] If split clippy passes are required (e.g., general warnings pass plus runtime strict-deny pass), document why in `Makefile` comments.

5. Implement runtime refactors for real violations (if discovered)
- [x] For each runtime violation, replace `unwrap`/`expect`/manual panic path with typed `Result` propagation and contextual error messages.
- [x] Do not introduce new `unwrap`/`expect` anywhere.
- [x] Preserve behavior and keep error surfaces consistent with existing `WorkerError`/domain-specific error enums.

6. Handle test-only exceptions with minimal scope
- [x] Avoid broad crate-wide test allowances.
- [x] Where unavoidable, annotate only the specific module/block with `#[allow(...)]`.
- [x] Add one-line inline justification for each allowance describing why it is test-only and safe.

7. Documentation update
- [x] Add a short lint policy note describing:
  - [x] What is denied (`unwrap`/`expect` and selected panic-prone patterns).
  - [x] How to run the enforced checks locally (`make lint`).
  - [x] Why test-only exceptions are narrowly scoped.
- [x] Keep docs consistent with the actual `Makefile`/crate attributes.

8. Verification sequence (sequential, not parallel Cargo gate runs)
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make test-bdd`.
- [x] Run `make lint`.
- [x] If any command fails, fix and rerun sequentially until all pass.

9. Completion protocol
- [x] Tick every acceptance checklist item with direct evidence from passing commands.
- [x] Update task header tags to done/passing only after all required gates are green.
- [x] Set `<passing>true</passing>` only at the very end.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changes (including `.ralph` files) with:
  - [x] `task finished 05a-task-enforce-strict-rust-lints-no-unwrap-expect-panic: <summary + evidence + challenges>`
- [x] Append new learnings/surprises to `AGENTS.md`.
- [x] Append diary progress to `.ralph/progress` via `.ralph/progress_append.sh`.

</execution_plan>

NOW EXECUTE
