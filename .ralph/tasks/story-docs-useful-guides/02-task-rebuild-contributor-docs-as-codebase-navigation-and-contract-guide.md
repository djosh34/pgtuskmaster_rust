---
## Task: Rebuild contributor docs as a codebase navigation and design-contract guide <status>not_started</status> <passes>false</passes>

<description>
Rewrite the contributor documentation so it becomes a genuinely useful guide for understanding the codebase, subsystem boundaries, implementation approach, and design contracts.

The agent must explore the current codebase and docs first, then rebuild contributor docs around what a new contributor actually needs to learn:
- how to navigate the codebase
- which modules own which responsibilities
- how the major systems are implemented
- what the important design contracts and invariants are
- how runtime data and control flow move between components

This should not become a vague essay or a dump of file names. It should be a strong learning guide for getting the whole codebase, with concrete references to real modules and contracts.

The agent should use parallel subagents after exploration to cover different subsystem areas and then unify the writing.
</description>

<acceptance_criteria>
- [ ] Contributor docs clearly explain codebase navigation and module ownership
- [ ] Contributor docs explain major subsystem implementation paths and design contracts in terms grounded in the current code
- [ ] Contributor docs are useful for learning the system rather than merely listing chapters or files
- [ ] Writing quality is substantially improved over the current contributor section
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
