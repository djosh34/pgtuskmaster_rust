---
## Task: Expand TLS adversarial e2e tests for certificate validation hardening <status>not_started</status> <passes>false</passes>

<blocked_by>22-task-ha-admin-api-read-write-surface</blocked_by>

<description>
**Goal:** Add skeptical TLS tests that actively try to break API and cluster TLS trust, including wrong certs and expired certs, and prove they are rejected.

**Scope:**
- Extend TLS test harness fixtures to generate invalid/expired/mismatched cert chains and client cert combinations.
- Add negative-path tests for API TLS/mTLS acceptance logic and handshake failures.
- Add cluster integration tests to ensure bad cert material is rejected across node/API communication paths.
- Verify valid cert flows still work so hardening does not break expected operation.

**Context from research:**
- Existing TLS support and helpers exist (`src/test_harness/tls.rs`, `src/api/worker.rs`) but adversarial coverage is limited.
- Requirement explicitly asks whether wrong/expired certs are accepted and to test break attempts.
- These checks are high-signal for production readiness and regression prevention.

**Expected outcome:**
- TLS tests prove the system rejects invalid trust material and only accepts properly valid cert configurations.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist completed with concrete module requirements: `src/test_harness/tls.rs` (invalid cert fixtures), `src/api/worker.rs` TLS auth paths (if fixes needed), API TLS test files (BDD/unit/integration), real TLS integration/e2e scenarios validating wrong CA, wrong SAN, expired cert, and client-cert mismatch rejection
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
