## Plan: Collapse API HTTPS Runtime Wrapper Stack

### Why this is the next reduction target

The API HTTPS path still keeps the same runtime value wrapped in multiple small layers even though one owner already has all the information it needs.

- `src/tls.rs:29-67` builds the same HTTPS runtime in three steps: `build_api_server_transport_v2(...)` matches on `ApiTransportV2`, `build_api_server_config_v2(...)` builds the `Arc<rustls::ServerConfig>`, and `build_api_rustls_config_v2(...)` immediately wraps that into `RustlsConfig`.
- `src/api/worker.rs:42-97` adds a second wrapper stack around that same value: `ApiTlsRuntime`, `ApiServerTransport::Https(ApiTlsRuntime)`, `ApiTlsCertificateReloadHandle::Https { server_config }`, and `ApiReloadCertificatesHandle`.
- `src/api/worker.rs:69-83`, `src/api/worker.rs:239-253`, and `src/api/worker.rs:312-318` then unwrap those layers right back into the same `RustlsConfig` only to reload or serve it.

That is the wrong ownership boundary. `tls.rs` should keep owning PEM/rustls construction, and the API worker should keep one transport enum that directly carries the runtime it serves and reloads.

### Current overlap already verified

- `src/tls.rs:29-37` and `src/api/worker.rs:48-50` both encode the same HTTP-vs-HTTPS transport split.
- `src/tls.rs:64-67` exists only to convert the result of `build_api_server_config_v2(...)` into `RustlsConfig`, while `src/api/worker.rs:248-249` and `src/api/worker.rs:75-80` both immediately need that `RustlsConfig`.
- `src/api/worker.rs:59-67` clones the `RustlsConfig` out of `ApiServerTransport` into `ApiTlsCertificateReloadHandle`, and `src/api/worker.rs:92-97` adds another wrapper on top even though the route state already stores `ApiServerTransport`.

### Execution plan

1. Make `ApiServerTransport` own the HTTPS runtime directly.
   - Delete `ApiTlsRuntime`.
   - Change `ApiServerTransport::Https(...)` to hold `RustlsConfig` directly, ideally as a named field for readability.
   - Replace the extra `build_api_rustls_config_v2(...)` layer with one remaining HTTPS runtime constructor.

2. Collapse the reload wrappers onto the transport owner.
   - Delete `ApiTlsCertificateReloadHandle`.
   - Either delete `ApiReloadCertificatesHandle` entirely and make `ApiServerTransport` expose a direct reload method, or reduce it to a thin wrapper around `ApiServerTransport` without a second HTTP-vs-HTTPS enum.
   - Reuse the same `RustlsConfig` instance for both `bind_rustls(...)` and certificate reload so the runtime stays hot-reloadable.

3. Retarget API runtime wiring and tests to the smaller surface.
   - Update `ApiRuntimeCtx::new`, `run`, and the test helpers in `src/api/worker.rs` to construct the transport through the surviving owner.
   - Update `src/tls.rs` tests to target the remaining real builder only.
   - Delete assertions or helper code whose only purpose was preserving the removed wrapper layers.

4. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and require the diff to remain net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Keep PEM parsing, verifier construction, and rustls-specific loading in `src/tls.rs`; this slice is about deleting wrapper layers, not moving TLS internals into the API worker.
- Do not add a replacement runtime wrapper just to preserve the old call graph.
- If axum-server or route-state ownership proves that the serving and reload paths cannot share the same `RustlsConfig` handle without adding another wrapper layer, switch this plan back to `TO BE VERIFIED` and stop.

### Expected yield

- Delete one dedicated HTTPS runtime wrapper struct and one dedicated reload-handle enum.
- Remove a pure pass-through TLS builder that only rewraps `Arc<ServerConfig>` into `RustlsConfig`.
- Shorten the API runtime construction/reload path so one transport owner carries the only HTTPS runtime value.

NOW EXECUTE
