---
## Task: Align doc file order and names with the rendered site structure <status>not_started</status> <passes>false</passes>

<description>
Make the docs source tree easier to navigate by aligning file names and ordering conventions with the rendered website structure.

The agent must explore the current docs tree, `SUMMARY.md`, and rendered navigation intent first, then implement the following fixed product decisions:
- docs source file naming should help a contributor understand the rendered order without guessing
- file names and ordering conventions should match the website structure closely enough that the source tree is not fighting the book navigation
- the source layout should become easier to navigate for humans working in the repo, not only for mdBook
- this should be done without adding ugly clutter or arbitrary complexity that makes file paths worse

This task exists because docs should be easier to work on directly in the repo, and the source tree should reflect the reading order instead of obscuring it.

The agent should use parallel subagents after exploration if that materially helps with doc-tree cleanup and link updates.
</description>

<acceptance_criteria>
- [ ] Docs source naming and ordering conventions are aligned with the rendered site structure
- [ ] `docs/src/SUMMARY.md` and the source tree no longer fight each other in obvious ordering/naming ways
- [ ] The docs source tree is easier to navigate directly from the filesystem
- [ ] Links and references remain correct after any renames or reordering
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
