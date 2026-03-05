---
## Task: Simplify config semantics and make secure mTLS the documented default <status>not_started</status> <passes>false</passes>

<description>
Rework the config contract and documentation so the supported settings make operational sense and the recommended setup is secure by default.

The agent must explore the current config model, docs, and runtime usage first, then implement the following fixed product decisions:
- the documented recommended setup uses TLS/mTLS by default for PostgreSQL, etcd, and the API
- the recommended config path uses CA/cert/key files for PostgreSQL, etcd, and the API
- password auth for PostgreSQL, etcd, and the API remains supported where needed, but it is not the recommended primary example
- docs and examples must stop recommending insecure or internally inconsistent combinations
- confusing or nonsensical config surfaces must be removed or redesigned
- configuration fields must reflect real runtime ownership instead of asking the user to provide values that should be derived from cluster state or coordination data
- suspicious config concepts are design problems to remove or redesign, not prose problems to explain away

The agent should use parallel subagents after exploration for runtime/config cleanup, validation/tests, and operator-doc updates.
</description>

<acceptance_criteria>
- [ ] Operator docs recommend a secure TLS/mTLS-first configuration for PostgreSQL, etcd, and the API
- [ ] Password-based auth remains supported where required without becoming the main recommended path
- [ ] Certificate/key/CA file configuration is the primary documented secure setup
- [ ] Confusing or nonsensical config fields and semantics are removed or redesigned
- [ ] Config semantics are aligned with real HA/runtime ownership rather than avoidable manual topology inputs
- [ ] Secure examples and docs do not recommend inconsistent TLS/auth combinations
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
