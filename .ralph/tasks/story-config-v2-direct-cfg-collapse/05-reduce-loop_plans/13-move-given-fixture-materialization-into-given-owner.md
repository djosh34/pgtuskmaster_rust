## Plan: Move Given Fixture Materialization Into Given Owner

### Why this reduction target

The next wrong-place hotspot is the fixture materialization block in `tests/ha/support/world/mod.rs`:

- `tests/ha/support/world/mod.rs` is still 1582 lines and mixes harness lifecycle with a long recursive file-materialization pipeline: `materialize_given_fixture`, `copy_shared_fixture_path`, `copy_directory`, `copy_file`, and private-key permission handling.
- `tests/ha/support/givens/mod.rs` already owns the given definition, shared fixture relative paths, compose variant path, compose include rendering, and per-member runtime config rendering. The remaining “copy these given-owned assets into a run directory” behavior belongs with that owner, not with the world/harness bootstrap file.
- The current split forces `world/mod.rs` to understand the static given asset layout even though the given owner already defines that layout, which is classic wrong-placeism and too-much-in-one-file smell from `improve-code-boundaries`.

### Current overlap already verified

- `tests/ha/support/givens/mod.rs` already exposes `shared_fixture_relative_paths`, `compose_variant_absolute_path`, `render_compose_include_file`, and `render_runtime_config`, so it owns the exact data that `materialize_given_fixture` consumes.
- `tests/ha/support/world/mod.rs` only calls `materialize_given_fixture` from harness bootstrap and from the fixture-materialization test block; the recursive copy helpers are otherwise world-local implementation detail.
- The existing `materializes_expected_fixture_assets_and_rendered_configs` test in `tests/ha/support/world/mod.rs` is validating given-owned fixture output, not harness lifecycle behavior.

### Execution plan

1. Move fixture materialization to the given owner.
   - Add a single given-owned materialization entrypoint in `tests/ha/support/givens/mod.rs` that copies shared assets, renders member runtime configs, writes the compose include, and preserves `.key` permissions.
   - Reuse the existing given definition data; do not add a second descriptor or “materializer” wrapper type.

2. Narrow `world/mod.rs` to harness lifecycle.
   - Replace the direct `materialize_given_fixture` call sites in `tests/ha/support/world/mod.rs` with the given-owned entrypoint.
   - Delete the world-local recursive copy helpers that only exist to implement given materialization.

3. Move fixture-output tests with the owner.
   - Relocate or rewrite the current fixture materialization test so it validates the given-owned entrypoint from `tests/ha/support/givens/mod.rs`.
   - Keep world tests focused on harness lifecycle and cleanup behavior, not static given asset rendering.

4. Validate reduction and repo health.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff improves beyond the current `+4478 -6838 diff: -2360` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.
   - Tick the task boxes only after the gates pass.

### Guardrails

- Do not introduce a new fixture-materializer module if `givens/mod.rs` can own the behavior directly.
- If moving the file-copy helpers into the given owner starts duplicating generic file utilities across `world/mod.rs` and `givens/mod.rs`, switch the plan back to `TO BE VERIFIED`.
- Keep `world/mod.rs` focused on harness startup, cleanup, and runtime coordination; static given asset layout should not stay there.

NOW EXECUTE
