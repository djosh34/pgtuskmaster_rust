---
## Task: Enforce strict Rust lint policy and forbid unwrap expect panic in runtime code <status>not_started</status> <passes>false</passes> <priority>ultra_high</priority>

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
- [ ] Exhaustive file/module checklist is completed:
- [ ] `Makefile` lint target enforces strict clippy denies consistently.
- [ ] Lint configuration file(s) are added/updated (for example `clippy.toml` and/or crate attributes) with explicit deny policy.
- [ ] Runtime crate entrypoints are updated to deny `unwrap`/`expect`/panic patterns where appropriate.
- [ ] Existing runtime violations are replaced with typed error handling (no `unwrap` introduced).
- [ ] Any test-only lint allowances are narrowly scoped and documented inline.
- [ ] Documentation is updated with lint rationale and local usage notes.
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
