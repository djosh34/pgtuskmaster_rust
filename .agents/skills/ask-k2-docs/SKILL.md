---
name: ask-k2-docs
description: Use K2 through the local opencode wrapper for documentation prose only. Use when drafting or revising mdBook page text from agent-supplied facts, outlines, or corrected drafts while keeping Diataxis form and strict no-invention rules.
---

# Ask K2 Docs

Use this skill when the work is prose quality, not truth-finding.

This skill is a helper for `create-docs` and `update-docs`. It does not replace either of them.

This skill is for drafting or revising page text only. The agent remains responsible for:

- rereading the Diataxis references before the run
- deciding whether the page is tutorial, how-to, reference, or explanation
- collecting and checking facts from the repo
- diagrams, Mermaid, and structure
- writing files
- final verification

K2 is good at wording. It knows nothing unless you tell it. Give it the facts, constraints, page goal, mdBook context, and the relevant Diataxis reminders in the prompt. Do not over-constrain its phrasing or force a brittle prompt schema when the writing problem needs room.

Do not make this skill a rigid classifier. The agent should already have reread the Diataxis source pages and decided what kind of page is being worked on. Then give K2 the writing guidance that matters for that page.

Prefer giving K2:

- the page goal
- the audience and user need
- the verified facts and non-facts
- the mdBook page context and any heading structure that should exist
- the writing constraints and style boundaries
- a short Diataxis summary relevant to the current page
- a statement of whether K2 is drafting fresh prose or revising existing prose

Use actual Diataxis wording where it helps, but only the parts relevant to the current page. Typical reminders:

- ask whether the page serves action or cognition, and acquisition or application
- keep the four forms separate instead of mixing them on one page
- how-to guides should contain `action and only action`
- reference should `describe and only describe`
- explanation should provide context, background, reasons, alternatives, and why
- tutorials are lessons with a carefully-managed path
- do not turn the page into a mixture of instruction, reference, and explanation

Language rules for K2 output:

- ASCII punctuation only
- no em dashes
- no invented facts
- no claims about files, commands, or behavior unless supplied in the prompt
- no references to Diataxis as a brag or meta framing in the page text
- no mention that the docs were written "using Diataxis"
- write for mdBook markdown pages

Call the existing `.agents/skills/opencode-openrouter-ask/query-opencode.sh` wrapper directly and put the entire prompt on stdin.

The prompt should be explicit and information-rich. It is good to give K2 many filled sections so the model sees exactly what is known and what is not. Be verbose in the prompt, not careless in the facts. A good pattern is:

```bash
OPENCODE_INPUT=$(cat <<'OPENCODE_EOF'
Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Decide the best wording and structure within the constraints below.
Do not invent facts.
Use ASCII punctuation only.
Do not use em dashes.
Return only markdown for the page body requested.

[Task]
- Draft new prose for a page / revise the supplied prose for clarity and flow.

[Page goal]
- ...

[Audience]
- ...

[User need]
- ...

[mdBook context]
- This will live in `docs/src/...`
- Keep headings and lists suitable for mdBook.
- Do not add verification artifacts or scratch notes.

[Diataxis guidance]
- ...
- ...

[Facts that are true]
- ...
- ...

[Facts that must not be invented or changed]
- ...

[Required structure or sections]
- ...
- ...

[Existing draft to revise]
- ...

[Style constraints]
- ...
- ...
OPENCODE_EOF
)

.agents/skills/opencode-openrouter-ask/query-opencode.sh <<<"$OPENCODE_INPUT"
```

Recommended flow:

1. Agent rereads the Diataxis references for the current run.
2. Agent gathers facts and decides what kind of page it is.
3. Agent asks K2 for a draft or rewrite from facts and constraints.
4. Agent verifies and edits the result for truth, structure, and diagrams.
5. Agent asks K2 for another revision if wording still needs work after agent corrections.
6. Agent performs the final check and writes the file.

Only use this skill for prose. Never use it to outsource verification, repo inspection, or diagrams.
