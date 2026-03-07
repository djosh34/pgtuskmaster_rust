# Docs Verification and Authoring

Contributor docs in this repo are part of the engineering surface, not polish added after the fact. A good page should help a new engineer answer three questions quickly: which code owns this behavior, what invariants keep it safe, and what evidence proves the description is still true.

That means contributor docs have two simultaneous quality bars. They must teach well, and they must stay grounded in the code, tests, and runtime signals that actually exist today. Verification evidence belongs in the task or PR artifacts that produced the change, not in a stale user-facing appendix.

## What a contributor chapter must do

Each contributor deep-dive should help the reader navigate real implementation paths rather than admire an architecture summary. In practice that means every page needs to answer a concrete contributor job, point at real identifiers, and explain the boundaries a change must preserve.

Use this minimum chapter contract when you write or review contributor docs:

1. Start with context: what question the page answers and what mental model matters.
2. Explain core internals with the real call path, state ownership, or projection path.
3. Describe failure behavior, not just the happy path.
4. Link to adjacent subsystems so the reader knows where the next boundary lives.
5. Name the sharp edges or tradeoffs a contributor is likely to get wrong.
6. End with evidence pointers: files, functions, types, or tests to open next.

If a page cannot satisfy that shape, it is probably either too shallow to keep or too broad to be one chapter.

## The skeptical verification mindset

This codebase controls HA behavior, DCS coordination, and real Postgres side effects. “Sounds plausible” is not enough. Treat prior docs, comments, and old task notes as hints, not proof.

When you verify a contributor page, prefer primary evidence:

- code paths that actually run, such as `run_node_from_config`, `run_workers`, `ha::decide::decide`, `apply_effect_plan`, or `api::worker::route_request`
- tests that would fail if the described contract changed
- runtime evidence such as `/ha/state`, `/debug/verbose`, or structured worker events

When evidence is missing, narrow the claim, delete it, or add the missing test. Do not keep a strong sentence just because it reads well.

## What counts as a risky claim

Any sentence that implies behavior, safety, interface shape, or recovery expectations needs evidence. Absence claims are especially risky: words like “never”, “cannot”, or “does not” should survive only when the code or tests enforce them.

The safest default wording is often “In the current implementation ...” followed by the exact owned code path. That keeps the page concrete without promising guarantees the code does not currently enforce.

## Verification workflow

Use this loop when you update contributor docs:

1. Extract the claims in the draft paragraph by paragraph.
2. Anchor each claim to code, a test, or a runtime signal.
3. Rewrite the prose so it matches the evidence exactly.
4. Remove or defer any claim that still lacks proof.
5. Record the audit trail in the active task or PR notes, not in the book itself.

For each claim, the outcome should be one of four states:

- `verified`: the current code or tests support it.
- `rewritten`: the original wording was too strong or too vague, so it now matches the evidence.
- `removed`: the claim was misleading or unsupported.
- `deferred`: only acceptable if you also create an explicit follow-up task.

## Writing rules that keep docs useful

Contributor docs should be paragraph-first and navigation-heavy. A bullet list can summarize, but it should not replace the explanation of why a call path or invariant matters. Start sections with short orienting paragraphs, then use bullets only where they compress concrete evidence or entrypoints.

Keep code snippets rare and small. Show real identifiers, only the part that matters, and explain what to notice. If a snippet grows into boilerplate, point to the owning file instead.

Keep contributor and operator docs distinct. Contributor pages may discuss module ownership, state channels, decision logic, and test boundaries. Operator pages should focus on what to do and what to expect, without dragging readers through implementation churn or audit notes.

## Review checklist

Before you merge contributor-doc changes, ask these questions:

- Does the page explain at least one real call path or projection path with current identifiers?
- Does it say who owns the relevant state and how consumers get it?
- Does it describe failure behavior or conservative fallback behavior?
- Does it link the reader to the next subsystem boundary?
- Are broad absolutes removed or explicitly proven?
- Is the audit evidence captured in the task or PR artifact rather than hidden in prose?

## Adjacent subsystem connections

This chapter is the maintenance contract for the rest of the contributor section. Use it together with:

- [Codebase Map](./codebase-map.md) when you need to decide which module should own a change.
- [Worker Wiring and State Flow](./worker-wiring.md) when a claim depends on published-state ownership.
- [HA Decision and Effect-Plan Pipeline](./ha-pipeline.md) when you are verifying HA or side-effect claims.
- [API and Debug Contracts](./api-debug-contracts.md) when you are documenting read/write surfaces or debug projections.
- [Testing System Deep Dive](./testing-system.md) when a claim depends on a test boundary rather than a single code path.
