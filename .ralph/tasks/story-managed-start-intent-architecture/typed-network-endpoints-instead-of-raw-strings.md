## Task: [Improvement] Type network endpoints instead of carrying raw strings across runtime <status>completed</status> <passes>true</passes>

<description>
The codebase carries API and DCS endpoint addresses as raw `String` values deep into runtime and harness paths, then parses or binds them at scattered call sites. This was detected during a representation-integrity scan looking for cases where subsystem boundaries retain ad-hoc primitive encodings instead of canonical typed models.

Examples of the drift surface:
- `src/config/schema.rs` stores `api.listen_addr: String` and `dcs.endpoints: Vec<String>`.
- `src/runtime/node.rs` binds `cfg.api.listen_addr.as_str()` directly at worker startup.
- `src/dcs/etcd_store.rs` accepts `Vec<String>` endpoints and clones/reuses them throughout the worker/store path.
- `src/test_harness/ha_e2e/util.rs` reparses endpoint strings with `parse_http_endpoint`, showing that the same concept remains untyped until late.

Explore the current endpoint/address flow first, decide the smallest coherent typed model or models for these network addresses, then refactor the internal paths to use typed representations and keep string encoding only at true external boundaries if still required by third-party APIs.
</description>

<acceptance_criteria>
- [x] Endpoint/address flows are mapped across config, runtime, DCS store, API startup, and harness code before edits begin
- [x] Internal API listen-address handling no longer depends on raw `String` values where a typed socket/address model is more appropriate
- [x] Internal DCS endpoint handling no longer carries unvalidated raw endpoint strings across runtime/store paths where a typed endpoint model is more appropriate
- [x] Any remaining string encoding is limited to explicit external boundaries and is justified by the target library/API contract
- [x] Tests are updated or added to cover the typed endpoint flow and any normalization/validation behavior introduced
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan

### Current flow map
- Config schema currently keeps raw network strings in two places: `src/config/schema.rs` stores `ApiConfig.listen_addr: String` and `DcsConfig.endpoints: Vec<String>`, and `src/config/parser.rs` forwards those values into the runtime config with only empty-string validation for DCS endpoints.
- Runtime startup keeps those raw strings alive until late call sites: `src/runtime/node.rs` binds the API listener with `TcpListener::bind(cfg.api.listen_addr.as_str())` and passes `cfg.dcs.endpoints.clone()` into every `EtcdDcsStore::connect(...)` call.
- The etcd store worker uses raw endpoint strings throughout its worker/session code because `src/dcs/etcd_store.rs` accepts `Vec<String>` and only re-encodes them when calling the external `etcd_client::Client::connect(...)` boundary.
- The HA E2E harness also carries raw endpoint strings deep into setup, then reparses them with `parse_http_endpoint` in `src/test_harness/ha_e2e/startup.rs` and `src/test_harness/ha_e2e/util.rs`, which confirms the model is still primitive well past config parsing.

### Typed model
- Introduce typed network address models in the config domain and export them from `src/config/mod.rs`.
- Model the API listen address as `std::net::SocketAddr`, because the runtime and harness bind TCP listeners and never need a looser representation internally.
- Model DCS endpoints as a dedicated HTTP endpoint newtype backed by a parsed URL rather than `String`, because etcd requires URL-style endpoints while the harness still needs structured access to host/port information for loopback proxy targets.
- Keep string encoding only at explicit external boundaries:
  - TOML/serde input remains string-based in input structs.
  - `etcd_client::Client::connect(...)` still receives `Vec<String>` produced from the typed DCS endpoints right before the call.
  - Any CLI or JSON fixture text remains string literals at the outermost input/rendering edge.

### Implementation steps
- Add a small typed endpoint module under `src/config/` or alongside schema types with:
  - parsing from config strings into `SocketAddr` for API listen addresses,
  - parsing from endpoint strings into an owned HTTP endpoint newtype for DCS,
  - validation that DCS endpoints are HTTP URLs with an explicit host and port,
  - explicit helpers for string conversion at external client/process boundaries and for extracting a loopback `SocketAddr` only in harness paths that require proxy targets,
  - `Display` / string conversion helpers only where external APIs still require strings.
- Refactor `src/config/schema.rs` runtime structs so:
  - `ApiConfig.listen_addr` becomes `SocketAddr`,
  - `DcsConfig.endpoints` becomes `Vec<...typed endpoint...>`,
  - `RuntimeConfigInput` stops reusing the typed runtime `DcsConfig` and instead gains a dedicated `DcsConfigInput` with raw `Vec<String>` endpoints so the serde boundary stays explicit.
- Refactor `src/config/parser.rs` to normalize and validate both address types during parsing instead of leaving validation to later runtime or harness code.
  - Replace the current DCS empty-string loop with parse-based validation.
  - Parse the default API listen address through the same typed normalization path instead of storing it as raw text.
- Refactor `src/runtime/node.rs` to consume typed values directly.
  - Bind the API listener with the typed socket address.
  - Pass typed DCS endpoints into `EtcdDcsStore::connect(...)`.
  - Update `RuntimeError::ApiBind` to store a typed address or to stringify only at error formatting time.
- Refactor `src/dcs/etcd_store.rs` so its internal state carries typed DCS endpoints and converts to `Vec<String>` only inside the etcd client connect helper.
  - Update worker/session helper signatures from `&[String]` to slices of the new endpoint type.
  - Preserve existing behavior around reconnects, health state, and errors.
- Refactor harness/runtime-config helpers that currently assume strings.
  - Update `src/test_harness/runtime_config.rs` sample builders and helper methods to construct typed endpoint values.
  - Update `src/test_harness/ha_e2e/startup.rs` to use typed DCS endpoints directly when creating proxies instead of reparsing endpoint strings.
  - Remove or narrow `parse_http_endpoint` in `src/test_harness/ha_e2e/util.rs` if the typed endpoint makes it redundant.
  - Keep `src/test_harness/etcd3.rs` process-spawn and etcd-client call sites string-based where they are true external boundaries, but stop carrying those raw strings inside runtime config objects or proxy-target derivation logic.
- Update unit and integration tests to cover:
  - config parsing success for valid typed API and DCS endpoints,
  - config parsing failure for malformed API listen addresses and malformed DCS endpoint URLs,
  - config parsing failure for DCS endpoints that are syntactically URLs but still invalid for this runtime model because they omit a port or use an unsupported scheme,
  - runtime/harness helper behavior that now consumes typed addresses without reparsing,
  - existing etcd store and API worker tests impacted by signature/type changes.
- Update docs for the refactor if any developer-facing docs describe config/address handling. The user requested an `update-docs` skill for docs updates, but that skill is not present in the current session, so doc work will need to be done directly in-repo if execution reaches that phase.

### Verification plan
- Run `make check`.
- Run `make test`.
- Run `make test-long`.
- Run `make lint`.
- Confirm any touched docs are current and remove stale wording if the refactor changes documented internals.

NOW EXECUTE
