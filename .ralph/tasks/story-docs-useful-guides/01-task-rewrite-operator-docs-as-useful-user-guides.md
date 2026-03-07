## Task: Rewrite operator docs as useful user guides and remove horror pages <status>done</status> <passes>true</passes>

<description>
Rewrite the non-contributor documentation so it reads like a strong operator/product guide instead of a thin or awkwardly templated book.

The agent must explore the current docs and implementation first, then rewrite the docs around what actually helps a user understand and operate the system.

This task must implement the following fixed product decisions:
- remove report-style, verification-report, and similar pages from the docs entirely; they must not remain in the book, appendix, or normal navigation
- remove cringe/filler writing and stop using mechanical prose templates such as forced "Why this exists", "Tradeoffs", and "When this matters" sections
- rewrite the operator-facing docs into direct, useful prose that is substantially richer where important information is currently missing
- make the docs feel like actual good docs: better to read, better to use, easier to navigate, and more helpful when trying to understand or operate the system
- keep the docs grounded in the real product and implementation rather than speculative framing, fake maturity signals, or fake legacy/version framing
- ensure the recommended setup and examples are secure by default
- do not present container-first quick-start or Docker-first setup as current product reality before the existing container deployment story lands
- keep the book focused on helping a user understand what the product does, how to run it, how to reason about it, and what to check when things go wrong

This is a docs quality and usefulness rewrite, not just a structure tweak. The agent should use parallel subagents after exploration to rewrite the book in coherent slices.
</description>

<acceptance_criteria>
- [x] `docs/src/verification/index.md` and verification-report style pages are removed from the docs book rather than merely rewritten or relocated
- [x] Operator-facing docs are rewritten in direct, useful prose without filler or forced template sections
- [x] Important missing operational and product information is added where needed, especially where the current docs are thin, weird, or hard to use
- [x] The docs are materially better to read and use as documentation, not merely more verbose
- [x] Security recommendations and examples in operator docs align with the intended secure default approach
- [x] Docs do not claim Docker/container-first setup before the existing container deployment story actually lands
- [x] Docs do not present fake product maturity, fake legacy/version framing, or speculative rationale that is not grounded in the implementation
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
</acceptance_criteria>

## Execution plan

### Phase 1: Remove stale report-style docs from the user-facing book

- Remove the `Verification` section from `docs/src/SUMMARY.md`.
- Delete `docs/src/verification/index.md` and `docs/src/verification/task-33-docs-verification-report.md` from the docs tree instead of relocating them.
- Re-read and update nearby navigation pages so the book still reads coherently after that removal, especially `docs/src/introduction.md` and `docs/src/start-here/docs-map.md`.
- Update contributor-facing references that currently point into the deleted verification section, especially `docs/src/contributors/index.md` and `docs/src/contributors/verification.md`, so verification guidance stays available without keeping the stale report pages in the main book.

### Phase 2: Rebuild the operator reading path around real user tasks

- Rewrite the thin operator-facing pages so they answer the questions an operator actually has in order:
  - what this system is and how to navigate the docs
  - what must exist before first run
  - how to start a node safely
  - how to check whether it is healthy
  - how to reason about deployment, observability, and troubleshooting
- Prioritize these files for the rewrite:
  - `docs/src/introduction.md`
  - `docs/src/start-here/docs-map.md`
  - `docs/src/quick-start/index.md`
  - `docs/src/quick-start/prerequisites.md`
  - `docs/src/quick-start/first-run.md`
  - `docs/src/quick-start/initial-validation.md`
  - `docs/src/operator/index.md`
  - `docs/src/operator/configuration.md`
  - `docs/src/operator/deployment.md`
  - `docs/src/operator/observability.md`
  - `docs/src/operator/troubleshooting.md`
  - `docs/src/interfaces/cli.md`
  - `docs/src/interfaces/node-api.md`
- Also rewrite the lifecycle pages that are still part of the operator reading path and still use the same mechanical template structure:
  - `docs/src/lifecycle/index.md`
  - `docs/src/lifecycle/bootstrap.md`
  - `docs/src/lifecycle/steady-state.md`
  - `docs/src/lifecycle/switchover.md`
  - `docs/src/lifecycle/failover.md`
  - `docs/src/lifecycle/recovery.md`
  - `docs/src/lifecycle/failsafe-fencing.md`
- Remove the mechanical section patterns currently present in several pages, especially headings like `Why this exists`, `Tradeoffs`, and `When this matters in operations`, unless a specific page genuinely needs one and it reads naturally.
- Keep the book local-binary and etcd oriented. Do not introduce container-first, Docker-first, or speculative deployment guidance.

### Phase 3: Ground every example and workflow in the current implementation

- Re-verify the rewritten docs against the current implementation before merging prose changes, using at minimum:
  - `src/bin/pgtuskmaster.rs`
  - `src/cli/args.rs`
  - `src/api/worker.rs`
  - `src/api/controller.rs`
  - `src/config/schema.rs`
  - `src/config/parser.rs`
  - `src/config/defaults.rs`
  - `src/tls.rs`
- Correct the current CLI/API mismatches in the docs:
  - the docs currently show `pgtuskmasterctl switchover --to <member-id>`, but the implementation exposes `pgtuskmasterctl ha switchover request --requested-by <member-id>`
  - the docs currently encourage `pgtuskmasterctl ha state` without addressing that the CLI default base URL is `http://127.0.0.1:8008` while the runtime default API listen address is `127.0.0.1:8080`
  - the docs must either document the explicit `--base-url http://127.0.0.1:8080` requirement or fix the code/docs mismatch during execution; the final book cannot leave that workflow misleading
- Correct the current config/security mismatch in `docs/src/operator/configuration.md`:
  - the example currently binds `api.listen_addr = "0.0.0.0:8080"` and disables API TLS/auth while the surrounding prose claims a fail-closed secure schema
  - rewrite the examples so the recommended path is secure by default and local-only examples are clearly labeled as local-only
  - keep examples aligned with the actual schema rules in `config_version = "v2"` instead of hand-wavy security wording
- Keep observability and troubleshooting examples tied to real event names and current routes, not aspirational surfaces.

### Phase 4: Use parallel subagents during execution, then integrate centrally

- After this plan is verified, use parallel subagents on disjoint doc slices:
  - one subagent for quick-start and operator-guide rewrites
  - one subagent for navigation cleanup and removal of stale verification/report pages
  - one subagent for interfaces/API/CLI fact-checking and any needed code-doc reconciliation
- Keep final editorial integration in the main agent so the book voice, navigation, and cross-links stay coherent.

### Phase 5: Verify the final state and complete Ralph closeout

- Re-read the changed docs in navigation order after edits to catch stale cross-links, repeated wording, or leftover filler/template structure.
- Run all required gates, not a subset:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- If any factual mismatch discovered during execution is really a product/code bug rather than a docs issue, either fix it in this task when it is tightly coupled to the operator workflow or create a follow-up bug immediately with the `add-bug` skill.
- Only after every gate passes:
  - tick off the acceptance items in this task file as completed where appropriate
  - set `<passes>true</passes>`
  - run `/bin/bash .ralph/task_switch.sh`
  - commit all changes, including `.ralph` updates, with the required `task finished [task name]: ...` prefix
  - `git push`

NOW EXECUTE
