---
## Task: Simplify config semantics and make secure mTLS the documented default <status>not_started</status> <passes>false</passes>

<description>
Rework the config contract and documentation so the supported settings make operational sense and the recommended setup is secure by default.

The agent must explore the current config model, docs, and runtime usage first, then implement the needed cleanup. The intended direction is:
- the documented recommended setup uses TLS/mTLS by default for PostgreSQL, etcd, and the API
- password-based auth remains supported where required, but it is not the recommended primary example
- certificate/key/CA file inputs are the main documented configuration approach
- confusing or nonsensical config surfaces should be removed or redesigned
- configuration fields should reflect real runtime ownership instead of asking the user to provide values that should be derived from cluster state or coordination data

This task should explicitly revisit unclear fields and semantics such as rewind source addressing and any other config that does not make sense in a greenfield HA system.

Specific expectations from product direction:
- the secure recommended examples in docs should be mTLS/TLS-first for PostgreSQL, etcd, and the API
- the recommended config path should use CA/cert/key files for all three surfaces
- password auth for PostgreSQL, etcd, and the API must still be supported where needed, but should be documented as a supported alternative rather than the main recommendation
- the docs should stop recommending insecure or internally inconsistent combinations
- operator-facing config should not ask for static topology values that the system should infer at runtime from current cluster state

This task should treat suspicious fields as design problems, not merely documentation problems. If a field or concept does not make sense for the product, the agent should remove or redesign it rather than polishing the explanation.

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
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
