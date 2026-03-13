## Task: Rewrite Config Around Typed TOML + `serde` And Remove The Hand-Rolled Parser <status>not_started</status> <passes>false</passes>

<priority>medium</priority>
<blocked_by>Full completion of `.ralph/tasks/story-ctl-operator-experience/`</blocked_by>

<description>
**Goal:** Replace the current hand-rolled config normalization pipeline with a direct TOML + `serde` design centered on the real typed config structs. The higher-order goal is to make the config model boring, typed, and compiler-driven: one real config schema for the daemon, one small operator-facing schema for `pgtm`, field defaults expressed on the structs themselves, and only a much smaller validation layer for true cross-field/domain invariants. This task must remove most of `src/config/parser.rs` and all of the duplicate `*Input` schema that exists only because the current code models ordinary defaults through `Option<T>` plus a second handwritten normalization pass.

**This task belongs to its own story and is intentionally blocked until `story-ctl-operator-experience` is fully complete.** Do not start it early. It is a cross-cutting config/docs/CLI/runtime redesign that will rewrite example config files, change the config surface, and deliberately delete compatibility code that other in-flight operator-story tasks may still be relying on.

**Decisions already made from user discussion:**
- Use plain TOML + `serde`. Do not introduce `confique`, `figment`, `config`, or any other “magic config” crate for this work.
- The current approach is considered overengineered: parallel `RuntimeConfig`/`RuntimeConfigInput`-style trees plus a 3000-line parser/normalizer must be removed.
- `src/config/parser.rs` must either be deleted entirely or reduced to a very small file whose only responsibilities are file IO, `toml::from_str`, and calling focused validation helpers. Do not keep the current monolithic “parser” architecture under a smaller disguise.
- The final config model must deserialize directly into typed structs. Defaults belong on the structs and fields via `Default`, `#[serde(default)]`, and `#[serde(default = "...")]`, not in a second handwritten normalization graph.
- Keep a post-deserialize validation pass only for real relational rules that cannot be expressed through ordinary field typing/defaults. Examples: “this field must match that field”, “this path must not point into that log directory”, “this TLS mode forbids that client material”. Those checks should live in focused validators, not in a giant parser pipeline.
- `DcsConfig` must be typed directly. `DcsEndpoint` already implements `Deserialize`, so the duplicate `DcsConfigInput` string layer is not acceptable and must be removed.
- The config redesign must add a separate `pgtm`-only TOML path with its own checker/loader. `pgtm` must no longer require a full daemon runtime config when it only needs operator-facing settings.
- `pgtm primary` must gain a config-backed host override and optional port override for emitted DSN/target output only. This override is specifically for `pgtm primary`, not a global cluster-routing rewrite and not an override for `pgtm replicas`.
- Config examples and docs must be redesigned along with the code. This is not only an internal refactor.
- PostgreSQL role config must become much simpler: remove `local_conn_identity` and `rewind_conn_identity` as user-facing config concepts. Require only the role definitions the system actually needs (`superuser`, `replicator`, `rewinder`) and allow those role usernames to be the same. Do not preserve the current “must all differ” validation.
- `dcs.scope` and cluster identity should not stay duplicated. The config must have one source of truth for the DCS scope/prefix.
- PostgreSQL binary paths must auto-discover by default and only need explicit override when discovery fails or the operator wants nonstandard paths.
- Runtime temp/log/socket paths should derive from one shared working-root default under `/tmp/pgtuskmaster` unless explicitly overridden. Do not require multiple unrelated path settings for routine local/docker operation.
- HA/process timing values should have sane defaults everywhere and still allow explicit override.
- The `pgtm`-only config should be minimal: API URL, auth, client TLS material, and the primary-command DSN override are the important inputs. Full daemon-only sections must not be required for `pgtm` use.

**Concrete repo context from research:**
- The current “real” config structs live in `src/config/schema.rs`:
  - `RuntimeConfig` and nested daemon types at the top of the file
  - `DcsConfig` is already typed with `Vec<DcsEndpoint>`
- The current duplicate raw input tree also lives in `src/config/schema.rs`:
  - `RuntimeConfigInput`
  - `ProcessConfigInput`
  - `ApiConfigInput`
  - `PostgresConfigInput`
  - `DcsConfigInput`
  - `RoleAuthConfigInput`
  - other `*Input` siblings
- The current load path in `src/config/parser.rs` is:
  - read file
  - `toml::from_str::<RuntimeConfigInput>`
  - hand-normalize into `RuntimeConfig`
  - re-validate the finished value
- `src/config/parser.rs` is currently about 3000 lines and contains:
  - a large `normalize_*` tree
  - stringly field-prefix handling for errors
  - a second large `validate_runtime_config` pass
  - tests that are tightly coupled to the current parser architecture
- The current schema still exposes invalid/impossible states that are rejected only later:
  - `RoleAuthConfig::Tls` exists but is later rejected as unsupported
  - TLS blocks allow “mode requires identity but identity missing”, then reject it later
- `src/config/endpoint.rs` already has a typed `DcsEndpoint` that implements `Deserialize`, but `schema.rs` still duplicates it with `DcsConfigInput { endpoints: Vec<String> }`.
- The current CLI/operator resolution path is in `src/cli/config.rs`. It still assumes full runtime-config loading and treats `[pgtm].api_url` as part of that shared config.
- The current `pgtm primary` / `pgtm replicas` DSN rendering path is in `src/cli/connect.rs`. `build_connection_target(...)` uses `member.routing.postgres.host` and `member.routing.postgres.port` directly, with no operator-facing host override layer. This is why the DSN output is wrong outside the Docker/container truth when the advertised host is only meaningful inside the cluster network.
- `src/runtime/node.rs` currently advertises operator API reachability from `[pgtm].api_url` or `api.listen_addr`.
- The current DCS config surface is too weak for real operator use:
  - `src/config/endpoint.rs` only permits `http`
  - it explicitly rejects username/password
  - it has no first-class DCS TLS/auth sub-structure
- The current docs/examples/runtime configs still show the old surface:
  - `local_conn_identity`
  - `rewind_conn_identity`
  - `dcs.scope`
  - explicit `[process.binaries]`
  - explicit socket/log path sprawl
- Example runtime configs are spread across `docker/*.toml`, `docs/examples/*.toml`, and HA given configs under `tests/ha/givens/**/configs/**`.

**Target architecture and required design direction:**

1. **One real daemon schema, deserialized directly**
- Replace the current `RuntimeConfigInput` approach with direct deserialization into the real daemon config types.
- Remove the parallel `*Input` types from `src/config/schema.rs`.
- Put defaults directly on the daemon types and their fields.
- Prefer stronger field types or small newtypes where they clearly eliminate runtime validation noise.
- Keep the type system honest: if a state is unsupported, do not expose it as a valid config variant.

2. **Small focused validation after deserialization**
- Keep only a much smaller validation layer for cross-field/domain invariants.
- Split validation by responsibility where practical, for example:
  - daemon config structural invariants
  - `pgtm` operator config invariants
  - DCS client security invariants
  - path ownership / self-ingestion invariants
- Do not keep the current giant function that revalidates every basic field after normalization.
- If the old `parser.rs` filename no longer matches its responsibility, rename or delete it instead of keeping a misleading monolith.

3. **Separate daemon config and `pgtm` config loading**
- Introduce two explicit loading/checking entry points:
  - full daemon runtime config for `pgtuskmaster`
  - `pgtm` operator config for the CLI
- The two loaders may share nested data types where that is genuinely useful, but they must not force `pgtm` to parse or validate irrelevant daemon-only sections.
- `pgtm` mode must accept:
  - a minimal TOML file containing only the operator-facing config
  - a full runtime config file that also contains `pgtm` settings
- In `pgtm` mode, daemon-only sections may be present but must not be required. The operator checker should ignore irrelevant daemon-only sections instead of failing because the daemon config is incomplete.

4. **New `pgtm`-only config responsibility**
- The operator-facing config must cover at least:
  - reachable API URL
  - API auth material
  - API client TLS material
  - PostgreSQL client TLS material when `--tls` DSN output needs it
  - a `pgtm primary` DSN host override and optional port override
- The `pgtm primary` override applies only to that command’s emitted target/DSN output.
- `pgtm replicas` must continue to use discovered per-member routing; do not silently apply the primary override there.
- The operator config should remain minimal and not drag in cluster bootstrap/process/DCS internals.

5. **Config-surface redesign requirements**
- Remove `postgres.local_conn_identity` from the user-facing config surface.
- Remove `postgres.rewind_conn_identity` from the user-facing config surface.
- Keep only the role definitions the system truly needs under `postgres.roles`.
- Allow the configured role usernames to be identical. Remove the existing validations that force them to differ.
- Internal local socket connections and internal rewind connections must derive their usernames from the configured roles rather than from duplicated identity blocks.
- Eliminate duplicate DCS scope naming. The implementation must choose one source of truth for the DCS key prefix. The expected direction is to use cluster identity rather than a second `dcs.scope` field.
- Extend DCS config to support first-class authentication and TLS instead of only insecure `http://host:port` endpoints.
- Make the standard local/docker path story simple:
  - one shared runtime working root with a default under `/tmp/pgtuskmaster`
  - derived defaults for sockets, logs, and other runtime artifacts under that root
  - targeted override fields only when needed
- Make all HA/process timing fields defaulted and overridable.
- Make PostgreSQL binary overrides optional:
  - first search the standard executable name on `PATH`
  - then search conventional versioned install locations
  - finally emit an actionable error naming the missing binary
  - explicit config override should still win when provided

6. **DCS typing and client-security redesign**
- `DcsConfig` must deserialize directly with typed endpoints.
- Remove `DcsConfigInput`.
- Redesign the DCS section so security/auth is not smuggled through an endpoint string.
- The DCS config must be able to express:
  - endpoints
  - optional auth
  - optional TLS CA/client cert/client key material
- Update the etcd client wiring accordingly in the DCS layer.
- Do not preserve the current “DCS is always unauthenticated plain HTTP” assumption.

7. **Role/model cleanup to remove impossible states**
- Remove unsupported config variants instead of validating them away later.
- If role TLS auth is not supported, it must not remain in the accepted config enum.
- If a TLS mode requires identity material, encode that in the config model rather than allowing a half-populated struct and rejecting it later.
- Prefer ADT-style config types that make impossible states unrepresentable where practical.

8. **Primary DSN override requirement**
- Add a config-backed override only for `pgtm primary` output so operators outside Docker can get a usable DSN when the member’s advertised PostgreSQL host is only cluster-internal.
- The override must include:
  - required override host
  - optional override port
- If the override port is absent, use the discovered primary’s advertised port.
- This override is not an excuse to rewrite cluster discovery. It is a rendering override for the primary command only.

9. **Docs/examples redesign is required**
- Update the runtime-config reference and all how-to/tutorial/example material to the new schema.
- Remove old examples that still teach the deleted config shape.
- Add clear documentation for:
  - daemon config vs `pgtm` operator config
  - DCS auth/TLS settings
  - runtime working-root default
  - binary autodiscovery and explicit override
  - primary-command DSN override behavior
- Do not leave docs in a half-old, half-new state.

**Scope:**
- Rewrite the config types and load path under `src/config/`.
- Update runtime callers and any code that still assumes the old parser-normalizer output shape.
- Update CLI config loading and DSN rendering for the new `pgtm` operator config and primary override.
- Update DCS client config and etcd-store wiring for typed DCS config plus optional auth/TLS.
- Update PostgreSQL internal-connection config usage to stop depending on `local_conn_identity` and `rewind_conn_identity`.
- Update docs, examples, HA given runtime configs, Docker configs, and test fixtures to the redesigned TOML schema.
- Delete stale tests, helpers, and documentation that only exist to support the removed parser architecture or the removed config fields.

**Out of scope:**
- Do not add a fancy config framework or dynamic config reloading system.
- Do not preserve backwards compatibility with the old TOML shape.
- Do not keep “deprecated” config aliases for old field names.
- Do not keep the old parser/normalizer in parallel with the new typed config path “for safety”.

**Files and modules that must be reviewed and updated:**
- `src/config/schema.rs`
- `src/config/parser.rs`
- `src/config/defaults.rs`
- `src/config/mod.rs`
- `src/config/endpoint.rs`
- `src/config/materialize.rs`
- `src/cli/config.rs`
- `src/cli/connect.rs`
- `src/runtime/node.rs`
- `src/postgres_roles.rs`
- `src/postgres_managed.rs`
- `src/process/worker.rs`
- `src/dcs/etcd_store.rs`
- `src/dcs/store.rs`
- `src/dcs/keys.rs`
- `src/test_harness/runtime_config.rs`
- config-related tests under `src/config/parser.rs` and any new replacement test modules
- CLI tests in `tests/cli_binary.rs`
- HA given/runtime config fixtures under `tests/ha/givens/**/configs/**`
- Docker/example configs under `docker/*.toml` and `docs/examples/*.toml`
- runtime-config and CLI docs under `docs/src/reference/*.md`, `docs/src/how-to/*.md`, `docs/src/tutorial/*.md`, and any architecture/explanation docs that show config snippets

**Implementation guidance and required concrete outcomes:**
- Prefer deleting large amounts of code over creating adapters around the current parser.
- Move parser tests away from a monolithic `parser.rs` unit-test pile if that file is deleted or becomes small.
- Use `serde` defaults and direct `Deserialize` on the actual config structs wherever a field is simply optional-with-default.
- Reserve custom deserialize logic for truly typed parsing needs, not for basic default application.
- Where the current config model duplicates the same concept in two places, collapse it to one authoritative place.
- The `pgtm`-only config may reuse nested auth/TLS structs from the daemon side if that keeps the code smaller and clearer, but it must not inherit daemon-only required fields.
- If a single “shared” config type becomes awkward or drags daemon-only semantics into `pgtm`, split the operator schema cleanly instead of forcing one giant struct to represent both tools.
- Make error messages actionable, but do not preserve the current giant field-prefix match trees just to keep every old error string identical.
- Prefer smaller focused files such as `load.rs`, `validate.rs`, `daemon.rs`, `operator.rs`, or similar over one giant replacement file.

**Expected outcome:**
- The daemon config is ordinary typed TOML + `serde`, not a handwritten normalization engine.
- Most of the current `src/config/parser.rs` logic is gone.
- The duplicate `*Input` config schema is gone.
- `DcsConfig` is typed directly and includes real auth/TLS modeling.
- `pgtm` can load a minimal operator TOML without requiring a full daemon config.
- `pgtm primary` has a config-backed DSN host override and optional port override for operator-facing output.
- The published config surface is simpler:
  - fewer required fields
  - more defaults
  - no duplicate DCS scope
  - no user-facing `local_conn_identity` / `rewind_conn_identity`
  - optional binary overrides instead of mandatory absolute-path boilerplate
- Docs/examples/tests all match the new schema with no stale legacy config guidance left behind.

</description>

<acceptance_criteria>
- [ ] This task remains blocked until `.ralph/tasks/story-ctl-operator-experience/` is fully complete; implementation notes or prep refactors must not bypass that dependency.
- [ ] `src/config/schema.rs` no longer contains the duplicate `RuntimeConfigInput`/`*Input` tree that exists only for normalization; the real config structs deserialize directly from TOML.
- [ ] `src/config/parser.rs` is either deleted or reduced to a small load/dispatch layer; the large `normalize_*` tree and the current monolithic parser architecture are removed.
- [ ] `src/config/mod.rs` exports the new loading/validation entry points cleanly, without preserving the old parser API shape just for compatibility.
- [ ] `src/config/defaults.rs` is reduced to genuine default helpers and no longer acts as a parallel normalization layer for optional input structs.
- [ ] The config model removes user-facing `postgres.local_conn_identity` and `postgres.rewind_conn_identity`, and all code/tests/docs depending on those fields are updated accordingly.
- [ ] PostgreSQL role config keeps only the required roles and no longer rejects equal usernames solely because they match.
- [ ] Any unsupported config variants that were previously accepted and rejected later are removed from the accepted schema instead of left as impossible states.
- [ ] `src/config/endpoint.rs` and the surrounding DCS config model no longer rely on `DcsConfigInput`; `DcsConfig` deserializes directly with typed endpoints.
- [ ] The DCS config surface now includes first-class optional auth and TLS material, and the etcd client wiring under `src/dcs/etcd_store.rs` is updated to honor it.
- [ ] The config surface no longer duplicates DCS scope; one authoritative cluster/DCS scope source remains, and `src/dcs/keys.rs` plus all callers use it consistently.
- [ ] HA/process timing fields have sane defaults expressed on the config structs and still allow explicit override.
- [ ] PostgreSQL binary overrides are optional; autodiscovery searches `PATH` first, then conventional versioned locations, and produces actionable errors when discovery fails.
- [ ] A shared runtime working-root default under `/tmp/pgtuskmaster` exists, and default socket/log/runtime artifact paths derive from it unless explicitly overridden.
- [ ] `src/cli/config.rs` has a distinct `pgtm` operator config load/validation path that does not require a full daemon runtime config.
- [ ] `pgtm` accepts a minimal operator TOML that contains only operator-relevant settings, while still tolerating full runtime files that also carry `pgtm` settings.
- [ ] `src/cli/connect.rs` implements the config-backed `pgtm primary` DSN host override and optional port override, and that override applies only to `pgtm primary`.
- [ ] `pgtm replicas` continues to use discovered per-member routing and is not silently rewritten by the primary override.
- [ ] `src/runtime/node.rs` and any API-advertisement logic are updated consistently with the new daemon/operator config split.
- [ ] `src/postgres_roles.rs`, `src/postgres_managed.rs`, `src/process/worker.rs`, and any related internal connection code no longer depend on deleted identity blocks and instead derive their behavior from the simplified config model.
- [ ] `src/test_harness/runtime_config.rs` and all config-producing test helpers generate the new schema, not the removed legacy shape.
- [ ] Config-related unit tests are rewritten around the new typed-deserialize + focused-validation architecture rather than preserving tests for the removed normalization pipeline.
- [ ] `tests/cli_binary.rs` includes coverage for:
  - minimal `pgtm`-only TOML loading
  - `pgtm primary` DSN override behavior
  - binary autodiscovery failure messaging
  - the simplified config validation rules
- [ ] HA given configs under `tests/ha/givens/**/configs/**` are migrated to the new config surface and keep the HA suite truthful.
- [ ] Docker configs under `docker/*.toml` and docs examples under `docs/examples/*.toml` are migrated to the new schema.
- [ ] Runtime-config reference docs and all affected how-to/tutorial/reference pages are updated to the new TOML shape and remove stale mentions of deleted fields.
- [ ] No compatibility shim remains for the removed config keys or old parser architecture.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
