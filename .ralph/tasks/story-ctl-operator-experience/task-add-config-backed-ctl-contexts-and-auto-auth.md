## Task: Add Config-Backed `pgtm` Configuration And Automatic Auth/TLS Discovery <status>not_started</status> <passes>false</passes>

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
- [ ] Define and document how `pgtm` reuses the existing runtime config file and struct system.
- [ ] `-c <config.toml>` is the normal required operator input; no `--base-url`, token, or TLS CLI flags are required for ordinary usage.
- [ ] Update the config schema and parser only where needed for client use, keeping new client-only fields minimal and explicit.
- [ ] If a client-only field is required, it is limited to an API URL override in a small `[pgtm]` subsection and otherwise `pgtm` reuses `api.listen_addr`.
- [ ] The shared config types support secret references from file paths and environment variables.
- [ ] If client-side TLS config is needed, it uses clear semantic names for client connections rather than print-oriented names like `printed_*`.
- [ ] `[pgtm.api_client]` is the primary client TLS block for `pgtm` as an API client.
- [ ] `[pgtm.postgres_client]` is optional and only acts as an override for TLS-expanded DSN printing when PostgreSQL client material differs from API client material.
- [ ] No separate `server_name` field is introduced unless implementation constraints prove it necessary.
- [ ] Add integration coverage in `tests/cli_binary.rs` and any needed unit tests for config loading, precedence, and failure modes.
- [ ] Ensure the config-backed flow supports at least one local single-node workflow and one multi-node cluster workflow without requiring repeated `--base-url` and token flags.
- [ ] The config-backed flow does not require duplicated node inventories for `primary`, `replicas`, or cluster status.
- [ ] The config-backed flow supports a flat minimal command surface under `pgtm`, without reintroducing `ha` as a routine operator prefix.
- [ ] Update the CLI reference plus the relevant how-to guides to teach the new context-driven operator path.
- [ ] Failure modes for missing files, invalid config, missing secret files, missing environment variables, unusable API override configuration, and incompatible TLS/auth settings are explicit and operator-readable.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
