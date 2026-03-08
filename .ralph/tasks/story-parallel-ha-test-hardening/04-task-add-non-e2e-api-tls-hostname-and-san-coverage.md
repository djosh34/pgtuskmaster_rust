## Task: Add Non-E2E API TLS Hostname And SAN Coverage <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Strengthen TLS transport correctness tests for API hostname and SAN validation without pushing that complexity into the real HA end-to-end harness. The higher-order goal is to cover important certificate-validation edge cases where they belong, in focused transport/security tests, while keeping the HA e2e harness simpler and parallel-friendly.

This task is intentionally not an HA e2e TLS task. The user explicitly said the SAN/hostname cases should exist, but should not complicate the full e2e setup. This task should respect that boundary.

**Scope:**
- Extend the existing API worker TLS/security tests and TLS fixture helpers to cover more hostname/SAN edge cases in a focused, non-e2e way.
- Keep the work centered in `src/api/worker.rs`, `src/tls.rs`, and `src/test_harness/tls.rs` unless additional fixture helpers are needed elsewhere.
- Candidate coverage areas include:
- certificate valid for DNS name but client connects by IP
- certificate valid for IP SAN but client connects by DNS name
- certificate containing multiple SAN entries where one should match and another should not
- explicit mismatch across `localhost`, `127.0.0.1`, and `::1` where supported by the current test surface
- server-name validation behaviour for expected and unexpected names
- Preserve the current decision that these are transport-layer correctness tests, not full HA topology tests.
- Keep CLI work out of scope.

**Context from research:**
- There is already API TLS coverage in `src/api/worker.rs`, including wrong CA, hostname mismatch, expiry, and mTLS trusted/untrusted client behaviour.
- The current hostname coverage is still relatively narrow and does not deeply probe SAN shape mismatches.
- The real HA harness currently hardcodes API TLS disabled, and the user explicitly does not want SAN/hostname edge cases turned into heavy e2e setup burden.

**Expected outcome:**
- TLS hostname validation is covered more thoroughly where it belongs, in focused transport tests.
- The suite gains confidence around SAN/hostname mismatches without making HA e2e scenarios harder to maintain.
- The separation of concerns remains clean: transport correctness here, HA orchestration elsewhere.

</description>

<acceptance_criteria>
- [ ] Extend the TLS fixture and/or API worker test helpers so hostname and SAN mismatch cases can be expressed clearly and deterministically.
- [ ] Add focused non-e2e tests for DNS-name-versus-IP mismatch cases.
- [ ] Add focused non-e2e tests for positive SAN matching cases so the suite proves both rejection and acceptance paths.
- [ ] Where practical with current rustls fixtures, include coverage for `localhost`, `127.0.0.1`, and IPv6 loopback naming differences rather than only a single `not-localhost` mismatch.
- [ ] Keep these tests out of the HA e2e harness unless a future separate task intentionally expands HA TLS coverage.
- [ ] Reuse or extend `src/test_harness/tls.rs` instead of open-coding certificate generation logic inside individual tests.
- [ ] The added tests remain small, deterministic, and parallel-safe.
- [ ] The implementation does not touch CLI code.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts long-running test selection: `make test-long` — passes cleanly
</acceptance_criteria>
