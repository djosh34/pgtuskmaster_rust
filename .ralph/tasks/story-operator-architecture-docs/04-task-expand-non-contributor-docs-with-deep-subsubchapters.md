## Task: Expand Non-Contributor Docs with Deep Subsubchapters While Keeping Strong Overviews <status>completed</status> <passes>true</passes>

<description>
**Goal:** Vastly deepen the non-contributor documentation by adding long-form, detail-rich subsubchapters and flowing explanations, while preserving the existing high-level overview quality at chapter entry points.

**Scope:**
- Keep overview pages and high-level framing concise and strong; do not flatten everything into dense walls of text.
- Add substantial depth below those overviews:
  - create/expand subchapters and subsubchapters with concrete operational and architectural detail
  - explain behavior, rationale, failure modes, and practical implications in natural narrative prose
  - connect sections so each chapter can be read independently without losing context.
- Encourage intentional, useful duplication where it improves out-of-context readability:
  - repeat key context when entering a new detailed subsection
  - cross-reference related sections only when it materially helps the reader.
- Expand explanations across the non-contributor areas:
  - Start Here / Quick Start
  - Operator Guide
  - System Lifecycle
  - Architecture Assurance
  - Interfaces
  - glossary/supporting navigation text.
- Prefer paragraph-first writing with clear transitions and precise terminology over terse bullet stubs.

**Context from research:**
- The top-level docs structure is already in good shape and high-level overviews are working well.
- Remaining gap: many subchapters are still too slim for readers who need deeper detail, especially when reading a page out of sequence.
- This project benefits from explanation redundancy in strategic places because operational and HA topics are tightly coupled and easy to misread in isolation.

**Expected outcome:**
- Readers can use overview chapters for quick orientation and then drop into deep subsections for complete detail.
- Non-contributor docs become substantially longer and richer where needed, with natural flow and explicit transitions.
- Detailed sections explain not just what to do, but how/why behavior occurs, including edge cases and operational consequences.
- Documentation is easier to consume out of context because critical rationale is repeated where appropriate.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x]  Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] `docs/src/introduction.md` remains a high-level orientation page and adds stronger handoff paragraphs into deeper sections without becoming implementation-dense.
- [x] `docs/src/start-here/problem.md` keeps executive framing but expands the lower section(s) with detailed problem mechanics, failure pressure, and why those constraints matter in practice.
- [x] `docs/src/start-here/solution.md` preserves high-level solution overview and adds deeper narrative on mechanism boundaries, tradeoffs, and concrete behavior implications.
- [x] `docs/src/start-here/docs-map.md` is expanded to explain how to read the book at multiple depth levels (overview path vs deep-dive path) with clear context for out-of-order reading.
- [x] `docs/src/quick-start/index.md` remains concise at top and adds richer explanation about what each quick-start stage proves and why those checks matter.
- [x] `docs/src/quick-start/prerequisites.md` gains detailed rationale for each prerequisite, common misreads, and symptom-level consequences when missing/misconfigured.
- [x] `docs/src/quick-start/first-run.md` is expanded with deeper step-by-step narrative, expected intermediate states, and troubleshooting guidance embedded at the relevant steps.
- [x] `docs/src/quick-start/initial-validation.md` includes detailed interpretation guidance for validation signals (what confirms health vs what indicates drift/risk).
- [x] `docs/src/operator/index.md` remains overview-first and clearly routes readers to deep operational subchapters by task intent and incident context.
- [x] `docs/src/operator/configuration.md` adds significantly deeper subsubchapters for config groups, behavior impact, failure signatures, and practical tuning decisions.
- [x] `docs/src/operator/deployment.md` adds detailed deployment flow explanations, environment assumptions, and nuanced operational caveats.
- [x] `docs/src/operator/observability.md` is expanded with deeper interpretation patterns for metrics/logs/signals and clearer diagnosis pathways.
- [x] `docs/src/operator/troubleshooting.md` is expanded into longer symptom-first narratives with detailed diagnostics, branching logic, and likely root-cause mapping.
- [x] `docs/src/lifecycle/index.md` keeps top-level lifecycle map concise and adds explicit handoff guidance into deeper phase-level chapters.
- [x] `docs/src/lifecycle/bootstrap.md` adds deeper chronology and decision-path detail, including edge conditions and operator-visible consequences.
- [x] `docs/src/lifecycle/steady-state.md` expands on normal control loops, expected drift windows, and what “healthy” means under real conditions.
- [x] `docs/src/lifecycle/switchover.md` provides deeper sequence and safety reasoning, including preconditions, orchestration details, and failure-path interpretation.
- [x] `docs/src/lifecycle/failover.md` adds deep explanations for trigger conditions, decision gates, and recovery implications.
- [x] `docs/src/lifecycle/failsafe-fencing.md` expands with nuanced fail-safe/fencing behavior details, bounded expectations, and operational caution points.
- [x] `docs/src/lifecycle/recovery.md` provides deeper recovery narratives and decision branches with practical validation checkpoints.
- [x] `docs/src/assurance/index.md` remains high-level and adds stronger map text guiding readers from summary claims to deep argument chapters.
- [x] `docs/src/assurance/safety-invariants.md` expands invariant explanations with richer context, boundaries, and consequence analysis.
- [x] `docs/src/assurance/decision-model.md` adds deeper stepwise reasoning, conflict handling detail, and clearer mapping between assumptions and outcomes.
- [x] `docs/src/assurance/dcs-data-model.md` expands with detailed ownership/update semantics and cross-subsystem implications.
- [x] `docs/src/assurance/runtime-topology.md` adds richer topology behavior descriptions, synchronization expectations, and stress-path nuance.
- [x] `docs/src/assurance/safety-case.md` includes deeper structured argumentation, assumptions, and “what this does not guarantee” clarifications.
- [x] `docs/src/assurance/tradeoffs-limits.md` is expanded with more explicit tradeoff narratives, operational cost reasoning, and scenario-oriented caveats.
- [x] `docs/src/interfaces/index.md` remains concise overview and adds clearer deep-dive entry guidance for API/CLI behavior details.
- [x] `docs/src/interfaces/node-api.md` expands endpoint/contract explanations with more detail on semantics, sequencing, and operational interpretation.
- [x] `docs/src/interfaces/cli.md` expands command behavior narratives, expected response patterns, and practical usage nuance.
- [x] `docs/src/concepts/glossary.md` extends key terms with richer definitions and concise context paragraphs so terms remain understandable out of chapter context.
- [x] `docs/src/SUMMARY.md` is updated as needed so any added deep subchapters/subsubchapters are navigable and ordered coherently.
- [x] Detailed sections are substantially expanded in length and depth, but chapter entry overviews stay intentionally high-level and readable.
- [x] Intentional duplication of essential context is explicitly allowed and used where it improves comprehension for readers entering mid-book.
- [x] Prose quality standard: flowing natural paragraphs, clear transitions, and precise language; terse bullet-only outlines are eliminated in deep sections.
- [x] All behavior claims remain aligned with current implementation/tests; speculative or over-absolute wording is bounded or removed.
- [x] `make docs-lint` passes cleanly
- [x] `make docs-build` passes cleanly
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Detailed Execution Plan (Draft 2, skeptical review updated 2026-03-07)

### Skeptical review changes from Draft 1

- The earlier draft treated `docs/src/SUMMARY.md` as likely minor cleanup. After re-checking the current book structure, that assumption was too weak: the navigation is flatter than the deepened docs will be, so execution should treat navigation wording and section ordering as a deliberate part of the rewrite rather than a last-minute tidy-up.
- The earlier draft pushed interfaces and glossary work almost to the end. Re-checking `docs/src/interfaces/index.md` and `docs/src/concepts/glossary.md` showed that the current terminology scaffold is too thin to leave unstable while the rest of the book is being rewritten. Execution should front-load glossary and interface framing enough to lock vocabulary and reader expectations before the deeper chapter passes.

### 1. Planning baseline and non-goals

- The current non-contributor docs already have the right top-level structure and chapter ordering in `docs/src/SUMMARY.md`; the gap is depth inside many chapter bodies, not missing major sections.
- The overview/index pages are intentionally short today. Execution must keep them readable orientation pages and move most new density into subordinate headings and subsubheadings instead of turning every landing page into a wall of text.
- The existing operator docs already include a separate `docs/src/operator/container-deployment.md` page. This task should preserve that page as the concrete container-first path and use the other non-contributor pages to explain context, rationale, lifecycle, and interpretation around it.
- The current Quick Start is explicitly container-first. Execution should deepen rationale and interpretation around that path rather than reintroducing host-native quick-start drift.
- The contributor docs are out of scope except where a contributor-facing page needs a link target or wording alignment caused directly by navigation changes in `docs/src/SUMMARY.md`. Do not expand contributor prose as part of this task.
- Do not invent features, guarantees, or operator capabilities that are not present in the current runtime and tests. Any statement that sounds absolute must be re-checked against code and tests during execution.

### 2. Writing contract for this rewrite

- Keep each index/overview page short at the top:
  - opening orientation paragraphs
  - a clear “read this next” handoff
  - only enough summary to frame the detailed pages beneath it
- Expand deep pages with paragraph-first prose and clear subsectioning:
  - behavior
  - rationale
  - edge conditions
  - operational consequences
  - how to interpret symptoms or states
- Use intentional context repetition when a reader may land mid-book. If a lifecycle or assurance page depends on trust, DCS state, or operator intent concepts, restate the minimum context locally instead of forcing a backtrack.
- Use bullets and tables only where they improve scanning. They should summarize a surrounding narrative, not replace it.
- Prefer bounded language such as “typically”, “under the checked-in container path”, “when trust remains full”, or “the current implementation” unless there is direct evidence for a stronger claim.
- Keep the prose operator-facing. This task may cite implementation behavior, but it should not turn non-contributor pages into code-path commentary.

### 3. Exhaustive checklist: files to modify during `NOW EXECUTE`

- [x] `docs/src/introduction.md`
  - Keep the page short and orientation-first.
  - Add stronger handoff paragraphs explaining how to choose the overview route versus the deep-dive route.
  - Make the opening explain the relationship between Quick Start, Operator Guide, Lifecycle, Assurance, and Interfaces without becoming implementation-dense.

- [x] `docs/src/start-here/problem.md`
  - Preserve executive framing at the top.
  - Expand lower sections with concrete failure mechanics: ambiguous leadership, stale assumptions, coordination loss, and why manual/scripted HA often fails under pressure.
  - Add practical consequence language for operators: what goes wrong operationally when safety signals are weak.

- [x] `docs/src/start-here/solution.md`
  - Keep the top-level observe-decide-act framing.
  - Add deeper subsections on mechanism boundaries: what the HA loop decides, what DCS contributes, what PostgreSQL observation contributes, and what operator intent can and cannot do.
  - Add explicit tradeoff discussion so readers understand conservative behavior before they reach troubleshooting.

- [x] `docs/src/start-here/docs-map.md`
  - Expand into a true “how to read this book” page.
  - Add separate paths for overview reading, incident reading, first-run reading, and architecture-assurance reading.
  - Add out-of-order reading guidance that repeats enough context so a reader knows what assumptions each major section already makes.

- [x] `docs/src/quick-start/index.md`
  - Keep the landing page concise.
  - Add explanation of what each stage proves: prerequisites prove environment fidelity, first run proves deployability, initial validation proves exposed surfaces behave coherently.
  - Add a handoff paragraph to the Operator Guide once the quick-start lab works.

- [x] `docs/src/quick-start/prerequisites.md`
  - Expand each prerequisite with rationale and common misreads.
  - Explain symptom-level consequences for missing Docker, broken Compose config rendering, missing secret files, bad port choices, or unwritable repo state.
  - Keep the checked-in Compose path as the default and bound host-native deployment as advanced/secondary.

- [x] `docs/src/quick-start/first-run.md`
  - Keep the single-node Compose flow as the main path.
  - Deepen each step with expected intermediate state, what the operator should see, why the step exists, and what likely failure looks like at that exact step.
  - Embed troubleshooting guidance inline rather than in a detached appendix.

- [x] `docs/src/quick-start/initial-validation.md`
  - Expand from a checklist into an interpretation guide.
  - For each validation signal, explain what healthy output means, what suspicious output means, and what drift/risk it points to.
  - Clarify the difference between “API is reachable”, “HA state is coherent”, and “the whole container path is trustworthy enough to continue”.

- [x] `docs/src/operator/index.md`
  - Keep it overview-first.
  - Add routing guidance by task intent: first deployment, config editing, incident triage, signal interpretation, and symptom-first debugging.
  - Add explicit cross-links into Lifecycle when behavior explanation matters more than step-by-step operator action.

- [x] `docs/src/operator/configuration.md`
  - Preserve existing strong baseline examples.
  - Add deeper subsubchapters for major config groups: cluster identity, PostgreSQL wiring, DCS scope/endpoints, HA timing, process binaries, API security, and debug posture.
  - For each group, add behavior impact, common failure signatures, and practical tuning/hardening considerations.

- [x] `docs/src/operator/deployment.md`
  - Expand beyond topology inventory into a deeper operational narrative.
  - Explain environment assumptions, network exposure decisions, internal versus external reachability, and where manual deployment diverges from the checked-in container path.
  - Add nuanced caveats about what the checked-in lab topology proves and what it does not prove about production hardening.

- [x] `docs/src/operator/observability.md`
  - Expand interpretation guidance for `/ha/state`, logs, DCS records, and debug routes.
  - Add deeper diagnosis patterns: how to correlate signals, what healthy convergence looks like, and how to recognize stale/coincidental evidence.
  - Keep observability operator-facing rather than backend-implementation-heavy.

- [x] `docs/src/operator/troubleshooting.md`
  - Expand each symptom section into longer narrative branches.
  - For each major symptom, add likely causes, ordering of checks, what evidence rules a cause in or out, and how to interpret conflicting signals.
  - Add stronger mapping from symptoms to lifecycle chapters when the answer depends on phase semantics.

- [x] `docs/src/lifecycle/index.md`
  - Keep it concise as the lifecycle map.
  - Add explicit handoff text describing which phase chapter answers which operational question.
  - Make it clear that this section explains why the node behaves as it does, not just what commands to run.

- [x] `docs/src/lifecycle/bootstrap.md`
  - Expand chronology, decision points, and edge cases around init, clone, and resume.
  - Add operator-visible consequences for each planner choice and refusal path.
  - Explain how stale on-disk state, DCS evidence, and startup restrictions interact in practice.

- [x] `docs/src/lifecycle/steady-state.md`
  - Expand the idea of “healthy steady state” beyond “nothing is happening”.
  - Add expected drift windows, normal reconciliation churn, and how to interpret quiet versus suspiciously idle behavior.
  - Clarify what operators should continuously expect to remain true when the cluster is healthy.

- [x] `docs/src/lifecycle/switchover.md`
  - Deepen sequence reasoning, safety preconditions, and orchestration flow.
  - Explain the meaning of accepted intent versus completed transition.
  - Add failure-path interpretation for stalled demotion, ineligible targets, or trust degradation mid-flow.

- [x] `docs/src/lifecycle/failover.md`
  - Expand the trigger and decision-gate explanation.
  - Distinguish missing leader evidence from sufficient promotion evidence.
  - Add recovery implications and why delayed failover can still be the correct safe outcome.

- [x] `docs/src/lifecycle/failsafe-fencing.md`
  - Expand fail-safe and fencing separately, with nuanced expectations and caution points.
  - Explain what each phase means operationally, what it does not guarantee, and how to interpret API availability during degraded coordination.
  - Add bounded language around demotion/constraining behavior so the text matches actual runtime behavior.

- [x] `docs/src/lifecycle/recovery.md`
  - Expand the recovery narrative into decision branches: rewind, bootstrap, rejoin, and refusal-to-rejoin until evidence is coherent.
  - Add practical validation checkpoints after recovery work.
  - Explain what operators should verify before trusting a recovered member again.

- [x] `docs/src/assurance/index.md`
  - Keep this as a high-level map into the assurance argument.
  - Add stronger guidance for readers choosing between invariants, decision model, DCS semantics, topology, safety case, and tradeoffs.
  - Make clear that this section explains confidence boundaries, not marketing claims.

- [x] `docs/src/assurance/safety-invariants.md`
  - Expand each invariant with context, boundary conditions, and consequence analysis.
  - Explain how invariants help distinguish protective conservatism from true defects.
  - Add explicit caveats about the evidence those invariants depend on.

- [x] `docs/src/assurance/decision-model.md`
  - Expand the stepwise reasoning model.
  - Add conflict-handling detail and a clearer mapping between inputs, guards, outcomes, and side effects.
  - Explain how operator intent enters the same decision model rather than bypassing it.

- [x] `docs/src/assurance/dcs-data-model.md`
  - Expand record ownership and update semantics.
  - Add cross-subsystem implications: why a stale leader key, switchover record, or member record means different debugging paths.
  - Clarify write ownership boundaries in operator-meaningful language.

- [x] `docs/src/assurance/runtime-topology.md`
  - Expand topology behavior, synchronization expectations, and subsystem boundaries.
  - Add more nuance on how worker separation helps diagnosis and failure containment.
  - Keep references to threads/tasks and projections bounded to what the docs can support confidently.

- [x] `docs/src/assurance/safety-case.md`
  - Expand the structured argument.
  - Add explicit assumptions, supporting reasoning, residual risk, and a “what this does not guarantee” section.
  - Ensure the page reads as a cautious assurance argument, not an absolute safety promise.

- [x] `docs/src/assurance/tradeoffs-limits.md`
  - Expand tradeoff narratives with scenario-oriented caveats.
  - Make operational costs explicit: slower promotion under ambiguity, configuration burden, and recovery prerequisites.
  - Add stronger “how to read conservative behavior” guidance tied back to lifecycle and troubleshooting.

- [x] `docs/src/interfaces/index.md`
  - Keep the overview concise.
  - Add clearer guidance for when to use API docs versus CLI docs and when to jump back to lifecycle/operator chapters for semantics.
  - Frame the Interfaces section as the contract surface, not the full behavior explanation.

- [x] `docs/src/interfaces/node-api.md`
  - Preserve the current route inventory and examples.
  - Expand endpoint semantics: sequencing, acceptance versus completion, degraded-mode interpretation, auth posture, and debug-route relationship to the API listener.
  - Add more practical guidance on how operators should interpret response patterns during incidents.

- [x] `docs/src/interfaces/cli.md`
  - Preserve the API-mapping structure.
  - Expand command-behavior narratives, expected responses, and practical usage nuance for secured versus local-lab paths.
  - Clarify what CLI success means versus what still requires follow-up observation in `/ha/state`.

- [x] `docs/src/concepts/glossary.md`
  - Expand terse definitions into richer short paragraphs.
  - Add enough context that a reader landing from search can understand a term without first reading prior chapters.
  - Keep the glossary concise per entry even while adding context.

- [x] `docs/src/SUMMARY.md`
  - Treat navigation review as a real deliverable, not optional cleanup.
  - Update section wording and page ordering wherever the expanded chapter depth makes the current flat navigation less legible.
  - Keep the same top-level chapter families unless execution finds a strong reader-comprehension reason to move a page between them.

- [x] `docs/src/operator/container-deployment.md`
  - Perform a consistency review during execution.
  - Patch only if adjacent page rewrites create stale wording, broken cross-links, or mismatched expectations about the container-first path.
  - Do not expand this page for its own sake unless that is required to keep the operator section coherent.

### 4. Primary evidence sources to re-check while writing

- Deployment and quick-start behavior:
  - `docker/compose/docker-compose.single.yml`
  - `docker/compose/docker-compose.cluster.yml`
  - `docker/configs/**`
  - `Makefile`
  - `tools/docker/*.sh`
- Runtime and lifecycle behavior:
  - `src/runtime/node.rs`
  - `src/ha/worker.rs`
  - `src/ha/decide.rs`
  - `src/ha/state.rs`
  - `src/process/worker.rs`
  - `src/pginfo/worker.rs`
  - `src/dcs/worker.rs`
  - `src/dcs/store.rs`
  - `src/dcs/keys.rs`
- API/debug/interface behavior:
  - `src/api/worker.rs`
  - `src/api/controller.rs`
  - `src/api/fallback.rs`
  - `src/debug_api/worker.rs`
  - `src/debug_api/snapshot.rs`
  - `src/debug_api/view.rs`
  - `src/bin/pgtuskmasterctl.rs`
- Tests and operator-proof points:
  - `tests/cli_binary.rs`
  - `tests/bdd_api_http.rs`
  - `tests/bdd_state_watch.rs`
  - `tests/policy_e2e_api_only.rs`
  - `src/ha/e2e_*.rs`
  - any docs-related gates in the repository build/test targets

### 5. Execution phases for the later `NOW EXECUTE` pass

- [x] Phase A: strengthen the reader map before deep rewrites
  - [x] Rewrite `docs/src/introduction.md`, `docs/src/start-here/problem.md`, `docs/src/start-here/solution.md`, and `docs/src/start-here/docs-map.md`.
  - [x] Make the reading paths explicit so later detailed chapters can assume less and still remain navigable.

- [x] Phase B: lock terminology and navigation guardrails early
  - [x] Rewrite `docs/src/interfaces/index.md` and `docs/src/concepts/glossary.md` before the large chapter-family passes.
  - [x] Do an initial `docs/src/SUMMARY.md` review so the eventual deepened structure already has coherent reader-facing labels and placement.
  - [x] Keep this pass light on route-level detail; the goal here is to stabilize vocabulary, reading intent, and navigation expectations.

- [x] Phase C: deepen the quick-start path without losing its speed
  - [x] Rewrite `docs/src/quick-start/index.md`, `docs/src/quick-start/prerequisites.md`, `docs/src/quick-start/first-run.md`, and `docs/src/quick-start/initial-validation.md`.
  - [x] Keep the top-of-page quick-start experience short, but add interpretation detail below the main flow.

- [x] Phase D: deepen the operator manual
  - [x] Rewrite `docs/src/operator/index.md`, `docs/src/operator/configuration.md`, `docs/src/operator/deployment.md`, `docs/src/operator/observability.md`, and `docs/src/operator/troubleshooting.md`.
  - [x] Review `docs/src/operator/container-deployment.md` for consistency after adjacent rewrites.

- [x] Phase E: deepen lifecycle explanation
  - [x] Rewrite `docs/src/lifecycle/index.md`, `docs/src/lifecycle/bootstrap.md`, `docs/src/lifecycle/steady-state.md`, `docs/src/lifecycle/switchover.md`, `docs/src/lifecycle/failover.md`, `docs/src/lifecycle/failsafe-fencing.md`, and `docs/src/lifecycle/recovery.md`.
  - [x] Add stronger phase-to-phase transitions and handoff language so each page stands on its own.

- [x] Phase F: deepen the assurance argument
  - [x] Rewrite `docs/src/assurance/index.md`, `docs/src/assurance/safety-invariants.md`, `docs/src/assurance/decision-model.md`, `docs/src/assurance/dcs-data-model.md`, `docs/src/assurance/runtime-topology.md`, `docs/src/assurance/safety-case.md`, and `docs/src/assurance/tradeoffs-limits.md`.
  - [x] Ensure assurance language stays bounded and evidence-backed.

- [x] Phase G: deepen the interface contract surfaces
  - [x] Rewrite `docs/src/interfaces/node-api.md` and `docs/src/interfaces/cli.md` after lifecycle and assurance wording is stable.
  - [x] Revisit `docs/src/interfaces/index.md`, `docs/src/concepts/glossary.md`, and `docs/src/SUMMARY.md` as needed so route semantics, vocabulary, and navigation match the final book wording.

- [x] Phase H: skeptical docs-only review before running heavy gates
  - [x] Re-read every touched page for over-claims, duplicated contradictions, stale cross-links, and accidental contributor-level code-detail drift.
  - [x] Ensure each overview page remains readable and that the added density lives mostly below overview openings.

- [x] Phase I: required verification and any follow-up fixes
  - [x] `make docs-lint`
  - [x] `make docs-build`
  - [x] `make check`
  - [x] `make test`
  - [x] `make test-long`
  - [x] `make lint`
  - [x] If any gate fails because the doc claims drift from behavior or examples, fix the docs and rerun until fully green.

### 6. Parallel execution split to use during `NOW EXECUTE`

- Track 1:
  - overview/start-here/quick-start pages plus early glossary support
- Track 2:
  - operator pages
- Track 3:
  - lifecycle pages
- Track 4:
  - assurance pages
- Track 5:
  - interfaces API/CLI pages and final navigation/consistency sweep

- Parallel work is allowed only where file ownership is disjoint.
- The main integration pass must normalize voice, cross-links, and repeated context after parallel drafting.
- Verification and any fixes after gate failures should be done serially to avoid conflicting edits.

### 7. Exact execution order once this becomes `NOW EXECUTE`

1. Update overview/start-here pages first so the book’s handoff language is stable.
2. Rewrite `docs/src/interfaces/index.md`, `docs/src/concepts/glossary.md`, and perform an initial `docs/src/SUMMARY.md` pass so terminology and navigation expectations are stable before the largest rewrites.
3. Rewrite quick-start pages next so deployment assumptions and validation interpretation are explicit.
4. Rewrite operator pages, including the consistency review of `docs/src/operator/container-deployment.md`.
5. Rewrite lifecycle pages after operator pages so incident explanations can reuse stable operator terminology.
6. Rewrite assurance pages after lifecycle pages so invariants and tradeoffs can refer to already-stabilized lifecycle language.
7. Rewrite `docs/src/interfaces/node-api.md` and `docs/src/interfaces/cli.md`, then revisit the Interfaces index, glossary, and summary navigation for final alignment.
8. Run a full skeptical editorial pass over all touched pages.
9. Run the required gates in the fixed order listed above and repair any fallout.
10. Only after all gates pass, tick the acceptance criteria, set `<passes>true</passes>`, run `.ralph/task_switch.sh`, commit, and push.

### 8. Risks and assumptions that the required `TO BE VERIFIED` pass must challenge

- The current plan still assumes `docs/src/operator/container-deployment.md` mainly needs consistency review rather than substantive expansion; execution should re-check whether leaving it mostly stable would create an uneven operator section.
- The current plan now front-loads glossary/interface framing, but execution should still watch for vocabulary drift that forces a second substantive glossary pass after lifecycle and assurance rewrites.
- The current plan assumes the acceptance criteria can be met without touching contributor docs; the skeptical pass should verify that no non-contributor cross-link drift forces a minimal contributor-doc adjustment.
- The skeptical review requirement has been satisfied in this draft by changing both execution order and navigation scope; further execution should follow this updated order rather than the original draft.

NOW EXECUTE
