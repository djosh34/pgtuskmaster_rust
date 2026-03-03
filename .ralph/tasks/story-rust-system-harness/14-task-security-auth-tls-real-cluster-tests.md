---
## Task: Implement security auth TLS validation tests in real cluster runs <status>done</status> <passes>true</passes> <passing>true</passing> <priority>high</priority>

<blocked_by>10-task-test-harness-namespace-ports-pg-etcd-spawners,13-task-e2e-multi-node-real-ha-loops-scenario-matrix</blocked_by>

<description>
**Goal:** Verify auth and TLS behavior under real deployment conditions.

**Scope:**
- Add test scenarios for node-to-node auth, client auth, TLS required/optional/disabled modes, invalid/expired certs, wrong CA, and endpoint role permissions.
- Ensure failures are explicit and do not silently downgrade security.

**Context from research:**
- Plan includes dedicated security/auth/TLS matrix as mandatory quality gate.

**Expected outcome:**
- Security behavior is proven by real-system tests and regressions are detectable.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Positive and negative auth/TLS cases are covered with real certificates and network connections.
- [x] API role permissions are tested for allow/deny behavior.
- [x] Security tests run in parallel-safe harness namespaces.
- [x] Run `make check`.
- [x] Run `make test`.
- [x] Run `make lint`.
- [x] Run `make test-bdd`.
- [x] On any failure, create `$add-bug` tasks including cert/config artifacts and reproductions.
</acceptance_criteria>

<execution_plan>
## Detailed Implementation Plan

1. Baseline and guardrails
- [x] Confirm current task dependencies (`10`, `13`) remain done/passing and capture current workspace state.
- [x] Keep strict constraints active through all edits: no unwrap/expect/panic, no skipped/optional real-binary tests, and no lint allows.
- [x] Implement this task in three phases: authz/runtime changes, TLS transport changes, then real-network security tests.

2. API auth role model (allow/deny endpoint behavior)
- [x] Extend `ApiWorkerCtx` with explicit role-token policy support (`read` + `admin`) while preserving existing `security.auth_token` behavior as a backward-compatible default.
- [x] Add endpoint-role classification (read-only endpoints vs privileged write endpoints) and enforce:
- [x] valid token + sufficient role => allowed,
- [x] missing/invalid token => `401`,
- [x] valid token with insufficient role => `403`.
- [x] Keep existing BDD semantics stable for legacy `auth_token` callers.

3. TLS transport enforcement in API worker
- [x] Add transport mode handling for API request acceptance: `Disabled`, `Optional`, and `Required`.
- [x] Add TLS acceptor wiring in `ApiWorkerCtx` so tests can inject real server TLS config/certs.
- [x] Implement optional-mode detection that accepts both plaintext HTTP and TLS client-hello traffic without silent downgrade on failed TLS handshakes.
- [x] Ensure required-mode plaintext attempts fail explicitly and do not proceed as plaintext requests.

4. Reusable TLS/auth harness helpers
- [x] Upgrade `src/test_harness/tls.rs` from placeholder-only structs to include reusable certificate material helpers for tests (CA/server/client paths + mode metadata) while preserving strict error handling.
- [x] Upgrade `src/test_harness/auth.rs` with explicit role-token helper utilities used by API security tests.

5. Real-certificate test matrix in API worker tests
- [x] Add dedicated security tests under `src/api/worker.rs` using real local network sockets and generated X.509 cert/key material.
- [x] Use `NamespaceGuard` per test for parallel-safe isolated cert/artifact directories.
- [x] Cover TLS modes:
- [x] disabled mode (plaintext success, TLS client rejected),
- [x] optional mode (both plaintext and TLS succeed),
- [x] required mode (TLS success, plaintext rejected).
- [x] Cover certificate failures:
- [x] wrong trust anchor (wrong CA) handshake failure,
- [x] invalid certificate identity (hostname mismatch) handshake failure,
- [x] expired certificate handshake failure.
- [x] Cover mTLS/node-auth behavior:
- [x] valid client cert allowed for required-client-auth server,
- [x] missing or untrusted client cert rejected.
- [x] Cover auth flows:
- [x] node/client bearer auth positive and negative paths,
- [x] role allow/deny for endpoint permissions (read token cannot perform admin endpoints, admin token can).

6. Keep existing integration BDD compatibility
- [x] Update any impacted API test scaffolding in `tests/bdd_api_http.rs` only as needed for compile/behavior compatibility.
- [x] Preserve existing test intent while adding assertions only when behavior has intentionally changed.

7. Verification and failure protocol
- [x] Run targeted security/API tests first until stable.
- [x] Run required gates sequentially: `make check`, `make test`, `make test-bdd`, `make lint`.
- [x] If any gate fails, create `$add-bug` task(s) with exact repro command, failing assertion, and cert/config artifact paths.

8. Completion bookkeeping
- [x] Tick all acceptance checkboxes with evidence after gates are green.
- [x] Update task header tags to done/passes true and set `<passing>true</passing>`.
- [x] Run `/bin/bash .ralph/task_switch.sh`.
- [x] Commit all changed files (including `.ralph` updates) with:
- [x] `task finished 14-task-security-auth-tls-real-cluster-tests: <summary + gate evidence + implementation challenges>`
- [x] Append durable learning(s) to `AGENTS.md`.
</execution_plan>

<evidence>
- Targeted security test run: `cargo test --lib api::worker::tests::security -- --nocapture`
- Required gates (all passing, sequential): `CARGO_BUILD_JOBS=1 make check`, `CARGO_BUILD_JOBS=1 make test`, `CARGO_BUILD_JOBS=1 make test-bdd`, `CARGO_BUILD_JOBS=1 make lint`
- Security test coverage location: `src/api/worker.rs` (`security_*` tests) using namespace-backed cert artifacts via `NamespaceGuard` + `write_tls_material(...)`
</evidence>

NOW EXECUTE
