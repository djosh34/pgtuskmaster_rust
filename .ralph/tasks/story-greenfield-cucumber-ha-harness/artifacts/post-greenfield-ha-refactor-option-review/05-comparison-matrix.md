# Comparison matrix

## Scenario-class mapping

| Failure class / scenario family | Option A: Authority layer convergence | Option B: Unified authority snapshot and ranker | Option C: Single HA authority machine |
| --- | --- | --- | --- |
| Lone survivor still operator-visible after quorum loss | Fixed by configured-membership majority and shared authority predicates | Fixed by configured-membership majority inside normalized authority snapshot | Fixed by machine-owned majority rules from startup onward |
| No-quorum primary view disappears but fail-safe state is inconsistent | Improved by richer trust states and explicit fail-safe criteria | Fixed more cleanly by typed authority states that separate non-authoritative from fail-safe | Fixed via explicit non-authoritative / fail-safe lifecycle phases |
| Healthy two-node majority fails to elect a new primary | Fixed by shared leader eligibility and majority-aware trust | Fixed by ranked candidate selection over typed authority snapshot | Fixed by rank-guarded transitions inside one machine |
| Lagging replica still wins failover | Fixed by shared durability-aware eligibility helper | Fixed by one ranker used for failover and switchover | Fixed by machine transition guards backed by ranker |
| Isolated or degraded target is accepted for switchover | Fixed by reusing failover eligibility for switchover validation | Fixed by same ranker and eligibility view used everywhere | Fixed by same transition guard for automatic and manual leadership changes |
| DCS-cut primary stays authoritative under mixed-fault topology | Partially reduced by richer trust predicates; still depends on discipline across old phase model | Strongly addressed by typed uncertainty model separating DCS, API, and SQL signals | Strongly addressed, but with highest migration cost |
| Wedged primary is not replaced | Fixed if storage stall feeds shared authority/ranker inputs | Fixed more robustly because ranker and authority snapshot can demote stale primary authority explicitly | Fixed through machine transitions that reclassify stalled primary and elect successor |
| Old primary remains `unknown` after switchover | Improved by shared recovery / rejoin authority checks | Fixed more cleanly by explicit integration state in authority snapshot | Fixed by integrated demotion-to-replica lifecycle phases |
| Clone / broken-rejoin path reports success before queryability | Improved by shared readiness gating | Strongly addressed by explicit recovery integration states | Strongly addressed by machine-managed recovery phases |
| Repeated failover stalls on stale leader lease | Improved by shared authority predicates and safer lease / ranking interplay | Strongly addressed by normalized stale-leader treatment inside authority snapshot | Strongly addressed by eliminating split startup / steady-state / worker-local authority paths |
| Rapid repeated failovers can lose acknowledged writes | Partially improved by durability-aware ranker | Strongly addressed because the ranker is first-class and shared everywhere | Strongly addressed but with highest implementation risk |

## Tradeoff comparison

| Dimension | Option A | Option B | Option C |
| --- | --- | --- | --- |
| Change size | Smallest | Middle | Largest |
| Conceptual cleanliness | Good | Best balance | Highest potential end-state cleanliness |
| Incremental delivery | Best | Good | Worst |
| Risk of hidden edge-case leftovers | Highest of the three | Lowest practical risk | Lowest theoretical leftovers but highest migration risk |
| Startup / steady-state alignment | Improved | Strong | Complete |
| Dedup removal quality | Good if effect identity is modeled well | Strong because effect identity is part of authority snapshot | Natural outcome of the machine design |
| Match to current code shape | Closest | Strong but intentionally reshaping HA model | Furthest from current shape |

## Recommendation

Best option: Option B, unified authority snapshot and ranker.

Why:

- It addresses the whole failure inventory as one model rather than as a predicate cleanup.
- It directly fixes the largest structural mismatch: startup, steady-state, recovery, and switchover are all reasoning over different fragments of authority today.
- It is still incremental enough to land without turning the refactor into a wholesale rewrite.
- It removes the worker dedup shortcut in a principled way by giving the loop explicit state about in-flight effects.

Second-best option: Option A, authority layer convergence.

Why:

- It is the safest incremental landing zone if the team wants to reduce immediate scope.
- It still forces quorum, ranking, startup alignment, and dedup removal into shared logic.
- The main downside is that it leaves more of the current shape intact, which increases the chance of future drift.

Option C remains credible, but it is better treated as a follow-on direction if Option B reveals that the current HA phase model is still too fragmented even after the shared authority snapshot exists.

## Final verification note

These options were cross-checked against:

- the greenfield HA feature files under [`cucumber_tests/ha/features`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/cucumber_tests/ha/features)
- the preserved bug-task evidence listed in [01-evidence-inventory.md](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/tasks/story-greenfield-cucumber-ha-harness/artifacts/post-greenfield-ha-refactor-option-review/01-evidence-inventory.md)
- the implicated source modules in `src/dcs`, `src/ha`, and `src/runtime`

The result is specific to this repo state. These are not generic HA architecture essays.
