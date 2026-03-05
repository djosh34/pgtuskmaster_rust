---
## Task: Rewrite operator docs as useful user guides and remove horror pages <status>not_started</status> <passes>false</passes>

<description>
Rewrite the non-contributor documentation so it reads like a strong operator/product guide instead of a thin or awkwardly templated book.

The agent must explore the current docs and implementation first, then rewrite the docs around what actually helps a user understand and operate the system.

Target outcomes:
- remove report-style, verification-report, and other horror pages from the docs path entirely
- remove cringe/filler writing and stop forcing mechanical section templates like "Why this exists / Tradeoffs / When this matters" where they make the docs worse
- make the docs user-friendly, better written, and substantially more informative where important information is currently missing
- keep the docs grounded in the real product and implementation rather than speculative framing or fake maturity signals
- ensure the recommended setup and examples are secure by default
- keep container-first quick-start claims out of the docs until container support actually exists in the existing container deployment story

This is a docs quality and usefulness rewrite, not just a structure tweak. The agent should use parallel subagents after exploration to rewrite the book in coherent slices.
</description>

<acceptance_criteria>
- [ ] Report-style and verification-report pages are removed from the docs book and main navigation
- [ ] Operator-facing docs are rewritten in direct, useful prose without filler or forced template sections
- [ ] Important missing operational and product information is added where needed
- [ ] Security recommendations in operator docs are aligned with the intended secure default approach
- [ ] Docs do not claim Docker/container-first setup before the existing container deployment story actually lands
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
