# HA Autonomy Audit (Task 15)

Date (UTC): 2026-03-03

## Rule being audited
E2E/integration scenarios should not directly execute HA transition internals; transitions should be produced by running worker loops based on external stimuli.

## What was checked
- Searched for direct decision/action entrypoints in scenario code.
- Reviewed `src/ha/e2e_multi_node.rs` scenario driver section.

## Findings
- No direct calls from e2e scenarios into `ha::decide::decide(...)` or direct `dispatch_actions(...)` for transition driving.
- E2E scenario uses external-input mechanisms (API switchover request, postgres stop, DCS key stimuli) while HA loops execute continuously.

## Potential autonomy-sensitive points
- `src/ha/e2e_multi_node.rs` writes `/{scope}/leader` directly during fault injection (`control_store.write_path(&leader_path(...), ...)` and delete/write around failover).
- Assessment: treat as explicit control-plane fault stimulus (external DCS state perturbation), not direct invocation of HA internals.
- Required rationale: these writes are used to simulate conflicting/changed lease ownership; actual transition actions (promote/demote/fence) are still dispatched by HA workers.

## Result
- PASS with documented exception rationale for explicit DCS leader-key perturbation as fault injection.
- No direct internal HA transition execution path was used by e2e/integration scenario drivers.
