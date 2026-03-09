## Bug: Targeted Switchover Request Can Promote Wrong Node <status>done</status> <passes>true</passes>

<description>
An accepted targeted switchover request is not reliably honored in the HA multi-node E2E environment. During work on repeated leadership-churn coverage, a request targeted at `node-2` was accepted through `POST /switchover`, but the cluster later stabilized on `node-3` as primary instead. The failure was reproduced in `e2e_multi_node_repeated_targeted_switchovers_preserve_single_primary`, which observed `node-3` as the only stable promoted primary after the targeted request to `node-2`.

The current behavior contradicts the operator/docs contract for targeted switchovers and strongly suggests the switchover request is being cleared or ignored before the intended successor has actually taken over. Explore and research the HA decision/apply path first, then fix the product behavior and add focused coverage that proves non-target nodes cannot win leadership while a targeted switchover is pending.
</description>

<acceptance_criteria>
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Plan

1. Confirm the real bug boundary across decision, lowering, and apply.
   - Re-read `src/ha/decide.rs`, `src/ha/decision.rs`, `src/ha/lower.rs`, and `src/ha/apply.rs` together before editing anything.
   - Preserve the DCS switchover request for the full handoff window; the current `HaDecision::StepDown(Switchover)` -> `clear_switchover=true` path is still the leading root-cause candidate because it frees untargeted replicas to campaign before the intended successor is established.
   - Treat this as a state-machine bug, not a test-only bug: the fix must explain who owns cleanup, when success is considered observed, and why non-target nodes remain blocked until then.

2. Redesign switchover completion so cleanup is owned by a safe success observer, not by the initial demotion step.
   - Keep targeted and untargeted switchover records in DCS until some node can positively observe that leadership has moved away from the old primary and the switchover has actually succeeded.
   - Do not assume the old primary is the right cleanup owner. If that node is unreachable or slow after demotion, cleanup still has to happen; prefer a success path owned by the promoted target or another node that can safely verify completion.
   - Make the targeted request continue blocking non-target leadership attempts for the entire interval between request acceptance and observed successor stabilization.

3. Refactor the decision/effect model to match the corrected lifecycle.
   - Remove or relocate `clear_switchover` from the `StepDownPlan` path if cleanup no longer belongs to `step_down`; update the decision type, lowering logic, apply dispatch, API/debug mapping, and CLI rendering consistently instead of leaving a stale boolean behind.
   - Add the smallest new decision/effect shape that can express “switchover completed; now clear the request” without reintroducing early deletion.
   - Keep error handling explicit; do not introduce any panic/unwrap/expect shortcuts while reshaping the flow.

4. Tighten unit coverage around the transition edges and public shape changes.
   - Update switchover decision tests in `src/ha/decide.rs` so the primary step-down path no longer expects immediate switchover clearing.
   - Add or extend tests for `WaitingSwitchoverSuccessor` and any new completion decision/effect so they prove:
     - the request remains pending while no successor is confirmed,
     - a non-target candidate keeps waiting while a targeted request names someone else,
     - cleanup happens only from the new success path,
     - the public/API/debug representation no longer advertises stale `clear_switchover` semantics if that field is removed or repurposed.
   - Update lowering/apply tests so dispatch assertions move from the old `step_down` path to the new completion path.

5. Add focused E2E coverage for the user-visible regression.
   - Reuse the existing stable-primary wait helper in `tests/ha/support/multi_node.rs` instead of inventing a parallel waiter; add only the missing targeted-switchover request helper and scenario glue.
   - Add a scenario in `tests/ha/support/multi_node.rs` and wire it through `tests/ha_multi_node_failover.rs` proving that a targeted request to one healthy replica stabilizes on that exact member, not on an alternate healthy replica.
   - Make the scenario explicit about old primary, requested successor, and disallowed alternate replica so failures distinguish “request ignored” from “request cleared too early” from “target blocked forever.”

6. Update operator-facing docs and references to match the new semantics.
   - Revise `docs/src/how-to/perform-switchover.md` for the corrected switchover lifecycle.
   - Update `docs/src/reference/ha-decisions.md` and `docs/src/reference/dcs-state-model.md` to reflect the new cleanup timing and any decision-shape changes; regenerate book artifacts only through the repo’s normal docs flow.
   - Do not treat draft docs as source of truth unless the build requires touching them.

7. Run the full verification and closeout only after code and docs are complete.
   - Run `make check`, `make test`, `make test-long`, and `make lint`.
   - After every gate passes, set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit all changes including `.ralph` state, and push.

NOW EXECUTE
