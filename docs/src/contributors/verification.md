# Verification Workflow

Documentation quality depends on two independent checks:

1. **writing quality**: does this teach the reader effectively?
2. **factual correctness**: does this match the code, tests, and runtime behavior?

This chapter is the contributor-facing workflow for verifying docs. Verification evidence belongs in the task or PR artifacts that produced the change, not in a stale user-facing “verification report” chapter.

## The skeptical mindset

This repo is about HA behavior and safety boundaries. “Sounds plausible” is not evidence.

Use this mindset:

- assume every non-trivial behavior claim may be wrong
- treat comments and prior docs as hints, not proof
- prefer primary sources:
  - code paths that actually run
  - tests that will fail if behavior changes
  - runtime evidence such as logs and debug snapshots

When evidence is missing, do not guess. Rewrite the statement to match what is proven, or remove it.

## What counts as a claim

A claim is any statement that implies:

- behavior
- safety guarantees
- operational expectations
- interface contracts
- absence assertions such as “never”, “does not”, or “cannot”

Even contributor docs need evidence for those statements.

## Verification workflow

### 1. Extract the claims you are making

Before verifying, reduce each paragraph to a few concrete behavior claims. Decide whether each claim should be proven by code, tests, or runtime output.

### 2. Find the evidence anchors

Anchor each claim to at least one of:

- a concrete function, type, or module
- a test that would fail if the claim stopped being true
- a runtime output surface such as `/ha/state` or `/debug/verbose`

Prefer stable identifiers over line numbers.

### 3. Make the claim match the evidence

If the evidence only partially supports the claim, narrow the language. For example:

- add preconditions
- replace universal wording with current-implementation wording
- remove guarantees the code does not enforce

If no evidence exists, delete the claim or add the missing test or guard.

### 4. Be extra strict with absence claims

Statements like “cannot” or “never” are fragile. Keep them only when a guard or test enforces them. Otherwise rewrite them into bounded language.

### 5. Keep evidence out of operator docs

Operator docs should contain the corrected guidance, not the audit trail. Keep the detailed proof in task artifacts, PR notes, or contributor evidence files.

## Claim outcomes

For any reviewed claim, the outcome should be one of:

- **verified**: supported by current code, tests, or runtime evidence
- **rewritten**: changed to match the evidence
- **removed**: deleted because it was wrong or misleading
- **deferred**: only acceptable with an explicit follow-up task

## Useful companion chapters

- [Codebase Map](./codebase-map.md)
- [HA Decision and Action Pipeline](./ha-pipeline.md)
- [API and Debug Contracts](./api-debug-contracts.md)
