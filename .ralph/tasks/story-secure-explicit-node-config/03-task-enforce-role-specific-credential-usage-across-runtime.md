---
## Task: Enforce role-specific credential usage across runtime operations <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Ensure each runtime function uses only its designated role (`superuser`, `replicator`, `rewinder`) and corresponding auth mode from config.

**Scope:**
- Trace and update credential usage in HA/process/pginfo/rewind/postgres control paths.
- Replace shared or hardcoded connection identities with explicit role-based selection.
- Ensure rewinder flows use rewinder role only, replication flows use replicator role only, and admin/system operations use superuser only.
- Add/expand type-safe interfaces preventing accidental cross-role credential use.

**Context from research:**
- Current runtime wiring includes hardcoded/default connection identity assumptions and does not model all role auth pathways explicitly.
- This creates risk of privilege bleed across operations.

**Expected outcome:**
- Runtime operations are least-privilege by construction and enforced in code paths.
- Regression tests fail on any role-misuse or fallback identity usage.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] All PostgreSQL connection builders select credentials via explicit role-kind mapping from config
- [ ] Rewind implementation uses rewinder role/auth only
- [ ] Replication/follow operations use replicator role/auth only
- [ ] Superuser-only actions are isolated and do not fallback to implicit defaults
- [ ] Tests cover role misuse prevention and least-privilege mapping behavior
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
