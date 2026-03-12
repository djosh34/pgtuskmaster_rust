## Bug: HA total-outage recovery can stall leader election or briefly diverge on leader after final rejoin <status>not_started</status> <passes>false</passes>

<description>
The ultra-long HA feature `ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins` has at least two real failure modes after a full cluster outage.

Mode 1: after restarting only `node-a` and `node-b`, quorum and membership recover but both nodes can remain operator-visible `unknown`, with `no_primary:LeaseOpen`, `Idle(AwaitingLeader)`, and no promotion ever happening before the recovery deadline.

Mode 2: after the two-node restore succeeds and `node-c` rejoins, operator-visible routing can still briefly fail because sampled nodes disagree on the leader (`<none>` vs `node-b`), causing `pgtm primary` to reject resolution even though the cluster looked healthy moments earlier.

Concrete evidence:
- preserved failing runs under `tests/ha/runs/ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins/...`
- fresh local reproduction on 2026-03-12 in run directory `tests/ha/runs/ha_all_nodes_stopped_then_two_nodes_restarted_then_final_node_rejoins/ha-all-nodes-stopped-then-two-nodes-restarted-then-final-node-rejoins-1773346578502-2013518`

Explore and research the codebase first, then fix. Focus on the greenfield HA restart/election path after full outage, especially:
- eligibility construction in `src/ha/worker.rs`
- no-lease election selection in `src/ha/decide.rs`
- publication / authority projection stability during replica rejoin
- observer-visible leader resolution in `src/cli/status.rs` and `src/cli/connect.rs`

The fix should make both the lease-open two-node restore and the final-node rejoin converge reliably, with no transient operator-visible leader disagreement that breaks `pgtm primary`.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
