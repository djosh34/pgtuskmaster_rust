## Bug: HA API polls hang during no-quorum fail-safe observation <status>not_started</status> <passes>false</passes>

<description>
During `make test-long`, `e2e_no_quorum_enters_failsafe_strict_all_nodes` can hang indefinitely after etcd quorum loss.
Live debugging showed that all nodes had already become non-primary by SQL evidence, but every `GET /ha/state` call to the node APIs hung instead of returning a `FailSafe` phase snapshot.
The test harness needed a SQL-only fallback to keep the suite progressing, but the underlying bug is that HA API requests become unresponsive during the no-quorum window.

Explore and research the codebase first, then fix the runtime/API hang so HA state remains observable during no-quorum fail-safe transitions.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
