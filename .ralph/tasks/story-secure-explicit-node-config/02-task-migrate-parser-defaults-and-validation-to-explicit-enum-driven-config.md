---
## Task: Migrate parser/defaults/validation to explicit enum-driven config semantics <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove hidden config inference by moving defaulting/validation behavior to explicit enum-driven semantics while preserving safe startup requirements.

**Scope:**
- Refactor `src/config/parser.rs` and `src/config/defaults.rs` to stop injecting implicit runtime identities (for example `postgres` user fallback).
- Introduce explicit default policy only where permitted by typed enums and safe documented defaults.
- Ensure parser errors are actionable when required secure fields are missing.
- Update config docs/comments/tests to reflect explicit requirements and no-inference contract.

**Context from research:**
- Current parser/default flow still fills values that should become explicit secure config inputs.
- Safe startup requires deterministic config sources and clear failure when mandatory values are missing.

**Expected outcome:**
- Config load path rejects incomplete secure configs.
- Any defaults that remain are enum-anchored and centrally defined, not scattered magic fallbacks.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] `src/config/parser.rs` has no inferred user/role/TLS identity fallback behavior outside explicit default enums
- [ ] `src/config/defaults.rs` is reduced to safe explicit defaults and does not silently synthesize sensitive identities
- [ ] Parse/validate error paths clearly identify missing required secure config fields
- [ ] Existing fixtures and sample configs are updated or intentionally rejected with explicit migration guidance
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
