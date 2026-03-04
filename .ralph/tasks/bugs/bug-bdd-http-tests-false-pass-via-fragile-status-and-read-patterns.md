---
## Bug: BDD HTTP tests false-pass via fragile status and read patterns <status>done</status> <passes>true</passes> <passing>true</passing>

<description>
BDD HTTP contract tests use weak status matching and response-read behavior that can hide protocol/handler regressions or induce hangs.

Detected during review of plan section 2.4 against:
- tests/bdd_api_http.rs:247-250, 298-301, 317-320, 336-339, 355-358, 407-410, 434-437, 455-458, 498-501, 539-542, 577, 598-601
  (status assertions use `contains("...")` instead of exact status code checks)
- tests/bdd_api_http.rs:241-245, 293-296, 312-315, 330-334, 350-353, 402-405, 429-432, 450-453, 493-496, 534-537, 573-575, 593-596
  (response reads rely on `read_to_end`, which depends on connection-close semantics)

Ask the fixing agent to explore and research the codebase first, then implement robust HTTP response assertions.

Expected direction:
- Parse status code as an exact numeric value and assert exact expectations.
- Replace connection-close-dependent `read_to_end` flow with bounded HTTP response reads that honor headers/content length (or a tested helper used by all cases).
- Keep tests black-box and contract-oriented; avoid overfitting to current low-level socket behavior.
- Add one negative test proving a wrong status code cannot satisfy assertions.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan

### 0) Context / root cause

The integration test `tests/bdd_api_http.rs` is intended to be a black-box, contract-style check of the node API. Today it has two issues that can cause false confidence:

- **Weak status assertions**: it asserts status via `status_line.contains("200")`-style matching on the HTTP status line string, instead of asserting the exact numeric status code.
- **Connection-close-dependent reads**: it reads the full response via `read_to_end`, which implicitly depends on the server closing the TCP connection (or the client hanging if it does not).

The API server currently sets `Content-Length` and `Connection: close` for every response, so the tests pass today; the bug is that the tests are fragile and can either false-pass (status matching) or hang (read-to-end framing) if the server contract evolves.

### 1) Research / constraints (already known)

- The API server currently always emits `Content-Length: ...` and `Connection: close` on responses (and does one request per socket), but tests should not rely on EOF/close to make progress.
- The workspace already has multiple ad-hoc HTTP response parsers; prefer one canonical helper per test module (and reuse between `tests/` and `src/` test helpers if practical).
- Repo policy: no `unwrap()` / `expect()` / `panic!()` in test or non-test code. Use `Result<_, WorkerError>` + `?` and keep panics limited to `assert_*` macros.

### 2) Add a robust request/response helper (bounded, framed, strict)

The root cause is duplicated, ad-hoc parsing + `read_to_end`. Fix it by introducing a canonical “send request + step worker + read framed response” helper and use it everywhere in the contract-style tests.

Do this in both places that currently implement `extract_status_and_body` + `read_to_end`:
- `tests/bdd_api_http.rs` (integration/BDD contract tests)
- `src/api/worker.rs` test helpers (unit tests; same fragility)

Helper requirements:

- No `read_to_end` anywhere in these tests/helpers.
- Read bytes incrementally until header terminator `\r\n\r\n` is present; enforce a hard `HEADER_LIMIT`.
- Parse the status line strictly:
  - Require `HTTP/1.1 <3-digit> ...` (exactly 3 ASCII digits)
  - Convert to `u16` and enforce `100..=599`
- Parse headers deterministically:
  - Split each header line on the first `:`
  - Case-insensitive match for `Content-Length`
  - `Content-Length` must be present and parseable to `usize`
- After discovering `Content-Length`, read exactly that many bytes for the body; enforce `MAX_BODY_BYTES` and `MAX_RESPONSE_BYTES`.
- Add an overall IO timeout (suggest `Duration::from_secs(2)` per request; adjust upward if CI is slow) using `tokio::time::timeout`, so regressions fail fast instead of hanging indefinitely.
- If the stream hits EOF before the full framed body is received, return an error (fail closed).

Suggested response shape:

- `struct TestHttpResponse { status_code: u16, headers: Vec<(String, String)>, body: Vec<u8> }`
- `fn header_value(headers: &[(String, String)], name: &str) -> Option<&str>` (case-insensitive lookup; useful for `Content-Type`, etc.)

Suggested helper signatures (exact names flexible):
- `async fn read_http_response_framed(stream: &mut (impl AsyncRead + Unpin), timeout: Duration) -> Result<TestHttpResponse, WorkerError>`
  - Works for both `TcpStream` and `tokio_rustls::client::TlsStream<TcpStream>`.
- `async fn send_plain_request(ctx: &mut ApiWorkerCtx, request_head: &str, body: Option<&[u8]>) -> Result<TestHttpResponse, WorkerError>`
- `async fn send_tls_request(...same...) -> Result<TestHttpResponse, WorkerError>`

Constants to keep behavior deterministic:

- `const HEADER_LIMIT: usize = 16 * 1024;`
- `const MAX_BODY_BYTES: usize = 256 * 1024;`
- `const MAX_RESPONSE_BYTES: usize = HEADER_LIMIT + MAX_BODY_BYTES;`
- `const IO_TIMEOUT: Duration = Duration::from_secs(2);`

### 3) Update all HTTP assertions to be strict (exact numeric status)

In `tests/bdd_api_http.rs`, update each site listed in the bug description:

- Replace `read_to_end` calls with the framed read helper.
- Replace `status_line.contains("XYZ")` with `assert_eq!(response.status_code, XYZ, "...")`.
- Keep existing body decoding checks (JSON decode / UTF-8 decode) but make them operate on `response.body`.

Also make each request explicitly include `Connection: close` to keep request behavior simple and explicit (even though reads no longer depend on it).

Apply the same strictness in `src/api/worker.rs` test helpers:
- Change `send_plain_request(...) -> (String, Vec<u8>)` into `-> TestHttpResponse` and update call sites.
- Change `send_tls_request(...) -> (String, Vec<u8>)` into `-> TestHttpResponse` and update call sites.
- Replace any status substring checks with numeric `assert_eq!(response.status_code, ...)`.

### 4) Add a negative regression test that proves “wrong status can’t pass”

Add one targeted test that fails under the old “contains” approach but passes with the new strict approach.

Recommended approach (no server needed; parser-only test):

- Add a `#[test]` (or `#[tokio::test]`) that calls the parsing helper on a **synthetic** raw HTTP response whose status code is invalid-but-contains a valid substring, e.g.:
  - status line `HTTP/1.1 1200 OK` with `Content-Length: 0`.
- Assert the parse fails because the status code is not exactly 3 digits / not in `100..=599`.

This demonstrates that a `contains("200")` style check would be dangerously permissive, while the strict helper fails closed.

### 5) Verification (required gates)

After implementation:

- Run `make check`
- Run `make test`
- Run `make test-long`
- Run `make lint`
- Store logs under `.ralph/evidence/bug-bdd-http-tests-false-pass-via-fragile-status-and-read-patterns/` (one file per command).

If all four are green:

- Set `<passes>true</passes>` in the task header.
- Update `<status>...` to the project’s “done” value (match other completed tasks).
- Run `/bin/bash .ralph/task_switch.sh` and proceed with commit/push flow as instructed.

NOW EXECUTE
