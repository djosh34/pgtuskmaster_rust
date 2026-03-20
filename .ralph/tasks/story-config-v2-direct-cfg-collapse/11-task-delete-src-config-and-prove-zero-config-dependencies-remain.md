## Task: Delete `src/config/` And Prove Zero Config Dependencies Remain <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Delete `src/config/` entirely and prove that no code, tests, docs, or fixtures depend on it anymore. The higher-order goal is to close the story with a hard architectural proof instead of stopping at a partial migration.

**Scope:**
- `src/config/`
- `src/lib.rs`
- all `src/**`, `tests/**`, `docs/**`, `docker/**`, and task/docs references that still mention old config paths or types

**Context from research and user decision:**
- this is a hard requirement: `config/` package will be removed post migration, with zero remaining dependencies
- older tasks/story material in the repo still describe weaker config migration directions
- the codebase currently has wide `crate::config::*` usage across runtime packages, dev support, and tests

**Implementation direction:**
- delete `src/config/`
- remove re-exports and module declarations for old config in `src/lib.rs`
- search exhaustively for:
  - `crate::config`
  - `pgtuskmaster_rust::config`
  - `RuntimeConfig`
  - `PgtmConfig`
  - old parser/validator entrypoints
- update docs/examples/fixtures that still refer to old config paths or type names
- if any package still needs a local config mirror struct to compile, the earlier package tasks were not finished; fix them instead of keeping a compatibility shim
- add a progress log section to this task as deletions are completed and keep `<passes>false</passes>`
- when this task is fully complete, update the task body with `QUIT IMMEDIATLY`

**Expected outcome:**
- `src/config/` is gone
- the repo builds and tests on V2 config only
- the architecture rule is enforceable by search, not just by intention

</description>

<acceptance_criteria>
- [ ] `src/config/` is deleted
- [ ] `src/lib.rs` no longer declares or re-exports old config modules/types
- [ ] Repo-wide search finds zero code dependencies on `crate::config` or `pgtuskmaster_rust::config`
- [ ] Docs/examples/fixtures no longer describe old config paths or old config type names
- [ ] The task body contains a progress log of removals and still keeps `<passes>false</passes>`
- [ ] The task body is updated to include `QUIT IMMEDIATLY` when done
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
