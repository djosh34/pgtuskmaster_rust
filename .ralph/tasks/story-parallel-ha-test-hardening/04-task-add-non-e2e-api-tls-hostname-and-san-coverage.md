## Task: Add Non-E2E API TLS Hostname And SAN Coverage <status>completed</status> <passes>true</passes>

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
- [x] Extend the TLS fixture and/or API worker test helpers so hostname and SAN mismatch cases can be expressed clearly and deterministically.
- [x] Add focused non-e2e tests for DNS-name-versus-IP mismatch cases.
- [x] Add focused non-e2e tests for positive SAN matching cases so the suite proves both rejection and acceptance paths.
- [x] Where practical with current rustls fixtures, include coverage for `localhost`, `127.0.0.1`, and IPv6 loopback naming differences rather than only a single `not-localhost` mismatch.
- [x] Keep these tests out of the HA e2e harness unless a future separate task intentionally expands HA TLS coverage.
- [x] Reuse or extend `src/test_harness/tls.rs` instead of open-coding certificate generation logic inside individual tests.
- [x] The added tests remain small, deterministic, and parallel-safe.
- [x] The implementation does not touch CLI code.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts long-running test selection: `make test-long` — passes cleanly
</acceptance_criteria>

## Execution Plan

### Planning notes locked in before execution

- Current research shows the TLS fixture gap is real and narrow: [src/test_harness/tls.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/tls.rs) currently builds leaf certificates through `generate_leaf_cert(common_name, dns_name, ...)`, which feeds a single DNS SAN into `CertificateParams::new(...)`. That means the fixture can express `localhost` and expiry/client-auth variations, but it cannot yet express IP SANs or mixed SAN sets.
- Current API transport coverage is already in the right place for this work: [src/api/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/worker.rs) has focused TLS tests for wrong CA, one hostname mismatch, expiry, and mTLS acceptance/rejection. The missing piece is broader SAN-shape coverage, not new HA orchestration coverage.
- The handshake helper in [src/api/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/worker.rs) validates certificates through the rustls `ServerName` passed into `TlsConnector::connect(...)`. The existing HTTP request-format helpers always send `Host: localhost`, so execution must avoid conflating HTTP Host headers with TLS server-name validation. If readability would otherwise suffer, execution should add an optional request-host helper, but TLS assertions should continue to hinge on the rustls server name rather than HTTP routing.
- This task should stay centered in [src/test_harness/tls.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/tls.rs), [src/api/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/worker.rs), and only touch [src/tls.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/tls.rs) if reusing the richer fixture there adds meaningful parity coverage for the production rustls builder.
- There is no `update-docs` skill available in this session. If execution finds shipped docs that must mention the new non-e2e TLS coverage, it should update them directly in-repo rather than inventing a substitute skill workflow.
- The worktree already contains unrelated `.ralph` modifications outside this task file. Execution should not revert or disturb those unrelated changes while implementing this task.

### Phase 1: Generalize the TLS test fixture so SAN cases are first-class and reusable

- [x] Model the server-certificate variants as a small named fixture set rather than a completely free-form builder, so the API tests stay readable and the fixture remains intentional.
  - [x] Keep explicit fixture entries for at least `localhost`-only, IPv4-loopback-only, and mixed loopback SAN coverage instead of making every worker test spell out raw SAN lists inline.
  - [x] Let the lower-level generator still accept SAN input data, but keep `build_adversarial_tls_fixture()` responsible for publishing the canonical cert variants the API tests will consume.
- [x] Extend [src/test_harness/tls.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/tls.rs) with an explicit SAN-aware certificate helper instead of continuing to hardcode one DNS name per server certificate.
- [x] Introduce a small test-only SAN representation for leaf generation that can express:
  - [x] DNS names such as `localhost`
  - [x] IPv4 addresses such as `127.0.0.1`
  - [x] IPv6 loopback `::1` where the current rcgen/rustls surface supports it cleanly in this repo
- [x] Refactor the current server-certificate generation path so it can build:
  - [x] the existing `localhost`-only server certificate used by current tests
  - [x] an IP-only server certificate for loopback-IP matching tests
  - [x] a mixed-SAN server certificate containing more than one valid alternative name so positive multi-SAN acceptance can be proven
  - [x] the existing expired server certificate without duplicating certificate-construction logic
- [x] Preserve the current client-certificate generation path for mTLS tests unless execution discovers that a richer SAN helper materially simplifies those tests too. This task is about server-name validation, so avoid unnecessary churn in client-auth fixtures.
- [x] Add or update fixture-level tests in [src/test_harness/tls.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/test_harness/tls.rs) so the richer adversarial fixture is validated directly instead of being trusted implicitly.
  - [x] Do not rely on “PEM differs” assertions to validate SAN behavior, because regenerated keys will make certificates differ even when SAN contents are wrong.
  - [x] Prefer asserting the exact intended SAN coverage either by inspecting the generated certificate contents or by using tightly scoped handshake/builder checks that uniquely prove each named fixture variant behaves as intended.
  - [x] Assert that the existing builder helpers still accept the updated fixture output.

### Phase 2: Tighten API worker test helpers around TLS server-name intent

- [x] Keep the existing `send_tls_request(...)` and `expect_tls_handshake_failure(...)` flow in [src/api/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/worker.rs) as the main transport test seam, because it already exercises the real listener and rustls client/server handshake.
- [x] Add only the smallest helper changes needed to express the new cases clearly.
  - [x] Default to leaving the HTTP `Host` header helpers alone, because the current assertions are about rustls `server_name` validation and not HTTP virtual-host routing.
  - [x] Only add a request-host override helper if a concrete test becomes ambiguous without it.
  - [x] Keep TLS pass/fail assertions tied to the rustls `server_name` argument, not to the HTTP Host header.
- [x] Reuse the existing `build_ctx(...)`, `configure_tls(...)`, `build_server_config(...)`, and `build_client_config(...)` test helpers instead of introducing a second parallel API-test harness.
- [x] Keep all new tests deterministic and parallel-safe by continuing to use per-test listeners, current-thread Tokio tests, and fixture-generated cert material only.

### Phase 3: Add focused non-e2e DNS-versus-IP mismatch coverage

- [x] Extend the API worker TLS failure coverage so it explicitly proves DNS/IP mismatch rejection in both directions.
- [x] Cover the case where the certificate is valid for a DNS SAN but the client validates against an IP server name.
  - [x] Use a `localhost`-only server certificate.
  - [x] Connect with `127.0.0.1` as the rustls server name and assert handshake failure.
- [x] Cover the case where the certificate is valid for an IP SAN but the client validates against a DNS server name.
  - [x] Use an IP-only loopback server certificate.
  - [x] Connect with `localhost` as the rustls server name and assert handshake failure.
- [x] Keep the existing wrong-CA and expired-certificate assertions, but split or reorganize the current combined test if that is needed to keep each failure story readable once the matrix grows.
- [x] Preserve at least one explicit unexpected-name case beyond DNS/IP shape mismatches, such as a non-matching DNS name against a `localhost` certificate, so the suite still covers a plain wrong-hostname rejection and not only type-shape mismatches.

### Phase 4: Add positive SAN matching coverage so the suite proves acceptance as well as rejection

- [x] Add focused success-path tests in [src/api/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/worker.rs) using richer fixture certificates rather than relying only on failure assertions.
- [x] Add a mixed-SAN positive case where one SAN entry matches and another does not.
  - [x] Start the API worker with a mixed-SAN server certificate that includes `localhost` and at least one loopback IP entry.
  - [x] Prove that a request succeeds when the client validates with `localhost`.
  - [x] Prove that a request also succeeds when the client validates with the included loopback IP entry.
- [x] Where practical with the current rustls and rcgen versions in this repo, add IPv6 loopback coverage rather than stopping at IPv4.
  - [x] Preferred outcome: include `::1` in the mixed-SAN or dedicated IP SAN fixture and prove success when the client validates with `::1`.
  - [x] Fallback outcome if the local crypto/test surface makes IPv6 SAN generation or validation impractical: keep the helper ready for IPv6-shaped SANs, retain strong `localhost` plus `127.0.0.1` coverage, and document the concrete blocker in the task execution notes instead of silently skipping the case.
- [x] Keep these success-path tests transport-layer focused: successful handshake plus a simple `/fallback/cluster` request returning `200` is sufficient proof. Do not expand into broader API authorization or HA behavior here.

### Phase 5: Preserve production-builder parity without moving the task into HA or CLI code

- [x] Reuse the richer fixture material to make sure the production rustls server-config builder still participates in at least one of the new SAN-aware scenarios.
- [x] Prefer proving this through [src/api/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/api/worker.rs) by configuring the server with `crate::tls::build_rustls_server_config(...)` and then performing a real TLS request against a SAN-rich certificate.
- [x] Only add direct unit coverage in [src/tls.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/tls.rs) if execution finds a fixture or builder edge that cannot be demonstrated cleanly through the existing API worker integration-style tests.
- [x] Keep CLI code entirely out of scope.

### Phase 6: Docs audit and required verification

- [x] Audit shipped docs for statements about API TLS or test coverage that would become stale after the new hostname/SAN cases land.
  - [x] Current planning research did not find a shipped doc page that enumerates this non-e2e API TLS test matrix, so execution should only change docs if it finds a concrete stale statement during implementation.
  - [x] Ignore `docs/draft/` unless execution is already editing a shipped counterpart and needs to remove clear duplication or stale text there as part of the same docs correction.
- [x] If docs do need updating, edit them directly in-repo because the requested `update-docs` skill is not available in this session.
- [x] Run the full required verification sequence after implementation:
  - [x] `make check`
  - [x] `make test`
  - [x] `make test-long`
  - [x] `make lint`
- [x] Only after all gates pass:
  - [x] mark the acceptance criteria and execution-plan checkboxes complete
  - [x] set `<passes>true</passes>`
  - [x] run `/bin/bash .ralph/task_switch.sh`
  - [x] commit all changes, including `.ralph` files, with the required `task finished [task name]: ...` commit message and explicit gate evidence
  - [x] push with `git push`

EXECUTED
