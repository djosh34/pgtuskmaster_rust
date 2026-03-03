---
## Task: Ultra-high-priority split ultra-long e2e tests into shorter parallel real-binary tests <status>not_started</status> <passes>false</passes> <priority>ultra-high</priority>

<description>
**Goal:** Replace the current ultra-long HA e2e stress tests with multiple shorter real-binary e2e tests that preserve full coverage and must run in parallel.

**Scope:**
- Decompose each current ultra-long scenario into smaller independent real-binary e2e tests with narrow objectives.
- Preserve all existing behavioral coverage and assertions from the original long scenarios.
- Ensure resulting short tests are parallel-safe and designed to run concurrently (no serial-only exemptions).
- Keep the ultra-long-only target small over time by moving shortened scenarios back into `make test`.
- Document mapping from each original long scenario to its replacement short tests.

**Context from research:**
- A small number of very long HA e2e tests dominate runtime and block development flow.
- The project requires real binaries in these checks, but long duration should not force serial developer loops.
- New short tests must still catch the same failures, not reduce assurance.

**Expected outcome:**
- Ultra-long scenarios are functionally replaced by a set of shorter real-binary e2e tests.
- Short replacements are parallelized by default and become part of regular `make test` flow when stable.
- `make test-long` shrinks to only truly unavoidable long-duration tests.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] For every current ultra-long scenario, a traceable set of shorter real-binary e2e tests exists that covers all prior assertions.
- [ ] New shorter tests are parallel-safe and executed in parallel with no serial-only exception path.
- [ ] Coverage mapping artifact is added (old long scenario -> new short test set).
- [ ] Any failure discovered only in `make test-long` gains a new short real-binary e2e regression test in `make test`.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly
</acceptance_criteria>
