## Bug: Lone survivor can regain full quorum and stay writable after peers age out <status>not_started</status> <passes>false</passes>

<description>
The HA system currently allows a single surviving node from a multi-node cluster to degrade into a one-member `FullQuorum` cluster after the stopped peers age out of DCS membership.

This was detected from the failing HA docker scenario `ha_two_replicas_stopped_then_one_replica_restarted_restores_quorum`. In the captured run, `wait_for_no_operator_primary` first observed `sampled 1/2 discovered members` and correctly treated `pgtm primary` as unresolved, but a few seconds later the same node reported `discovered_member_count = 1`, `warnings = []`, and `pgtm primary` resolved that lone node as the operator-visible primary. The result is that the "lone online node is not treated as a writable primary" invariant is violated even though two peers were only stopped, not intentionally removed from cluster membership.

The code path appears to be:
- `src/dcs/state.rs` treats `member_slots.len() == 1` as quorum in `has_member_quorum`.
- `src/cli/status.rs` maps a sampled self-node to role `"primary"` whenever DCS trust is `FullQuorum`, the authority projection points at self, and the role intent is `Leader`.
- `src/cli/connect.rs` then resolves `pgtm primary` successfully because there are no blocking warnings once the dead peers have disappeared from the discovered member set.

Please explore and research the codebase first, then fix. The fix should make multi-node clusters fail closed when they temporarily collapse to a lone survivor after peer loss, instead of silently reclassifying themselves as a healthy one-node cluster. Review whether the correct source of truth is configured cluster size, explicit bootstrap topology, or another durable quorum baseline. The HA tests and CLI/operator-visible contracts should agree on that model.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
