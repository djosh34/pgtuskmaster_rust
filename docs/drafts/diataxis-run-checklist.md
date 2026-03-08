# Diataxis Docs Run Checklist

Use this checklist to package facts for a single docs run. It is process scaffolding only, not page prose.

1. Re-read the mandatory Diataxis sources for this run.
   - `start-here`
   - `compass`
   - `how-to-use-diataxis`
   - the form-specific source for the pages in this run
   - task `01`
   - any earlier story task that this task explicitly depends on
2. Pick at most 3 pages to draft or revise in this run.
   - the story-task cap overrides the generic 5-page cap mentioned in the reusable docs skills
3. Gather facts from the repository, not from prior prose.
   - code paths
   - config files and defaults
   - tests
   - commands and runnable behavior
   - existing docs pages only as non-authoritative revision inputs after repo re-checking
4. Package a rich K2 context payload.
   - page goal
   - audience and user need
   - verified facts
   - facts that must not be invented or changed
   - Diataxis constraints for the current form
   - required headings or page boundaries
   - existing draft text only when revising, and never as a substitute for repo facts
5. Use `ask-k2-docs` for every prose draft and prose revision.
6. Use `update-docs` whenever revising or promoting an existing docs page or `docs/src/SUMMARY.md`.
7. Check the K2 output yourself.
   - remove invented facts
   - remove mixed-form drift
   - keep diagram requests as placeholders
8. Tick task checkboxes only after the work really happened.
9. Quit immediately after the capped run work is complete.
