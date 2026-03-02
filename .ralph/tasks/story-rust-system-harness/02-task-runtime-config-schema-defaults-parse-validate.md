---
## Task: Implement runtime config schema defaults parser and validation <status>not_started</status> <passes>false</passes> <priority>ultra_high</priority>

<blocked_by>01-task-core-types-time-errors-watch-channel</blocked_by>

<description>
**Goal:** Define and validate the full typed runtime configuration model.

**Scope:**
- Create `src/config/schema.rs`, `src/config/defaults.rs`, `src/config/parser.rs`, `src/config/mod.rs`.
- Implement `RuntimeConfig`, nested config structs, `ProcessConfig`, and `BinaryPaths`.
- Implement `load_runtime_config`, `apply_defaults`, and `validate_runtime_config`.

**Context from research:**
- Build Order step 2 and typed-runtime-input rule in plan.
- Config must fully control runtime behavior; no hidden magic constants.

**Expected outcome:**
- Runtime config can be loaded from file, defaulted, and rejected on invalid settings.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] All config structs from plan are present and wired through `RuntimeConfig`.
- [ ] Validation covers mandatory binary paths, timeout bounds, and required HA/DCS settings.
- [ ] Table-driven tests cover valid configs, missing fields defaulting, and invalid-file rejections.
- [ ] Run targeted config tests.
- [ ] Run `make check`.
- [ ] Run `make test`.
- [ ] Run `make lint`.
- [ ] Run `make test-bdd`.
- [ ] If any fail, use `$add-bug` skill to create bug task(s) in `.ralph/tasks/bugs/`.
</acceptance_criteria>
