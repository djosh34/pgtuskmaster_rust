---
name: k2-docs-loop
description: K2-led docs loop using raw Diataxis context, raw repo listings, and minimal Codex factual correction only.
---

# K2 Docs Loop

Use this skill when running the docs workflow for this repo.

Principles:

- K2 leads on what to write.
- Feed K2 raw context, not your own invented structure.
- You must not interpret, reshape, improve, or critique K2 output based on taste or opinion.
- You must not do content design in between steps.
- Your job is to gather the proper raw files, provide them to K2, and only correct factual errors afterward.
- Only correct facts after drafting.
- Do not rewrite structure or content because of your own taste.
- If K2 chooses a page shape you would not have chosen, leave it alone unless the facts force a change.
- K2 can take long to complete, and is expected and no reason to worry
- K2 can fail in its output (e.g. tool instead of text), that's also ok, please try again with k2, do not try to do it yourself.
- Never run mdbooks directly, instead use the provided make files only!

Commands:

```bash
.agents/skills/k2-docs-loop/k2-docs-loop.sh choose-doc
.agents/skills/k2-docs-loop/k2-docs-loop.sh prepare-draft docs/reference/example.md
```

Artifacts:

- `diataxis-summary.md`: K2-authored markdown summary of the raw Diataxis `.rst` corpus.
- `choose-doc-prompt.md`: stable prompt template for choosing one next doc and requesting needed info.
- `write-doc-prompt.md`: stable prompt template for drafting one doc from a full gathered context file.

Workflow:

1. Run `choose-doc` to show the prompt and ask K2 to choose one new doc and report the exact raw files and exact optional runtime evidence needed.
   The chosen target must be a real content page under a subdirectory like `docs/src/tutorial/...`, `docs/src/how-to/...`, `docs/src/reference/...`, or `docs/src/explanation/...`.
   Never choose `docs/src/SUMMARY.md` as the output document.
2. Gather the exact raw files K2 requested. When K2 asks for overviews and explanations, make those VERY VERBOSE. 
   You tend to output far too little info, so it should feel to you, you're providing too much info. Give it very exhaustively all details.
    Write those extra context intro docs/tmp/verbose_extra_context/[filename].md and include that in the requested files as well.
3. Run `prepare-draft <docs/path.md> <requested files...>` to build a context file in `docs/tmp/` and pipe it into `docs/draft/<docs/path.md>`.
4. Inspect the draft only for factual correctness.
5. If facts are wrong or unsupported, do not correct them, instead write in the draft text your extra comment with // todo: [...]
6. Do not revise structure, tone, scope, or ordering just because you would have written it differently.
7. Then pipe that entire draft into ask-k2 raw together with `.agents/skills/k2-docs-loop/revise_prompt.md` and pipe the output into another draft file
8. Do a final check, on the output. Alter trim the last stuff. PLEASE DO NOT EMPOSE YOUR OPINION AT ALL. Just clean up the artifact output minimally. This is also the moment to write the mermaid diagram, and to make sure it works via docs-lint.
9. Finally, move it to the docs/src part where it belongs.
