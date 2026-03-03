---
## Bug: test-bdd full-suite command hangs in real HA e2e <status>not_started</status> <passes>false</passes>

<description>
After updating `make test-bdd` to run `PGTUSKMASTER_REQUIRE_REAL_BINARIES=1 cargo test --all-targets -- --include-ignored`, the verification run did not complete within an extended runtime window (over 15 minutes observed on 2026-03-03).

Detection details:
- `make test-bdd` started and progressed through unit + real-binary tests.
- Output stalled at `ha::e2e_multi_node::e2e_multi_node_real_ha_scenario_matrix has been running for over 60 seconds` and never completed during the observed window.
- Active `etcd` and `postgres` child processes for the e2e namespace remained running until manual interruption.

Please explore and research the codebase first to identify whether this is a genuine deadlock/hang or an expected runtime regression, then implement the fix.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
