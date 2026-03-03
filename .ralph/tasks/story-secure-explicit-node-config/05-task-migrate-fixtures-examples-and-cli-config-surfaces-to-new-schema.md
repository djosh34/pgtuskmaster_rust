---
## Task: Migrate fixtures/examples/CLI config surfaces to the secure explicit schema <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Align all config producers/consumers (tests, examples, CLI entrypoints) with the expanded schema and explicit secure requirements.

**Scope:**
- Update test fixtures under `src/` and `tests/` to provide full explicit config values.
- Update `examples/` and any contract fixture builders to compile with new config fields.
- Update CLI/config loading UX and docs to reflect new required fields and migration path.
- Add focused migration tests covering missing/invalid role auth combinations and TLS material.

**Context from research:**
- Existing fixtures rely on old defaults and partial config assumptions.
- `--all-targets` often fails if examples/fixtures are not updated in same patch.

**Expected outcome:**
- All tests/examples/config consumers build and run against one explicit secure config contract.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] All in-repo fixtures/examples compile with new required config fields populated
- [ ] CLI/config load paths provide clear guidance for missing required secure fields
- [ ] Migration tests cover invalid/missing role auth and TLS material combinations
- [ ] No lingering legacy config field usage remains in examples/tests without explicit migration rationale
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] BDD features pass (covered by `make test`).
</acceptance_criteria>
