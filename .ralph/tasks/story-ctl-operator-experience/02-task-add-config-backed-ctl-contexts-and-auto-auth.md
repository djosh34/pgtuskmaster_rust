## Task: Add Config-Backed `pgtm` Configuration And Automatic Auth/TLS Discovery <status>done</status> <passes>true</passes>

<priority>medium</priority>

<description>
**Goal:** Let operators point `pgtm` at the existing runtime config file with `-c` and avoid manually repeating base URLs, tokens, and TLS parameters on every command. The higher-order goal is to make the CLI feel like a first-class operator interface rather than a thin one-shot HTTP wrapper.

**Scope:**
- Integrate `pgtm` with the existing runtime config struct system instead of introducing a second standalone client config file format.
- Add only the smallest possible client-specific config subsection if something is truly missing for the client use case.
- Make `-c <config.toml>` the only required global input for normal operator use.
- Add CLI support to resolve cluster URL, auth, and TLS trust material from that shared config.
- Extend shared secret references so secrets can come from file-based injection or environment variables without requiring extra CLI flags.
- Do not require users to duplicate node inventories in the config. Reuse cluster state and DCS data where possible.
- Design client-side TLS config deliberately: `pgtm` is only an HTTP client to the API and a DSN printer for PostgreSQL, not a PostgreSQL client itself.
- Keep the resulting UX minimal: common operator commands such as `status`, `switchover`, `primary`, `replicas`, and `status --watch` should not need repeated connection flags once the config is loaded.
- Update docs and tests so the operator story is centered on `pgtm -c config.toml`.

**Context from research:**
- `RuntimeConfig` and `RuntimeConfigInput` already exist in `src/config/schema.rs`, including API listen/auth/TLS settings and debug enablement.
- The shared config types already support path-or-inline values through `InlineOrPath` and `SecretSource`, but they do not yet support env-backed secret references.
- DCS member records already contain PostgreSQL host and port, so `pgtm primary` and `pgtm replicas` should not require duplicated node inventories in config.
- The likely minimal client-specific addition is an API URL override when `api.listen_addr` is only a bind address and not a usable client target; if that is needed, it should live in a small dedicated `[pgtm]` subsection and act as an override rather than replacing the shared server config model.
- The current CLI does not read config and requires direct base URL plus token flags on every call.
- Current server config only describes server-side TLS posture; it does not fully describe TLS material for `pgtm` as an HTTP client or for printed PostgreSQL client DSNs.
- For v1, do not add a separate `server_name` override unless implementation constraints prove it necessary; the URL host should be the TLS hostname.

**Expected outcome:**
- Operators can run common commands as `pgtm -c config.toml ...` without retyping authentication and connection details.
- No duplicated cluster node inventory is required in the config.
- Shared config structures are reused as much as possible, and any new client-only subsection is minimal and explicit.
- Secret resolution works consistently for both server and client-facing config needs through file-backed or env-backed references.
- The docs can stop repeating long command prefixes and token-export boilerplate in every guide.

Expected config direction:

```toml
[api]
listen_addr = "0.0.0.0:8080"
security = { tls = { mode = "required", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } }, client_auth = { client_ca = { path = "/etc/pgtuskmaster/tls/client-ca.pem" }, require_client_cert = true } }, auth = { type = "role_tokens", read_token = "read-secret", admin_token = "admin-secret" } }

[pgtm]
api_url = "https://db-a.example.com:8443" # optional override; otherwise reuse api.listen_addr

[pgtm.api_client]
ca_cert = { path = "/etc/pgtm/api-ca.pem" }
client_cert = { path = "/etc/pgtm/operator.crt" }     # optional
client_key = { env = "PGTM_API_CLIENT_KEY" }          # optional

[pgtm.postgres_client]
ca_cert = { path = "/etc/pgtm/postgres-ca.pem" }      # optional override only if different
client_cert = { path = "/etc/pgtm/postgres.crt" }     # optional override only if different
client_key = { path = "/run/secrets/postgres.key" }   # optional override only if different
```

Override rules:
- `pgtm` uses `[pgtm].api_url` when present, otherwise derives the API target from `api.listen_addr`.
- `pgtm primary --tls` and `pgtm replicas --tls` use `[pgtm.postgres_client]` if present.
- If `[pgtm.postgres_client]` is absent, TLS-expanded DSN printing falls back to `[pgtm.api_client]`.
- No duplicated node inventory is allowed in config.
- `pgtm` never opens PostgreSQL connections in this task.

</description>

<acceptance_criteria>
- [x] Define and document how `pgtm` reuses the existing runtime config file and struct system.
- [x] `-c <config.toml>` is the normal required operator input; no `--base-url`, token, or TLS CLI flags are required for ordinary usage.
- [x] Update the config schema and parser only where needed for client use, keeping new client-only fields minimal and explicit.
- [x] If a client-only field is required, it is limited to an API URL override in a small `[pgtm]` subsection and otherwise `pgtm` reuses `api.listen_addr`.
- [x] The shared config types support secret references from file paths and environment variables.
- [x] If client-side TLS config is needed, it uses clear semantic names for client connections rather than print-oriented names like `printed_*`.
- [x] `[pgtm.api_client]` is the primary client TLS block for `pgtm` as an API client.
- [x] `[pgtm.postgres_client]` is optional and only acts as an override for TLS-expanded DSN printing when PostgreSQL client material differs from API client material.
- [x] No separate `server_name` field is introduced unless implementation constraints prove it necessary.
- [x] Add integration coverage in `tests/cli_binary.rs` and any needed unit tests for config loading, precedence, and failure modes.
- [x] Ensure the config-backed flow supports at least one local single-node workflow and one multi-node cluster workflow without requiring repeated `--base-url` and token flags.
- [x] The config-backed flow does not require duplicated node inventories for `primary`, `replicas`, or cluster status.
- [x] The config-backed flow supports a flat minimal command surface under `pgtm`, without reintroducing `ha` as a routine operator prefix.
- [x] Update the CLI reference plus the relevant how-to guides to teach the new context-driven operator path.
- [x] Failure modes for missing files, invalid config, missing secret files, missing environment variables, unusable API override configuration, and incompatible TLS/auth settings are explicit and operator-readable.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Execution Plan

### Product decisions and task boundary
- Keep this task focused on config-backed operator context for the current `pgtm` surface and the shared config model. Do not implement the later `status --watch`, `primary`, or `replicas` feature work here; instead, make this task produce the config/runtime plumbing those later tasks will reuse.
- Make `-c, --config <PATH>` the normal operator entry path. Keep direct CLI connection flags only as explicit escape-hatch overrides for tests, troubleshooting, and targeted manual retries, with deterministic precedence instead of duplicating business logic in multiple paths.
- Define precedence once and document it everywhere:
  - API URL: `--base-url` override first, then `[pgtm].api_url`, then a derived URL from `api.listen_addr`.
  - Read/admin tokens: direct CLI flags first, then config-backed auth from `api.security.auth`, preserving the current read-then-admin fallback for read-only calls.
  - API client TLS material: only from `[pgtm.api_client]`.
  - PostgreSQL DSN TLS material for future `primary` / `replicas`: `[pgtm.postgres_client]` when present, otherwise fall back to `[pgtm.api_client]`.
- Keep the v1 TLS hostname rule minimal: do not add a separate `server_name` override. The URL host is the TLS hostname, and configs that need a different client-visible hostname must use `[pgtm].api_url`.
- Require an explicit `[pgtm].api_url` when `api.listen_addr` is not usable as a client target, such as `0.0.0.0`, `::`, or any deployment where the bind address is not the operator-facing address. Treat that as a first-class validation/runtime error, not an implicit guess.

### Shared config model changes
- Extend `src/config/schema.rs` so `RuntimeConfig` and `RuntimeConfigInput` gain an optional `pgtm` subsection with the smallest client-only surface needed for this story:
  - `PgtmConfig { api_url: Option<String>, api_client: Option<PgtmApiClientConfig>, postgres_client: Option<PgtmPostgresClientConfig> }`
  - `PgtmApiClientConfig { ca_cert: Option<InlineOrPath>, client_cert: Option<InlineOrPath>, client_key: Option<SecretSource> }`
  - `PgtmPostgresClientConfig` with the same semantic fields for later DSN expansion work.
- Replace the current `SecretSource(pub InlineOrPath)` newtype with an explicit secret-bearing enum that supports the existing inline/path forms plus env-backed resolution. The concrete target shape should be equivalent to:

```rust
pub enum SecretSource {
    Path(PathBuf),
    PathConfig { path: PathBuf },
    Inline { content: String },
    Env { env: String },
}
```

- Use that secret-bearing type for every field that is actually secret material:
  - PostgreSQL role passwords.
  - API role tokens in `ApiRoleTokensConfig`.
  - `pgtm` client private keys.
- Keep non-secret client cert / CA inputs as `InlineOrPath`; do not broaden env-backed sourcing to every non-secret content field in this task.
- Update `src/config/mod.rs` exports and the runtime-config test helpers in `src/test_harness/runtime_config.rs` so the new types are easy to construct in unit and integration tests.

### Parser, normalization, and validation work
- Update `src/config/parser.rs` normalization so the new `[pgtm]` block is carried into `RuntimeConfig` without inventing unsafe defaults.
- Add validation for the new secret and client-context fields:
  - env-backed secret references must use non-empty env names;
  - `pgtm.api_url`, when present, must parse as an absolute `http` or `https` URL;
  - if `api.listen_addr` is used as the fallback target, deriving a client URL must reject unspecified bind addresses and clearly instruct the operator to set `[pgtm].api_url`;
  - if `api.security.tls.mode = "disabled"`, reject contradictory `https` API URLs or client TLS blocks that imply TLS must be used;
  - if API TLS is enabled and `[pgtm.api_client]` provides `client_cert`, require `client_key`, and vice versa;
  - if API TLS client-auth is required by the server config, fail early when `pgtm` lacks the matching client cert/key pair;
  - `[pgtm.postgres_client]` should be optional, but if present it must satisfy the same cert/key completeness rules because future DSN printing will rely on it directly.
- Keep failure text operator-readable and field-specific so missing files, missing env vars, unusable API target derivation, and incompatible TLS/auth combinations land as explicit diagnostics rather than transport noise.

### Shared material resolution
- Introduce a shared config-resolution helper module instead of adding another independent secret loader. The goal is one source of truth for reading:
  - `InlineOrPath` file/inline content,
  - `SecretSource` file/inline/env secret content.
- Put that helper under the config surface (for example `src/config/materialize.rs`) and update the currently duplicated loaders in `src/process/jobs.rs` and `src/postgres_managed.rs` to use it. Do not turn this task into a broad repository-wide refactor unless another call site is directly required by the config-backed `pgtm` path.
- Keep the helper purely fallible and context-rich: every returned error should include the logical field name being resolved so CLI and server code can surface actionable messages without `unwrap`, `expect`, or swallowed IO/env failures.

### CLI architecture changes
- Update `src/cli/args.rs` so `Cli` accepts `-c, --config <PATH>` plus the existing manual overrides. Do not make the clap layer try to resolve precedence by itself; parse the raw flags, then resolve them in one dedicated operator-context step.
- Add a small `src/cli/config.rs` or equivalent module responsible for:
  - loading `RuntimeConfig` via `load_runtime_config`,
  - resolving the effective API URL from CLI/config precedence,
  - resolving read/admin tokens from CLI or config secret sources,
  - resolving API TLS client material from `[pgtm.api_client]`,
  - resolving PostgreSQL client TLS material into a future-proof struct for later tasks even if this task only stores it and validates it.
- Refactor `src/cli/mod.rs` so it constructs this resolved operator context once before matching subcommands, then passes a ready-to-use client config into the HTTP client layer.
- Refactor `src/cli/client.rs` so the HTTP layer can be built from a structured operator-context config instead of loose raw inputs. Preserve or add a thin convenience constructor as needed for existing harness call sites that still build clients directly during the transition. Build the `reqwest::Client` with:
  - timeout as today,
  - optional custom root CA from `[pgtm.api_client].ca_cert`,
  - optional mTLS identity built from `client_cert` plus `client_key`.
- Preserve the current auth semantics:
  - read operations use the read token when present and fall back to admin,
  - admin operations require the admin token when auth is enabled.
- Extend `src/cli/error.rs` with a config/operator-context error variant instead of overloading all config failures into transport noise. Keep exit-code behavior deterministic; if a new exit code is added, document it in the CLI reference and test it explicitly.

### Concrete operator behavior to deliver in this task
- `pgtm -c config.toml status` must work without `--base-url` or token flags in the common single-node path.
- `pgtm -c config.toml switchover request` and `pgtm -c config.toml switchover clear` must work without repeated token flags when config auth is present.
- The task should not duplicate node inventory in config. For this pass, the config-backed context only needs to resolve the API target and auth/TLS material for the existing commands while leaving later cluster/member resolution to the follow-on tasks.
- Even though `primary` / `replicas` are not implemented here, wire the parsed `[pgtm.postgres_client]` block into the resolved operator context now so the later DSN task can reuse the same precedence and validation rules instead of reopening the schema.

### Tests to add or update
- Update parser tests in `src/cli/args.rs` so they cover:
  - `-c` / `--config` parsing,
  - CLI override coexistence with config mode,
  - stable defaults when only `-c` is present.
- Add unit tests in `src/config/parser.rs` for:
  - env-backed secret parsing and validation,
  - API role token secret sources,
  - explicit `pgtm.api_url` acceptance/rejection,
  - rejection of wildcard/unspecified `api.listen_addr` fallback without override,
  - TLS cert/key completeness and incompatible TLS/auth settings.
- Add focused unit tests around the new shared resolver so missing files, unreadable files, unset env vars, and empty env values produce stable field-qualified errors.
- Expand `src/cli/client.rs` tests to cover auth selection after the refactor and to verify request construction still uses the correct tokens after moving resolution into structured config.
- Extend `tests/cli_binary.rs` with real-binary coverage for config-backed flows. At minimum add cases for:
  - `pgtm --help` advertising `-c`,
  - connection-refused or local-test-server `status` using only `-c`,
  - `switchover request` using config-backed admin auth,
  - missing config file,
  - missing env-backed token/key,
  - unusable derived API target without `[pgtm].api_url`.
- Keep the workflow split aligned with the current test surfaces:
  - `tests/cli_binary.rs` should own spawned-binary smoke and failure-path coverage.
  - The existing HA harness should own the real multi-node operator workflow, because it already drives `pgtm` switchover behavior against live cluster fixtures.
- Add at least one single-node and one multi-node operator workflow that exercise the config-backed path without repeated `--base-url` / token flags:
  - single-node: a config-backed `status` flow in `tests/cli_binary.rs` or equivalent spawned-binary coverage;
  - multi-node: a config-backed `switchover request` or repeated `status` flow through the HA fixture in `tests/ha/support/multi_node.rs` or the owning end-to-end test module.
- Keep tests against real binaries mandatory. Do not skip `tests/cli_binary.rs`, and do not weaken the existing end-to-end coverage to compensate for the new config path.

### Documentation work
- Because the requested `update-docs` skill is not available in this session, update docs directly in the repository during execution.
- Update `docs/src/reference/runtime-configuration.md` to document:
  - the new `[pgtm]`, `[pgtm.api_client]`, and `[pgtm.postgres_client]` sections,
  - env-backed secret syntax,
  - the API target derivation and override rules,
  - the intended fallback from `postgres_client` to `api_client` for later DSN helpers.
- Update `docs/src/reference/pgtm-cli.md` so synopsis, global options, examples, and exit-code docs are centered on `pgtm -c config.toml`.
- Rewrite the operator-facing how-to/tutorial pages that currently repeat `--base-url` and token boilerplate, especially the switchover, monitoring, single-node setup, and HA-cluster tutorial paths, so the operator story is context-first rather than per-command flag repetition.
- Remove stale prose that still frames `pgtm` as only a thin one-shot HTTP wrapper once config-backed operator context exists.
- Rebuild generated docs after source edits instead of hand-editing `docs/book`.

### Verification and completion sequence
- After implementation, run a targeted search across live source/doc trees for stale operator examples that still require repeated `--base-url` / token boilerplate where the new config-driven path should be shown.
- Run `make docs-build` before the Rust gates so broken mdBook links or renamed examples fail early.
- Run `make check`.
- Run `make test`.
- Run `make test-long`.
- Run `make lint`.
- Only after every required gate passes:
  - tick the acceptance boxes that are truly complete,
  - set `<passes>true</passes>`,
  - run `/bin/bash .ralph/task_switch.sh`,
  - commit all changes, including `.ralph` updates, with `task finished [task name]: ...`,
  - push with `git push`,
  - stop immediately.

NOW EXECUTE
