# Verification Workflow

Documentation quality depends on two independent checks:

1. writing quality and structural clarity
2. factual verification against code, tests, and runtime behavior

## Skeptical verification model

- assume every non-trivial claim may be wrong
- verify with primary evidence from implementation and tests
- treat comments and prior docs as hints, not proof
- downgrade or remove claims that cannot be evidenced

## Claim handling outcomes

- verified
- rewritten to bounded wording
- removed
- deferred with explicit follow-up task

## Process boundary

Verification mechanics are tracked in contributor artifacts. Operator docs should contain the corrected outcomes, not the internal audit narrative.
