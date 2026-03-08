## Task: Remove phantom config versioning and restore a single as-is config contract <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Fully remove the hallucinated runtime-config versioning model from this repository. There is one config contract only. There is no `config_version` field, there never was a `v1` config, there never was a `v2` config, and no code, test, doc, fixture, or generated doc artifact may describe or enforce such a split.

This is a cleanup/correction task, not a migration task. The higher-order goal is to restore the repo to a truthful model where runtime config is accepted as-is and the implementation no longer contains any version-envelope logic, version-specific validation text, or migration guidance for a non-existent schema transition.

**Priority:** medium. It does not need to preempt critical blocking work, but it should be scheduled ahead of cosmetic cleanup because the false versioning model can keep propagating into code, tests, and docs if left in place.

**Scope:**
- Remove config schema version fields/types/checks from runtime config parsing and validation.
- Remove all config loader behavior that treats missing `config_version` as an error or that rejects/handles `v1`/`v2`.
- Remove config fixtures/examples/docs that declare `config_version = "v2"` or mention `v1`/`v2` runtime config semantics.
- Remove CLI/tests/docs wording that tells operators to set `config_version` or migrate from `v1` to `v2`.
- Remove generated docs/tmp prompt artifacts and draft docs that repeat the same false story, if those files are tracked in-repo.
- Keep unrelated uses of the strings `v1` / `v2` that are not about runtime config versioning untouched. Examples that must stay include Mermaid `stateDiagram-v2`, ordinary state/version counters, HTTP schema metadata unrelated to runtime config, and other non-config version numbers.
- If more config-versioning references are found outside the paths listed below, they are in scope too. The checklist must be expanded until the removal is exhaustive.

**Context from research:**
- The false versioning model was introduced deliberately by the archived task chain under `story-secure-explicit-node-config`, then spread into code/docs/fixtures.
- Archive evidence:
  - `.ralph/archive/archive_tasks/story-secure-explicit-node-config/01-task-expand-runtime-config-schema-for-explicit-secure-node-startup.md` planned `ConfigVersion` plus version-gated parsing and a v1/v2 split.
  - `.ralph/archive/archive_tasks/story-secure-explicit-node-config/02-task-migrate-parser-defaults-and-validation-to-explicit-enum-driven-config.md` explicitly required `config_version`, made `v2` executable, and rejected `v1`.
  - `.ralph/archive/archive_tasks/story-secure-explicit-node-config/05-task-migrate-fixtures-examples-and-cli-config-surfaces-to-new-schema.md` propagated the story into fixtures, CLI UX, and docs.
- Commit trail:
  - `76f79d8` introduced schema/version gating.
  - `a919931` made the versioned parser executable and rejection-oriented.
  - `e499308` spread the versioning model into fixtures/examples/CLI/docs.
- Progress-log evidence:
  - `.ralph/progress/65.jsonl` records planning language such as “config_version v1/v2 compat strategy” and “safe version gating”.
  - `.ralph/progress/69.jsonl` records “v2 is mandatory now”, CLI capture for missing `config_version`, and v2 migration/doc planning.

**Expected outcome:**
- The runtime config parser reads a single config shape directly, without version envelopes or version-specific branches.
- No tracked runtime config file contains a `config_version` field.
- No runtime-config docs, tests, CLI messages, or fixtures mention `v1`/`v2` config semantics or a migration between them.
- `passes=true` is allowed only after repo-wide evidence shows the phantom versioning model is gone rather than merely bypassed.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] `src/config/parser.rs`, `src/config/schema.rs`, `src/config/defaults.rs`, and `src/config/mod.rs` contain no runtime-config version field/type/check/branch/migration logic
- [ ] No tracked runtime config example or fixture contains `config_version`
- [ ] No CLI or parser test asserts on missing/required `config_version`, `config_version = "v2"`, or rejection of `config_version = "v1"`
- [ ] No docs page, draft page, or tracked generated docs artifact describes runtime config as `v1`/`v2` or instructs the operator to set `config_version`
- [ ] `rg -n "config_version|config version|runtime config.*v1|runtime config.*v2|config_version = \\\"v1\\\"|config_version = \\\"v2\\\"" src tests docs docker .ralph/tasks` returns no config-versioning matches after intentional exclusions are documented
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If the touched area still impacts ultra-long coverage after cleanup: `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan (full dehallucination of config versioning)

### 1) First establish the exact removal boundary
- Start with a repo-wide `rg` for:
  - `config_version`
  - runtime-config mentions of `v1` and `v2`
  - migration wording such as `set config_version`, `explicit secure schema`, `migrate to config_version`, `config_version=v2`
- Classify every hit as one of:
  - must remove now
  - unrelated and safe to keep
  - generated/tracked artifact that must be regenerated or deleted
- Record the intentional exclusions explicitly in the task notes or commit message so the final grep gate is defensible.

### 2) Remove the false model from runtime config code first
- [ ] `src/config/parser.rs`
  - Delete `ConfigEnvelope`/`config_version` parsing.
  - Delete `ConfigVersion`-driven branching.
  - Delete v1-probing compatibility paths and all migration/rejection messages.
  - Normalize and validate a single config shape directly.
  - Rewrite parser tests so they assert the single as-is config contract rather than versioned behavior.
- [ ] `src/config/schema.rs`
  - Remove config-version fields/types from runtime config input structs.
  - Collapse `RuntimeConfigV2Input` naming/semantics if it only exists to support the phantom split.
  - Rename types/functions/comments so they describe one real config contract instead of a staged schema epoch.
- [ ] `src/config/defaults.rs`
  - Remove error text that mentions `config_version=v2`.
  - Ensure defaults/validation helpers no longer assume a versioned config world.
- [ ] `src/config/mod.rs`
  - Remove version-related exports and stale comments.

### 3) Remove the false model from tests and examples
- [ ] `tests/cli_binary.rs`
  - Delete tests for missing `config_version`, `v2` migration hints, and `v1` rejection.
  - Replace them with assertions for the real single-config behavior if needed.
- [ ] `src/config/parser.rs` test fixtures
  - Remove `config_version = "v2"` / `config_version = "v1"` literals and version-specific temp-file names/messages.
- [ ] `docker/configs/cluster/node-a/runtime.toml`
- [ ] `docker/configs/cluster/node-b/runtime.toml`
- [ ] `docker/configs/cluster/node-c/runtime.toml`
- [ ] `docker/configs/single/node-a/runtime.toml`
  - Remove the top-level `config_version` field from every tracked runtime config file.
- [ ] Any additional config TOML literals under `src/`, `tests/`, `examples/`, and harness files
  - Remove the field and adjust expectations to the single config contract.

### 4) Remove the false model from docs and tracked generated artifacts
- [ ] `docs/src/reference/runtime-configuration.md`
  - Rewrite as a single runtime-config reference with no version field and no migration framing.
- [ ] `docs/src/explanation/architecture.md`
  - Remove prose describing the runtime config as a “complete `config_version = "v2"` file”.
- [ ] `docs/src/reference/http-api.md`
  - Only touch if runtime config versioning leaked into HTTP docs. Do not remove unrelated HTTP schema metadata unless it is also part of the same hallucinated story and should be corrected separately.
- [ ] `docs/draft/docs/src/reference/runtime-configuration.md`
- [ ] `docs/draft/docs/src/reference/runtime-configuration.revised.md`
- [ ] `docs/tmp/docs/src/reference/runtime-configuration.prompt.md`
- [ ] `docs/tmp/docs/src/reference/cli.prompt.md`
- [ ] `docs/tmp/docs/src/reference/pgtuskmasterctl-cli.prompt.md`
- [ ] `docs/tmp/verbose_extra_context/runtime-config-summary.md`
- [ ] `docs/tmp/verbose_extra_context/runtime-config-deep-summary.md`
  - Remove or regenerate tracked artifacts so they stop repeating the phantom v1/v2 story.
- [ ] Any other tracked `docs/tmp` or `docs/draft` file that still mentions runtime config versioning
  - Expand the checklist and clean it too.

### 5) Clean up Ralph task/story residue that perpetuates the false model
- [ ] Audit current tracked `.ralph/tasks` files for active instructions that still describe runtime config as `v1`/`v2`.
- [ ] Remove or rewrite active task references that would reintroduce the hallucinated model on a future pass.
- [ ] Do not rewrite archival evidence under `.ralph/archive/archive_tasks/` just to hide history; keep archive as evidence unless project conventions require archive scrubbing. If archive files are intentionally left unchanged, state that explicitly as an exclusion in the final grep evidence.

### 6) Verification gates
- [ ] Run the repo-wide grep gate and confirm only intentional exclusions remain.
- [ ] Run `make check`
- [ ] Run `make test`
- [ ] Run `make lint`
- [ ] If any touched behavior is covered only by ultra-long tests, run `make test-long`

### 7) Passes-true rule
- Do not set `<passes>true</passes>` until all of the following are simultaneously true:
  - runtime config code has no version split
  - tracked runtime config files contain no version field
  - tracked docs no longer mention runtime config v1/v2 semantics
  - parser/CLI/tests no longer talk about config migration or version rejection
  - the grep gate is clean except for explicitly documented non-config exclusions
  - build/test/lint gates are green
