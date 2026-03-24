## Plan: Remove String-Overlay Test-Config Layer Onto Typed Documents

### Why this reduction target

Plan 49 confirmed the narrower operator auth/TLS helper surface is not the real cost center. The code smell is one layer broader:

- `src/config_v2/parser/private_schema.rs` already owns the typed config schema through `RuntimeDocument`, `OperatorDocument`, and the nested config structs.
- Test support still forces many callers to leave that typed owner and rebuild config as raw TOML snippets passed through `render_runtime_test_config_toml(..., extra_sections)` and `render_operator_test_config_toml(..., extra_sections)`.
- The string-overlay path duplicates schema knowledge in many files: `src/process/cluster.rs`, `src/pginfo/worker.rs`, `tests/cli_binary.rs`, `src/config_v2/parser/load_config.rs`, and `tests/ha/support/givens/mod.rs`.
- The failed plan-48 and plan-49 attempts both fought one local symptom at a time. This plan targets the shared abstraction that makes those local string overlays exist at all.

This is the better boundary because the typed schema already exists. The reduction opportunity is to keep callers inside that typed owner, mutate the real document structs directly, render once, and delete the generic string-overlay layer.

### Current overlap already verified

- `private_schema.rs` already defines the full typed runtime/operator shapes and already serializes them to TOML.
- `render_runtime_test_config_toml(...)` and `render_operator_test_config_toml(_with_parts)(...)` currently act as string assembly wrappers around those typed shapes.
- `join_rendered_sections(...)`, `toml_path_source(...)`, and `toml_string_secret(...)` exist only to support callers that still describe config as text instead of mutating the typed document.
- `tests/ha/support/givens/mod.rs` is the clearest worst boundary: it renders operator TOML, parses it back into `toml::Value`, parses a second runtime overlay string into `toml::Value`, recursively merges tables, and serializes again.
- Other call sites mostly use the same pattern on a smaller scale: create a base runtime/operator document, then bolt on `[process.*]`, `[logging.*]`, `[postgres.*]`, `[api.*]`, or `[pgtm.api]` sections as strings.

### Execution plan

1. Remove the failed dirty slice before adding the broader refactor.
   - Revert the plan-49-only cfg-gated helper exports and constructor surface in `src/config_v2/mod.rs`, `src/config_v2/parser/mod.rs`, `src/config_v2/parser/private_schema.rs`, `src/cli/config.rs`, `src/config_v2/parser/load_config.rs`, `tests/ha/support/givens/mod.rs`, and `tests/ha/support/observer/pgtm.rs`.
   - Restore the worktree to the last known reduction baseline before starting the new slice.

2. Expose typed test-document entrypoints from the real schema owner, not string overlays.
   - In `src/config_v2/parser/private_schema.rs`, add test-support builders that return `RuntimeDocument` and `OperatorDocument` directly, reusing the existing structs instead of inventing new DTOs.
   - Keep rendering as a thin final step: serialize the typed document once at the boundary.
   - Prefer one small mutation-oriented surface such as `render_*_test_config_with(...)` or typed builder-return helpers, rather than more format-string helpers.

3. Rewrite runtime test callers onto typed mutation.
   - Replace raw `extra_sections` use in `src/process/cluster.rs`, `src/pginfo/worker.rs`, `tests/cli_binary.rs`, and `src/config_v2/parser/load_config.rs` by mutating the typed `RuntimeDocument` directly before rendering.
   - Inline existing path/secret construction through the already-owned typed config enums (`PathSource`, `SecretSource`, TLS structs, auth structs) instead of reconstructing TOML syntax in strings.
   - For the few parser-boundary tests that intentionally need invalid shapes, keep the smallest direct raw TOML fixture necessary and do not widen the typed API to represent invalid states.

4. Collapse the HA fixture onto the same typed owner.
   - Rewrite `tests/ha/support/givens/mod.rs` so it builds one typed `RuntimeDocument`, sets its runtime-specific fields directly, assigns `pgtm` from the typed `OperatorDocument` serialization path once, and removes the parse-merge helpers.
   - Keep `tests/ha/support/observer/pgtm.rs` on the same shared operator builder so there is a single owner for observer/operator config shape.

5. Delete the redundant layer.
   - Remove `join_rendered_sections(...)` if no callers remain.
   - Remove `toml_path_source(...)` and `toml_string_secret(...)` if all valid-shape callers are now typed.
   - Remove or collapse the old `render_*_test_config_toml(..., extra_sections)` helpers if they become one-line wrappers with no real value.

6. Validate the reduction.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require improvement beyond the tracked `+8109 -11146 diff: -3037` baseline.
   - Run `make check`, `make lint`, `make test`, and `make test-long`.

### Guardrails

- Do not add a new parallel hierarchy of test-only patch DTOs. Reuse the existing typed schema structs or stop.
- Do not preserve both the typed-mutation path and the old `extra_sections` path long term. The goal is to delete one layer.
- Do not encode intentionally invalid parser fixtures in the typed API. Invalid-shape tests may keep tiny raw TOML fragments where necessary.
- If converting the call sites requires exposing too many currently-private internals, stop and shrink the public surface instead of widening it blindly.
- If the typed mutation surface still grows code more than the deleted string overlays and merge helpers, switch this plan back to `TO BE VERIFIED` with the failed-yield note before executing further.
- Failed yield note: the direct typed-document path is blocked by visibility. External test-support callers cannot mutate `RuntimeDocument` / `OperatorDocument` without either making a broad private-schema surface public or adding wrapper builders, and the partial public-helper attempt already regressed the tracked reduction baseline from `+8109 -11146 diff: -3037` to `+8172 -11151 diff: -2979`. Replan around a narrower owner-preserving reduction instead of widening the schema further.

TO BE VERIFIED
