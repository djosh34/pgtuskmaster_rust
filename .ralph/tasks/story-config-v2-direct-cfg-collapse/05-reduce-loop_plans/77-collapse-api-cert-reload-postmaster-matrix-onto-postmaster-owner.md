## Plan: Collapse API Cert Reload Postmaster Matrix Onto Postmaster Owner

### Why this is the next verified reduction target

The previously attached plan `76-collapse-ha-test-config-renderers-onto-config-v2-test-support.md` is stale in the current tree: the config-v2 TOML helper promotion and HA/observer call-site reuse it described are already present.

The next live boundary problem is a duplicated test matrix around managed-postmaster reload:

- `src/process/postmaster.rs` already owns the runtime contract for managed postmaster lookup, data-dir verification, and `SIGHUP` delivery.
- `src/api/worker.rs` delegates certificate reload straight into that postmaster owner, but its tests still rebuild the same postmaster fixture matrix for success, missing pid, stale pid, and data-dir mismatch.
- The shared fake managed-postmaster harness is already centralized in `src/process/test_support.rs`, so the remaining waste is not fixture implementation anymore; it is test ownership.

### Current overlap verified in code

- `src/api/worker.rs:58-84` shows the production boundary: `ApiServerTransport::reload_certificates(...)` reloads API TLS state, then calls `reload_managed_postmaster(...)`, then projects the result into `ReloadCertificatesResponse`.
- `src/process/postmaster.rs:145-205` already owns the same postmaster semantics under test:
  - `reload_managed_postmaster(...)`
  - `lookup_managed_postmaster(...)`
  - `signal_managed_postmaster(...)`
- `src/process/postmaster.rs:493-640` already tests the postmaster owner for:
  - missing `postmaster.pid`
  - stale pid
  - data-dir mismatch
  - successful `SIGHUP` delivery
- `src/api/worker.rs:506-695` repeats those same runtime fixture states through HTTP:
  - success path with signal log assertion
  - missing pid failure
  - stale pid failure
  - data-dir mismatch failure
- `src/process/test_support.rs:1-161` already provides the shared fake-postmaster process, pid-file writer, readiness wait, and signal-log wait that both modules reuse, so there is no reason for the API layer to also own the postmaster edge-case matrix.

### Execution plan

1. Shrink API-worker tests to API-owned behavior only.
   - Keep coverage for behavior that only `src/api/worker.rs` owns:
     - HTTPS transport reload succeeds and returns the projected JSON payload.
     - API reload failure short-circuits before PostgreSQL signaling.
     - HTTPS runtime / HTTP config mismatch returns the API-specific transport error.
     - auth and handler wiring still return the expected HTTP status/body shape.
   - Replace the repeated postmaster-specific failure trio with, at most, one API-level regression test that proves a postmaster error is surfaced as `500` with the `"postgres certificate reload failed"` prefix.

2. Move postmaster semantic coverage fully onto the postmaster owner.
   - Expand `src/process/postmaster.rs` tests only if needed to cover any assertion currently unique to the API path.
   - Keep the detailed ownership of:
     - missing pid details
     - stale pid details
     - mismatched managed data dir details
     - successful `SIGHUP` delivery and pid reporting
   - Do not duplicate those semantics again in API tests.

3. Keep the shared fake-postmaster harness unchanged unless execution reveals one small missing helper.
   - Reuse `src/process/test_support.rs` as-is if possible.
   - Only add a small helper there if it lets both API and postmaster tests express one common assertion without reintroducing wrapper layers.

4. Validate the slice.
   - Run `cargo fmt`.
   - Run `bash .ralph/git_diff_lines_since.sh` and confirm the diff is net-negative.
   - Run `make check`.
   - Run `make lint`.
   - Run `make test`.
   - Run `make test-long`.

### Guardrails

- Do not move postmaster runtime logic into the API layer; the reduction is about deleting duplicated tests, not changing the owner.
- Do not introduce a new cross-module DTO or test-only abstraction just to describe postmaster failure cases.
- Keep at least one API-level failure-path test so the handler still proves it maps postmaster errors into the expected HTTP response shape.
- If execution shows the API route is currently the only place asserting some response-field projection that postmaster tests cannot cover, preserve that API assertion and only delete the duplicated fixture cases.

### Expected yield

- Delete the repeated success/failure postmaster fixture matrix from `src/api/worker.rs`.
- Keep postmaster edge cases tested once, in the module that owns `reload_managed_postmaster(...)`.
- Leave `src/api/worker.rs` tests focused on HTTP/API behavior rather than process semantics.
- Produce a net-negative slice without changing runtime behavior.

NOW EXECUTE
