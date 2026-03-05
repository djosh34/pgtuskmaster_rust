---
## Task: Audit and replace magic numbers project-wide <status>not_started</status> <passes>false</passes> <priority>low</priority>

<description>
Audit the project for unexplained magic numbers and replace them with explicit typed constants, configuration, or otherwise well-justified named values.

The agent must explore the whole codebase first, not only HA, then implement the following fixed product decisions:
- this is a project-wide cleanup, not only an `src/ha/state.rs` cleanup
- unexplained magic numbers should be checked everywhere in runtime code, tests, harness code, and supporting modules
- values that are real product knobs should become explicit config or typed settings where appropriate
- values that are fixed implementation constants should become clearly named constants with obvious ownership
- purely arbitrary numeric literals that remain must be justified by local meaning, not left as unexplained inline numbers

This is intentionally low priority and should not preempt the architectural rewrite stories, but the final codebase should not keep accumulating unexplained numeric literals.

The agent should use parallel subagents after exploration to audit different codebase slices and then apply the cleanup coherently.
</description>

<acceptance_criteria>
- [ ] Project-wide audit covers runtime code, tests, harness code, and supporting modules rather than only HA
- [ ] Unexplained magic numbers are replaced by config, typed settings, or clearly named constants where appropriate
- [ ] Remaining numeric literals are locally justified by obvious meaning rather than unexplained inline usage
- [ ] The cleanup does not introduce bogus configurability where fixed constants are the better design
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
