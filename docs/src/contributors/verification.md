# Verification Workflow

Documentation quality depends on two independent checks:

1. **writing quality** (does this teach the reader effectively?)
2. **factual correctness** (does this match the code/tests/runtime behavior?)

This chapter is a contributor-facing operational workflow for verifying docs. It complements (and links to) the canonical verification section in this book:

- [Verification](../verification/index.md)

The intent is to make verification a routine engineering habit, not a one-off audit task.

## The skeptical mindset (why verification is strict)

This repo is about HA behavior and safety boundaries. “Sounds plausible” is not evidence.

Use this mindset:

- assume every non-trivial behavior claim may be wrong
- treat comments and prior docs as hints, not proof
- prefer primary sources:
  - code paths (functions/types that actually run)
  - tests (that will fail if behavior changes)
  - runtime evidence (logs, debug snapshot outputs in e2e scenarios).

When evidence is missing, do not guess. Rewrite to bounded language or remove the claim.

## What counts as a “claim”

A claim is any statement that implies:

- behavior (“the system does X when Y happens”)
- safety guarantees (“this cannot happen”)
- operational expectations (“this will converge in N seconds”)
- interface contracts (“this endpoint always returns …”)
- negative/absence assertions (“never”, “does not”, “cannot”).

Even in contributor docs, claims must be anchored to reality.

## Verification workflow (step-by-step)

Use this workflow whenever you touch docs that describe behavior:

### 1) Extract the claims you are making

Before verifying, list the specific sentences that imply behavior.

Practical technique:

- turn a paragraph into 3–8 bullets of “claims” in your head (or in your PR notes)
- for each claim, write the expected evidence source (“code”, “test”, or “runtime proof”).

### 2) Find the evidence anchors

Anchor each claim to at least one of:

- a concrete function/type/module name
- a test that would fail if the claim changed
- a runtime output surface (for example `/ha/state` or `/debug/verbose`) that exposes the relevant state.

Prefer concrete identifiers over line numbers. Line numbers churn; module paths and function/type names are more stable.

### 3) Make the claim match the evidence (not the other way around)

If evidence matches, keep the claim.

If evidence partially matches, rewrite:

- narrow the scope (“in the current implementation …”)
- remove implied guarantees
- explain preconditions (“when DCS trust is full quorum …”).

If evidence does not exist, either:

- remove the claim, or
- add the missing test/guard (and then keep the claim).

### 4) Handle absence claims with extra skepticism

Absence claims are high-risk because they are easy to accidentally violate later.

Rule:

- If the claim is “cannot/never/does not”, require a guard or a test.
- Otherwise rewrite to bounded guidance:
  - “is not expected to”
  - “is not currently wired to”
  - “is not part of the supported contract”.

### 5) Record artifacts without polluting operator docs

Operator docs should contain the corrected content, not the internal audit narrative.

When you need to keep detailed evidence trails (for example a list of claims and their proof links), keep them in contributor artifacts or task evidence (for example in the `.ralph/` task evidence flow used by this repo’s engineering loop).

## Claim outcomes (what “done” looks like)

For any reviewed claim, the outcome should be one of:

- **verified**: supported by current code/tests/runtime evidence
- **rewritten**: changed to match evidence with bounded language
- **removed**: deleted because it was wrong or misleading
- **deferred**: only allowed if you create an explicit follow-up task that will force resolution (avoid “TODO forever”).

## Adjacent subsystem connections

Verification touches every subsystem. These chapters are the most useful companions when verifying a claim:

- [Codebase Map](./codebase-map.md): choose the correct owner module for a behavior and avoid cross-module “wishful descriptions”.
- [HA Decision and Action Pipeline](./ha-pipeline.md): verify lifecycle claims by tracing `ha::decide` and dispatch behavior.
- [API and Debug Contracts](./api-debug-contracts.md): verify interface claims (routes, auth, debug surfaces) against the worker/controller/view code paths.
