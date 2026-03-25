## Plan: Collapse Config Parser Raw/Runtime Shape Duplication

### Why this is the next reduction target

The config parser is still paying for a large shadow type layer that mostly exists to deserialize TOML, then immediately explode back into the canonical runtime config by hand.

- `src/config_v2/parser/private_schema.rs:51-172` defines parser-only wrappers plus a large `RuntimeDocument` tree, and `src/config_v2/parser/private_schema.rs:241-667` keeps runtime-shaped subtrees for postgres, DCS, logging, API, token auth, and operator TLS.
- `src/config_v2/parser/load_config.rs:70-188` then destructures that tree field by field, rethreads the same data through local temporaries, and rebuilds the canonical `RuntimeConfigV2` from `src/config_v2/types.rs:32-247`.
- `src/config_v2/parser/load_config.rs:741-856` and `src/config_v2/parser/load_config.rs:956-1000` repeat the same boundary problem for API/operator auth and TLS: parse a raw token/TLS shape, then manually fan it back into `ApiAuth`, `ApiTransport`, and `OperatorConfigV2`.
- `src/config_v2/parser/private_schema.rs:174-239` adds a second normalization pass over the raw document, and `load_config.rs:42-46`, `56-59`, and `245-247` all have to remember to call it before using the parsed data.

That is exactly the wrong boundary: config validation and normalization should stay private to the parser, but the parser should not keep a full shadow runtime-ish schema that only exists so another file can manually reconstruct the real config.

### Current overlap already verified

- `private_schema::TokenAuthConfig` and related operator TLS inputs (`src/config_v2/parser/private_schema.rs:615-667`) exist only so `load_config.rs` can re-interpret them with `token_auth_mode`, `take_token_sources`, `resolve_operator_client_tls`, and `map_runtime_api_auth_at` (`src/config_v2/parser/load_config.rs:651-856`).
- `private_schema::ApiTransportConfig`, `ApiTlsConfig`, and `TlsServerConfig` (`src/config_v2/parser/private_schema.rs:110-152`) are structurally parallel to runtime `ApiTransport` and `TlsConfig` (`src/config_v2/types.rs:45-50` and `188-214`), but the conversion lives centrally in `load_config.rs:783-828`.
- `raw::RuntimeDocument::normalize()` (`src/config_v2/parser/private_schema.rs:174-227`) mutates parsed config just to fix defaultable zero/empty values before `into_runtime_config()` immediately consumes it (`src/config_v2/parser/load_config.rs:70-188`).
- `load_runtime_timing_values()` (`src/config_v2/parser/load_config.rs:242-251`) has to parse the full raw runtime document and call `normalize()` just to read four durations, which is a direct symptom of the oversized raw-document layer.

### Execution plan

1. Collapse normalization into the parser inputs instead of a second whole-document pass.
   - Replace `RuntimeDocument::normalize()` and `default_if_zero()` with targeted serde/default parsing helpers on the specific input fields that currently need "zero means default" or "blank host means default" behavior.
   - Update runtime/operator/timing loaders so they deserialize once and stop remembering an extra normalization step.

2. Move raw-to-runtime conversion next to the input shapes and delete the central mega-destructure.
   - Introduce section-local conversion methods on the parser inputs for cluster, postgres, DCS, process/binaries, logging, API auth, and operator config, so `load_config.rs` stops unpacking almost the entire raw tree in one function.
   - Keep the parser-only source wrappers (`PathOrInline`, `PathSource`, `SecretSource`) where they are useful, but collapse runtime-shaped raw structs that only serve as couriers into smaller conversion-owned inputs or direct runtime construction.
   - Prefer deleting parser structs/functions over renaming them; if a raw type exists only to ferry one runtime struct, remove or merge it.

3. Unify auth/TLS conversion paths so runtime API and operator config stop carrying parallel logic.
   - Reuse one parser-private token-auth extraction path instead of separate "runtime API auth" versus "operator auth" handling.
   - Reuse one parser-private TLS/input resolution path for API HTTPS, postgres server TLS, DCS TLS, and operator client TLS wherever the current logic is identical except for field labels.
   - Preserve the few real semantic differences, such as operator TLS merge rules and the DCS HTTPS requirement.

4. Keep validation inside `src/config_v2/parser/` and tighten the output boundary.
   - Any path resolution, secret loading, URL validation, and unsupported-field rejection must still happen before returning `RuntimeConfigV2` or `OperatorConfigV2`.
   - Do not move validation into runtime/process/API modules; the end result must still be fully validated config objects.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to remain net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Preserve the external TOML surface and current validation semantics; this slice is about deleting internal duplication, not changing user-facing config keys.
- Keep parser internals private. If a helper or input type is only used in `src/config_v2/parser/`, it should stay private there.
- Do not introduce another intermediary "normalized config" wrapper. The point is to remove the extra layer, not rename it.
- If collapsing the raw layer reveals a smaller reusable runtime type that already exists in `src/config_v2/types.rs`, reuse that existing type instead of adding a new mirror struct.

### Expected yield

- Delete `RuntimeDocument::normalize()`, `default_if_zero()`, and the repeated "parse then normalize" call sites.
- Shrink or partially delete `src/config_v2/parser/private_schema.rs` by removing runtime-shaped courier structs that only feed `load_config.rs`.
- Shorten `src/config_v2/parser/load_config.rs` by replacing the central mega-destructure and duplicate auth/TLS helpers with section-local conversions.

NOW EXECUTE
