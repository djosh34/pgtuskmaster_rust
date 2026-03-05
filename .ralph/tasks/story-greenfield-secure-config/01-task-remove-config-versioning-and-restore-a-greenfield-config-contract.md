---
## Task: Remove config versioning and restore a greenfield config contract <status>not_started</status> <passes>false</passes>

<description>
Remove user-facing config versioning from the product and restore a simple greenfield config contract with no fake `v2` framing.

The agent must explore the current schema, parser, runtime assumptions, tests, and docs first, then implement the following fixed product decisions:
- there is no user-facing `config_version` field
- the parser must stop requiring or recognizing fake schema generations as the main product contract
- docs, examples, fixtures, debug/config surfaces, and parser errors must stop talking as if there were previously published config epochs
- the product exposes one real greenfield config contract

The work must update runtime parsing, tests, fixtures, examples, documentation, and config/debug surfaces together so the product no longer presents a fake versioned config story anywhere.

The parser and error messages must be tightened as part of this task:
- missing config must no longer produce guidance telling the user to set `config_version = "v2"`
- rejection paths must no longer mention `v1` / `v2` migration stories
- config load guidance must describe the one supported config contract directly

The agent should use parallel subagents after exploration for parser/schema, fixture/test migration, and docs updates.
</description>

<acceptance_criteria>
- [ ] User-facing config no longer requires, accepts, or documents `config_version`
- [ ] Runtime parsing and validation are aligned with a single greenfield config contract
- [ ] Parser errors and config load guidance no longer mention fake `v1` / `v2` schema generations
- [ ] Tests, fixtures, examples, and debug/config surfaces are updated to the non-versioned config shape
- [ ] Docs no longer describe or recommend fake schema generations
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
