---
## Task: Derive rewind source from the current primary instead of static config <status>not_started</status> <passes>false</passes>

<description>
Remove static rewind source addressing from the product and derive rewind behavior from current cluster state.

The agent must explore the current config schema, HA/runtime flow, DCS/member-state usage, process dispatch, and rewind execution path first, then redesign the rewind source contract so it matches how HA should actually work.

Intended direction:
- a node should rewind from the current primary/leader, not from a permanently configured host/port
- static config fields for rewind source host/port should be removed if they are only compensating for missing runtime derivation
- runtime logic should derive the rewind target from the current cluster leader/member information rather than operator-entered topology fields
- config should contain only the reusable connection/auth/TLS material needed to connect securely to the eventual rewind source
- docs and examples should explain rewind in terms of cluster-state discovery rather than a fixed peer endpoint

This is not only a docs or schema cleanup. The agent must update the runtime behavior and surrounding contracts so rewind source selection is owned by the cluster state model rather than by static node config.

The agent should use parallel subagents after exploration for runtime/HA changes, config and fixture migration, and doc updates.
</description>

<acceptance_criteria>
- [ ] Static rewind source host/port configuration is removed or otherwise no longer required as a user-facing contract
- [ ] Rewind source selection is derived from current primary/leader discovery at runtime
- [ ] Rewind configuration is reduced to the auth/TLS material and other genuinely reusable settings needed to connect securely
- [ ] Tests and fixtures cover the runtime-derived rewind source behavior
- [ ] Docs and examples describe rewind in terms of current-primary discovery rather than fixed config endpoints
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
