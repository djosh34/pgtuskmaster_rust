---
name: create-docs
description: Build documentation-writing or documentation-rewrite workflows. Use when creating a docs skill, importing source documentation into local references, or grounding later docs work in the Diataxis source material bundled with this skill.
---

# Create Docs

Use this skill when creating or rebuilding project documentation around Diataxis while keeping the local mdBook toolchain.

If the docs already exist and the task is to maintain or revise them because code changed, behavior changed, or a bug exposed stale docs, use `update-docs` instead of treating it as a greenfield docs-creation problem.

Keep the guidance lean. Do not restate obvious repository facts that can be learned by reading the tree or the bundled source pages.

Ground the work in Diataxis. Re-read the source pages before each docs run, not just once at the start of the story.

Minimum re-read set before continuing any docs task:

- `references/diataxis.fr/start-here/index.md`
- `references/diataxis.fr/compass/index.md`
- `references/diataxis.fr/how-to-use-diataxis/index.md`
- the page for the form you are about to write:
  - `references/diataxis.fr/tutorials/index.md`
  - `references/diataxis.fr/how-to-guides/index.md`
  - `references/diataxis.fr/reference/index.md`
  - `references/diataxis.fr/explanation/index.md`

Use the original Diataxis rules, not a fuzzy memory of them:

- ask the compass questions: `action or cognition?` and `acquisition or application?`
- keep the four forms separate instead of mixing them on one page
- tutorials are lessons and a carefully-managed path for study
- how-to guides are goal-oriented directions for work and should contain `action and only action`
- reference should `describe and only describe` and mirror the structure of the machinery
- explanation should provide context, background, reasons, and why
- `do not create empty structures for tutorials/howto guides/reference/explanation with nothing in them`
- `work one step at a time`: choose something, assess it, decide one next action, do it

Local rules for this repo:

- keep mdBook as the engine
- Mermaid diagrams are allowed when they clarify something and the agent can verify them
- keep verification artifacts out of `docs/src/`; place them only under `docs/verifications/`
- during docs creation, write at most 5 pages per run, append progress, then quit immediately so the next run starts fresh from the Diataxis references again

Local source material is bundled under `references/`.
Start with `references/diataxis-import-index.md`, then open only the specific Diataxis pages needed for the task at hand.
