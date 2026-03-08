---
name: k2-docs-loop
description: K2-led docs loop using raw Diataxis context, raw repo listings, and minimal Codex factual correction only.
---

# K2 Docs Loop

Use this skill when running the docs workflow for this repo.

Principles:

- K2 leads on what to write.
- Feed K2 raw context, not your own invented structure.
- Pipe the Diataxis summary markdown directly into K2.
- Codex must not interpret, reshape, improve, or critique K2 output based on taste or opinion.
- Codex must not do content design in between steps.
- Codex's job is to gather the proper raw files, provide them to K2, and only correct factual errors afterward.
- Only correct facts after drafting.
- Do not rewrite structure or content because of your own taste.
- If K2 chooses a page shape you would not have chosen, leave it alone unless the facts force a change.

Commands:

```bash
.agents/skills/k2-docs-loop/k2-docs-loop.sh summarize-diataxis
.agents/skills/k2-docs-loop/k2-docs-loop.sh choose-doc
.agents/skills/k2-docs-loop/k2-docs-loop.sh prepare-draft docs/reference/example.md
```

Artifacts:

- `diataxis-summary.md`: K2-authored markdown summary of the raw Diataxis `.rst` corpus.
- `choose-doc-prompt.md`: stable prompt template for choosing one next doc and requesting needed info.
- `write-doc-prompt.md`: stable prompt template for drafting one doc from a full gathered context file.

Workflow:

1. Run `summarize-diataxis` to refresh `diataxis-summary.md` from raw `.rst` files.
2. Run `choose-doc` to show the prompt and ask K2 to choose one new doc and report the exact raw files and exact optional runtime evidence needed.
3. Gather the exact raw files K2 requested. Prefer full files, not summaries.
4. Run `prepare-draft <docs/path.md> <requested files...>` to build a context file in `docs/tmp/` and pipe it into `docs/draft/<docs/path.md>`.
5. Inspect the draft only for factual correctness.
6. If facts are wrong or unsupported, correct those facts only.
7. Do not revise structure, tone, scope, or ordering just because you would have written it differently.

What `choose-doc` includes:

- current `docs/src` file listing
- full current `docs/src` file contents
- Diataxis summary markdown
- top-level manifests and docs config
- `src/` and `tests/` file listing
- `docker/` and `docs/` support file listing

What `prepare-draft` includes:

- verbose write prompt
- target docs path
- current `docs/src` file listing
- `docs/src/SUMMARY.md`
- Diataxis summary markdown
- top-level manifests and docs config
- `src/` and `tests/` file listing
- `docker/` and `docs/` support file listing
- every exact full file K2 requested for the page

Why manifests/config are included:

- `Cargo.toml` gives product name, crate shape, dependencies, and ecosystem context.
- `docs/book.toml` gives mdBook structure and confirms Mermaid support.
- These are raw repo facts, not interpretation.

Automation boundary:

- This skill already automates the stable parts of the loop.
- A future safe automation would be parsing K2's `choose-doc` response to extract file paths automatically into the next command.
- Another future safe automation would be running exact optional evidence commands K2 requested and appending their raw outputs into the draft prompt.
- Those are reliable to automate only if K2 keeps the response format stable.
