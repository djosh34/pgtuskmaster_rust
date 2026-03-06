## Bug: Contributor docs claims drift from implementation and contract <status>done</status> <passes>true</passes>

<description>
Two contributor-facing documentation defects were found during slice verification.

1) `docs/src/contributors/harness-internals.md` states readiness is checked by connecting to the client port. Current etcd harness startup does port-connect checks per member (`wait_for_port`) and then performs an etcd KV round-trip readiness probe (`Client::connect` + put/get/delete) before considering the cluster ready. The doc wording is stale/incomplete.

2) `docs/src/contributors/docs-style.md` claims every contributor deep-dive chapter must include a minimum shape, including failure behavior, tradeoffs/sharp edges, and evidence pointers. At least `docs/src/contributors/codebase-map.md` does not currently satisfy that minimum contract as written.

Please explore and research the relevant docs and code first, then fix the docs so claims are accurate and internally consistent.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
