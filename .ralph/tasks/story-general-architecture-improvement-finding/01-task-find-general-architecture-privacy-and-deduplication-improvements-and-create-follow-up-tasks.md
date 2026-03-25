## Task: Find General Architecture, Privacy, And Deduplication Improvements And Create Follow-Up Tasks <status>completed</status> <passes>true</passes>

<priority>medium</priority>
<blocked_by>Full completion of `.ralph/tasks/story-ctl-operator-experience/10-task-collapse-dcs-behind-a-single-private-component-and-read-only-dcs-view.md`</blocked_by>

<description>
**Goal:** After the DCS refactor task is fully complete, perform a fresh cross-cutting architecture review of the rest of the repository to find other high-value cleanup/refactor targets in the same spirit: make packages and modules more private, reduce interfaces between components to the smallest useful surface, remove duplicated logic and duplicated representations, untangle spaghetti ownership/dependencies, and identify legacy or overgrown code/tests that should be simplified or deleted. This task is not the implementation of those refactors. Its job is to find multiple concrete problem areas and create one focused follow-up implementation task for each area.

**Higher-order goal:** The repository should keep converging toward small private components with narrow typed boundaries, single ownership, less code, less duplication, and clearer tests. The DCS task is the immediate prerequisite and example of the kind of architectural improvement being sought here, but this task must look for the next set of problem areas elsewhere in the codebase rather than continuing DCS itself.

**Original request that this task must preserve and carry forward into follow-up tasks:**
- "just like the dcs refactor task, i want a fully general improvement finding task"
- "make packages/mods more private"
- "reduce code interface between other components, make as small as possible interface"
- "find/checks/refactors radically internally to reduce code duplication. tries to simplify logic, de-spagthify, clean up old legacy logic/tests/shit"
- "untangle spagethi dependencies: just like dcs was controlled in many parts of the code, instead of a single worker. Find some other component that can be untangled, made almost fully private except very scoped/small interface, and thereby massively improving code quality, testability, reducing code in general (less code = better), cleaning up shit, making it more readable"
- "this task should evaluate and find multiple areas for improvement, and for each use the add-task-as-agent skill to create a task that actually fixes it"
- "that task must include the original request, but must not dictate exactly how to do it, the research must be done during the created task itself, and this task must not dictate a solution, only find a problem"
- "(must be done after that dcs task, e.g. is blocked by)"

**Important boundary of this task:**
- This is a research-and-task-creation task, not a code refactor task.
- Do not implement the discovered improvements in this task.
- Do not create one giant omnibus refactor task. Create multiple focused follow-up tasks, one per problem area.
- Each follow-up task must describe a real problem with enough repo context to stand alone, but it must not hard-code a specific refactor design or dictate the exact solution architecture. The implementation research is meant to happen inside the created follow-up task itself.

**Scope:**
- Review the current repository after DCS refactor completion, primarily under `src/`, plus any related tests/docs/config/examples that expose the same architectural smell.
- Look for components that currently leak too much surface area through `pub`, `pub(crate)`, re-exports, shared raw types, stringly helper APIs, or direct cross-component mutation.
- Look for places where one conceptual subsystem is controlled from many files instead of one owner component.
- Look for duplicated logic, duplicated state representations, duplicated test scaffolding, duplicated config shapes, duplicated parsing/normalization layers, or old compatibility code that should be removed.
- Look for legacy or low-value tests/helpers/docs/examples that keep old architecture alive or make the code harder to simplify.
- For each discovered area that is independently actionable, create a new implementation task in this same story directory using the `add-task-as-agent` skill at `.agents/skills/add-task-as-agent/SKILL.md`.

**Search heuristics and problem patterns to evaluate:**
- modules that export broad surfaces even though most consumers only need one read-only view or one narrow command/query handle
- components with multiple owners or multiple call sites mutating the same conceptual state
- components with parallel type trees, duplicated DTO/domain/storage shapes, or repeated conversion code that suggests missing central ownership
- files where many modules reconstruct the same semantics from internal records instead of depending on one typed boundary
- cross-module helpers that pass raw maps/paths/strings/options around where stronger types or a smaller interface would eliminate ambiguity
- legacy tests that encode old internal architecture instead of externally meaningful behavior
- any area where deleting code and collapsing layers would likely improve readability, testability, and correctness

**Required output of this task:**
- Create multiple follow-up task files under `.ralph/tasks/story-general-architecture-improvement-finding/`.
- "Multiple" here means more than one. At least two independently actionable follow-up tasks must be created, and more should be created if the review finds additional strong candidates.
- Each follow-up task must be focused on one problem area only, not a vague pile of unrelated cleanup.
- Each follow-up task must be created via the `add-task-as-agent` skill format and must stand fully on its own.
- Each follow-up task must include the original request above so the architectural intent is preserved.
- Each follow-up task must describe:
  - the problem area and why it is a problem now
  - concrete repo evidence and current code locations that show the problem
  - the higher-order goal of reducing interface size, increasing privacy, reducing duplication, untangling ownership, or deleting legacy code
  - expected outcome / acceptance criteria
- Each follow-up task must not lock the implementer into an exact solution. It should name the problem and the boundaries that need to improve, not prescribe the exact internal design.

**Context from current repo state and why this task is blocked:**
- The immediate predecessor is `.ralph/tasks/story-ctl-operator-experience/10-task-collapse-dcs-behind-a-single-private-component-and-read-only-dcs-view.md`.
- That DCS task already establishes the pattern the user wants: one owner, private internals, narrow public boundary, less duplicated meaning, and aggressive simplification rather than hiding the same complexity.
- This task must start only after that DCS work is done, because the point is to inspect the next most valuable areas after the largest active untangling effort is complete. Running this task before then would create overlap and stale findings.

**How to decide whether a finding deserves a follow-up task:**
- The problem must be architectural or structural, not just a tiny local cleanup.
- The likely payoff should be meaningful in at least one of:
  - stronger privacy / smaller module surface
  - reduced inter-component coupling
  - less duplicated code or duplicated representations
  - clearer ownership and less spaghetti coordination
  - simpler or more behavior-focused tests
  - deletion of legacy or compatibility code
- The area must be narrow enough that one future task can reasonably own it.
- If two findings are really the same architectural problem, create one task, not two near-duplicates.
- If a finding is speculative and not supported by concrete repo evidence, do not create a task for it yet.

**Rules for the follow-up tasks created from this task:**
- Use descriptive filenames under `.ralph/tasks/story-general-architecture-improvement-finding/`.
- Use the `add-task-as-agent` skill rather than inventing a new task format.
- Include exact file/module references discovered during the review so the task is independently understandable.
- Include the original request text in the created task.
- Do not dictate exact implementation mechanics such as "introduce type X" or "split into files A/B/C" unless the problem is impossible to describe honestly without that detail. Prefer describing the current problem and the required end-state properties instead.
- Prefer one task per component/subsystem/problem area.
- Choose priority intentionally based on impact; do not automatically mark every follow-up task as high priority.

**Out of scope:**
- Do not continue editing the DCS refactor itself here.
- Do not perform the actual code cleanup/refactors in this task.
- Do not produce purely abstract notes without creating real follow-up task files.
- Do not create tasks that are only style nits, trivial renames, or micro-cleanups without clear architectural value.

**Expected outcome:**
- The repository gains a new story containing this research/meta-task plus multiple follow-up implementation tasks for concrete architectural cleanup opportunities discovered after DCS.
- Each follow-up task is self-contained, grounded in current repo evidence, aligned with the user’s original request, and framed around a problem to solve rather than a pre-decided solution.
- The resulting backlog gives the project several high-value next refactor targets focused on privacy, smaller interfaces, less duplication, less spaghetti ownership, and deletion of legacy code.

</description>

<acceptance_criteria>
- [x] `.ralph/tasks/story-ctl-operator-experience/10-task-collapse-dcs-behind-a-single-private-component-and-read-only-dcs-view.md` is fully complete before this task begins.
- [x] The review covers the current post-DCS repo broadly enough to evaluate multiple candidate subsystems/components rather than only one local file cluster.
- [x] The review explicitly looks for problems in module privacy, over-wide interfaces, duplicated logic/representations, tangled ownership, and legacy code/tests that keep bad architecture alive.
- [x] At least two independently actionable follow-up task files are created under `.ralph/tasks/story-general-architecture-improvement-finding/` using the `add-task-as-agent` skill.
- [x] Each created follow-up task includes the original user request text captured in this task so the architectural intent is preserved.
- [x] Each created follow-up task is self-contained and includes concrete repo evidence, affected files/modules, higher-order goal, and expected outcome.
- [x] Each created follow-up task describes a problem to solve and the end-state properties that must improve, without dictating an exact implementation design.
- [x] No implementation refactor code is performed as part of this task beyond creating the task markdown files themselves.
- [x] The created follow-up tasks are scoped as focused independent work items rather than one omnibus cleanup task.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
