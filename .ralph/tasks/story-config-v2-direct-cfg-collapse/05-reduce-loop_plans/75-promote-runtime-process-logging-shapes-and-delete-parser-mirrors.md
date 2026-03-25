## Plan: Promote Runtime Process/Logging Shapes and Delete Parser Mirrors

### Why this is the next reduction target

The parser still owns the canonical structure for runtime-owned HA/process/logging sections, then `load_config.rs` immediately flattens and rethreads those sections back into separate runtime structs.

- `src/config_v2/parser/private_schema.rs:438-612` defines `HaConfig`, `ProcessConfig`, `ProcessTimeoutsConfig`, `BinaryResolutionConfig`, `BinaryPathOverrides`, `LoggingConfig`, `PostgresLoggingConfig`, `LoggingSinksConfig`, `StderrSinkConfig`, `FileSinkConfig`, and `LogCleanupConfig` as parser-private couriers.
- `src/config_v2/parser/load_config.rs:72-113` then has `RuntimeDocument::into_runtime_config()` manually split one conceptual runtime section across `timing`, `binaries`, and `logging`, even though those values already entered in grouped TOML sections.
- `src/config_v2/parser/load_config.rs:166-177` has to deserialize the whole runtime document just to read four timeout values, which is a direct symptom of the wrong ownership boundary.
- `src/config_v2/parser/load_config.rs:782-905` is mostly section reassembly: normalize working-root paths, repack timeouts, resolve binaries, and flatten nested logging back into `src/config_v2/types.rs:129-163`.

### Current overlap already verified

- Parser `HaConfig` plus `ProcessTimeoutsConfig` (`src/config_v2/parser/private_schema.rs:438-491`) exist only to feed runtime `TimingConfig` (`src/config_v2/types.rs:129-135`) through `Duration::from_millis(...)` calls in `src/config_v2/parser/load_config.rs:98-103`.
- Parser `BinaryResolutionConfig` and `BinaryPathOverrides` (`src/config_v2/parser/private_schema.rs:498-511`) only survive long enough for `src/config_v2/parser/load_config.rs:809-846` to rebuild runtime `BinariesConfig` (`src/config_v2/types.rs:140-145`).
- Parser `LoggingConfig` and its nested structs (`src/config_v2/parser/private_schema.rs:514-612`) mirror the same runtime concern as `src/config_v2/types.rs:150-163`, but `src/config_v2/parser/load_config.rs:849-910` has to destructure and flatten them field by field.
- Downstream runtime usage is contained: timing is read in `src/pginfo/worker.rs`, `src/process/jobs.rs`, `src/dcs/worker.rs`, and `src/ha/worker.rs`; binaries are read in `src/process/cluster.rs` and `src/dev_support/binaries.rs`; logging is read mostly in `src/process/cluster.rs`, `src/process/worker.rs`, `src/logging/core/runtime.rs`, and `src/logging/postgres_ingest.rs`.

### Execution plan

1. Move canonical ownership of the HA/process/logging layout into `src/config_v2/types.rs`.
   - Replace the flat `TimingConfig`, `BinariesConfig`, and `LoggingConfig` shape with canonical runtime-owned section types that match the config structure closely enough to reuse the parser section layout instead of mirroring it.
   - Concretely, make `RuntimeConfigV2` own nested `ha`, `process`, and `logging` sections directly, with resolved runtime leaf values (`Duration`, normalized `PathBuf`, resolved binary paths) stored inside those canonical section types rather than in separate flattened mirrors.
   - Prefer promoting or relocating the existing parser section structs over inventing new names. If a parser type can become the canonical runtime type after path/duration cleanup, reuse it.
   - Keep only the genuinely parser-specific leaf wrappers, such as `PathOrInline`, `PathSource`, `SecretSource`, and TLS/auth inputs that still need path or secret resolution.

2. Shrink `private_schema.rs` to input-only wrappers and serde defaults.
   - Delete parser-private structs whose only job is to mirror runtime-owned `ha`, `process`, `binaries`, and `logging` hierarchy.
   - Attach the existing defaulting behavior (`zero means default`, blank listen host fallback, default working root/log paths where still appropriate) to the promoted canonical section types or tiny parser-only leaf inputs.
   - Do not introduce a new intermediate "normalized runtime config" layer. The overlap should disappear, not move.

3. Replace section re-threading in `load_config.rs` with section-local finalize methods.
   - Remove `raw::ProcessConfig::into_runtime_parts`, `raw::BinaryResolutionConfig::into_runtime_config`, and `raw::LoggingConfig::into_runtime_config`.
   - Let the promoted runtime sections finalize themselves in place: resolve working-root-relative paths, resolve binary overrides, and convert timeout millisecond fields to `Duration` at the owning section boundary.
   - Simplify `RuntimeDocument::into_runtime_config()` so it composes validated sections instead of manually reconstructing timing, binaries, and logging field by field.

4. Update downstream consumers directly to the new canonical layout.
   - Rewrite the limited runtime call sites to use the promoted section structure instead of preserving flat compatibility accessors.
   - Adjust focused tests and config helpers in `src/config_v2/parser/load_config.rs` and the affected runtime modules rather than adding adapter methods.

5. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to remain net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not add new mirror types. When parser and runtime shapes overlap, promote one as canonical and delete the other.
- Keep path and secret resolution inside parser/finalize code so runtime consumers still receive validated, resolved config values.
- Prefer direct consumer updates over alias fields or compatibility getters; this project is greenfield and should not keep duplicate shapes alive.

### Expected yield

- Delete the parser `ha/process/binaries/logging` mirror structs from `src/config_v2/parser/private_schema.rs`.
- Delete the section reassembly impls in `src/config_v2/parser/load_config.rs:782-905`.
- Reduce the amount of runtime test-config scaffolding in `src/config_v2/parser/load_config.rs` because the runtime layout will no longer need a second flattened representation.

NOW EXECUTE
