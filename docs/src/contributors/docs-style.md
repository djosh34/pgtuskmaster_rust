# Docs Authoring (Contributor Contract)

This chapter is a **writing contract** for contributor-facing documentation.

The goal is not “nice prose”. The goal is **transferable engineering understanding**: a reader should be able to reason about runtime behavior, find the relevant code, and make changes without accidentally breaking safety boundaries.

## Audience and scope

This book has two audiences:

- **Operators**: want “what to do” and “what to expect” in production.
- **Contributors**: want “how it works”, “why it is shaped this way”, and “where to change it”.

Contributor pages may go deep on:

- concrete call paths (who calls whom, in what order, under which conditions)
- state ownership and update semantics (who publishes, who consumes, what “latest” means)
- failure paths and recovery logic
- internal invariants and trust boundaries
- tests that protect the behavior.

Operator pages should avoid:

- module-by-module walkthroughs
- implementation details that churn
- internal verification/audit logs.

## Required chapter shape

Every contributor deep-dive chapter must follow this *minimum* structure (headings can vary, but the content must exist):

1. **Context**: what problem the chapter answers, and what “mental model” to keep.
2. **Core internals**: the actual call path(s) and state transitions, described in narrative paragraphs.
3. **Failure behavior**: what happens when inputs are missing, stale, conflicting, or unhealthy.
4. **Adjacent subsystem connections**: explicit links to at least one other contributor chapter, plus a short explanation of how the behaviors compose.
5. **Tradeoffs / sharp edges**: what is intentionally not handled, and what a contributor is likely to get wrong.
6. **Evidence pointers**: the key modules / tests to read next (function/type names, not line numbers).

## Paragraph-first density rules

Contributor docs are expected to be **paragraph-first**:

- Use paragraphs to explain “why” and “how”, not just “what”.
- Use bullets to summarize, not to replace explanation.
- Start sections with a short orienting paragraph before listing details.
- Prefer explicit transitions (“Because…”, “Therefore…”, “In the failure path…”) over disconnected fragments.

## Code snippet policy

Code snippets are allowed in contributor docs, but only when they materially improve comprehension.

Rules:

- Prefer showing **real identifiers** (functions, types, variants) from the codebase.
- Keep snippets short (usually 5–20 lines). If a snippet gets long, show only the part you are explaining and link to the owning module.
- Never paste full files or large blocks of boilerplate.
- A snippet must have surrounding text that explains:
  - what question the snippet answers
  - what to notice in the snippet
  - what not to assume from it.

## Claim language and verification expectations

Contributor docs must be correct, but they must also be *honest about uncertainty*.

Guidelines:

- Prefer “In the current implementation …” when describing a concrete code path.
- Avoid broad absolutes (“never”, “always”, “cannot”) unless supported by code + tests.
- Treat negative claims (“does not”, “cannot”, “never”) as **high-risk**:
  - either prove them with a guard/test
  - or rewrite them to bounded language (“is not expected to”, “is not currently wired to”).
- Comments and prior docs are hints, not proof.

If you change behavior, update contributor docs and tests together. If you change docs, verify that the described call paths still exist.

## Cross-linking and navigation

Contributor docs should read like a coherent deep technical guide, not a set of isolated pages.

Requirements:

- The reading order described in `contributors/index.md` must match `SUMMARY.md` exactly.
- Deep-dive chapters must link to adjacent chapters in their “Adjacent subsystem connections” section.
- If two chapters describe the same concept (for example, “trust” or “switchover”), define it once and link to the canonical explanation.

## PR review checklist (contributor docs)

Use this when reviewing changes to contributor chapters:

- Does the chapter explain at least one concrete call path with real identifiers?
- Are state ownership and update semantics described (publisher/subscriber, “latest” vs “changed”)?
- Are failure paths described (unhealthy DCS, unreachable Postgres, conflicting leader records, etc.)?
- Does it include an explicit “Adjacent subsystem connections” section with links?
- Are absolutes avoided or proven with code/tests?
- Do code snippets stay small and are they explained in surrounding prose?
