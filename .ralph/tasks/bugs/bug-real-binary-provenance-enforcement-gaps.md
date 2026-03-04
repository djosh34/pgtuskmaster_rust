---
## Bug: Real-binary provenance enforcement gaps in installers and harness <status>not_started</status> <passes>false</passes>

<description>
Real-binary tooling currently enforces existence/executability but not strong provenance at runtime.

Detected during skeptical audit of `tools/install-postgres16.sh`, `tools/install-etcd.sh`, and `src/test_harness/binaries.rs`.

Key gaps:
- `src/test_harness/binaries.rs` validates only `exists + regular file + executable`; it does not verify ownership, canonical target policy, version, or checksum.
- Symlinked binaries are accepted implicitly via `fs::metadata`, so `.tools/*` can point outside expected roots without rejection.
- `tools/install-postgres16.sh` trusts package-manager state + version string for major `16.x`, but does not pin exact package provenance in-repo and links `.tools/postgres16/bin/*` to mutable system paths.
- Installer scripts invoke privileged/sensitive commands by name (`sudo`, `dnf`, `curl`, etc.) via PATH resolution and do not harden PATH trust assumptions.

Please explore and research the full codebase first, then implement fail-closed provenance verification for real-binary prerequisites and installer trust boundaries.
</description>

<acceptance_criteria>
- [ ] Add a fail-closed real-binary provenance check path (canonical-path policy + version assertions + checksum/signature verification strategy) used by real-binary tests.
- [ ] Add a deterministic negative-control test/fixture proving substituted `.tools` binaries fail early and loudly.
- [ ] Add executable-trace evidence (for example `strace execve`) to prove runtime uses expected `.tools` paths.
- [ ] Document trust assumptions and operator verification commands in contributor docs.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
