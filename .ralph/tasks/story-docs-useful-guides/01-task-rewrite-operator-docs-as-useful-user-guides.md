## Task: Rewrite operator docs as useful user guides and remove horror pages <status>not_started</status> <passes>false</passes>

<description>
Rewrite the non-contributor documentation so it reads like a strong operator/product guide instead of a thin or awkwardly templated book.

The agent must explore the current docs and implementation first, then rewrite the docs around what actually helps a user understand and operate the system.

This task must implement the following fixed product decisions:
- remove report-style, verification-report, and similar pages from the docs entirely; they must not remain in the book, appendix, or normal navigation
- remove cringe/filler writing and stop using mechanical prose templates such as forced "Why this exists", "Tradeoffs", and "When this matters" sections
- rewrite the operator-facing docs into direct, useful prose that is substantially richer where important information is currently missing
- make the docs feel like actual good docs: better to read, better to use, easier to navigate, and more helpful when trying to understand or operate the system
- keep the docs grounded in the real product and implementation rather than speculative framing, fake maturity signals, or fake legacy/version framing
- ensure the recommended setup and examples are secure by default
- do not present container-first quick-start or Docker-first setup as current product reality before the existing container deployment story lands
- keep the book focused on helping a user understand what the product does, how to run it, how to reason about it, and what to check when things go wrong

This is a docs quality and usefulness rewrite, not just a structure tweak. The agent should use parallel subagents after exploration to rewrite the book in coherent slices.
</description>

<acceptance_criteria>
- [ ] `docs/src/verification/index.md` and verification-report style pages are removed from the docs book rather than merely rewritten or relocated
- [ ] Operator-facing docs are rewritten in direct, useful prose without filler or forced template sections
- [ ] Important missing operational and product information is added where needed, especially where the current docs are thin, weird, or hard to use
- [ ] The docs are materially better to read and use as documentation, not merely more verbose
- [ ] Security recommendations and examples in operator docs align with the intended secure default approach
- [ ] Docs do not claim Docker/container-first setup before the existing container deployment story actually lands
- [ ] Docs do not present fake product maturity, fake legacy/version framing, or speculative rationale that is not grounded in the implementation
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
