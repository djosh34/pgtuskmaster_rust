You are choosing exactly one next documentation file to write.

Rules:
- Follow Diataxis strictly.
- Use only the supplied repo facts and supplied Diataxis summary.
- If a fact is missing, say "missing source support" instead of inventing.
- ASCII only.
- No em dashes.
- Prefer diagrams only when the supplied facts support every node and edge.

You are not being asked to write the document yet.
You are being asked to choose the single best next document and to ask for the exact raw evidence needed so that you can later write it well without invention.

You will receive a large raw context pack. Treat it as authoritative even if it is incomplete.
Ask for exact files, exact directories, exact tests, exact commands, exact sample configs, and exact generated outputs when needed.

Output format:

Target docs path: <path>
Diataxis type: <tutorial|how-to|reference|explanation>
Why this is the next doc:
- ...
- ...

Exact additional information needed:
- file: <repo path>
  why: <why this exact file is needed>
- file: <repo path>
  why: <why this exact file is needed>
- extra info: <extra question on project>

Optional runtime evidence to generate:
- command: <exact command>
  why: <what evidence this would provide>

Hard constraints on your answer:
- Be specific.
- The target docs path must be a content page under `docs/src/<section>/...` and must not be `docs/src/SUMMARY.md`.
- Name exact file paths whenever possible.
- Do not write the page.
- Do not propose multiple competing next pages.
