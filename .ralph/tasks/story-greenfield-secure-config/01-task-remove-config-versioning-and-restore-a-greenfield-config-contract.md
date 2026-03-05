---
## Task: Remove config versioning and restore a greenfield config contract <status>not_started</status> <passes>false</passes>

<description>
Remove user-facing config versioning from the product and restore a simple greenfield config contract with no fake `v2` framing.

The agent must explore the current schema, parser, runtime assumptions, tests, and docs first, then implement a config model that reflects the reality of a greenfield project:
- no `config_version` field in user-facing config
- no docs or runtime behavior pretending there are legacy schema generations when none were ever published
- migration and parsing behavior simplified around the one real config contract that the product supports

The work must update runtime parsing, tests, fixtures, examples, and documentation together so the product no longer presents a fake versioned config story.

The agent should use parallel subagents after exploration for parser/schema, fixture/test migration, and docs updates.
</description>

<acceptance_criteria>
- [ ] User-facing config no longer requires or documents `config_version`
- [ ] Runtime parsing and validation are aligned with a single greenfield config contract
- [ ] Tests, fixtures, and examples are updated to the non-versioned config shape
- [ ] Docs no longer describe or recommend fake schema generations
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
