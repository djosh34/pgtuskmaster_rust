---
## Task: Rebuild contributor docs as a codebase navigation and design-contract guide <status>not_started</status> <passes>false</passes>

<description>
Rewrite the contributor documentation so it becomes a genuinely useful guide for understanding the codebase, subsystem boundaries, implementation approach, and design contracts.

The agent must explore the current codebase and docs first, then rebuild contributor docs around the exact things a new contributor needs to learn:
- how to navigate the codebase
- which modules own which responsibilities
- how the major systems are implemented
- what the important design contracts and invariants are
- how runtime data and control flow move between components
- how to locate the code for specific behaviors quickly
- how the implementation is split between runtime, process control, HA, DCS, APIs, config, tests, and harness code

This task must implement the following fixed product decisions:
- the contributor section must be a strong guide for learning the whole codebase
- it must explain code navigation, implementation paths, subsystem boundaries, and design contracts in terms grounded in the current code
- it must not become a vague essay, a chapter directory, or a dump of file names without explanation
- it must make the current contributor docs materially useful for understanding how the system is built and how to safely change it
- it must also be better to read and better to use as documentation, not just more complete

The agent should use parallel subagents after exploration to cover different subsystem areas and then unify the writing.
</description>

<acceptance_criteria>
- [ ] Contributor docs clearly explain codebase navigation and module ownership
- [ ] Contributor docs explain major subsystem implementation paths and design contracts in terms grounded in the current code
- [ ] Contributor docs explain how to find the code for important behaviors and how information/control flow moves across subsystems
- [ ] Contributor docs are useful for learning the system rather than merely listing chapters or files
- [ ] Writing quality is substantially improved over the current contributor section
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
