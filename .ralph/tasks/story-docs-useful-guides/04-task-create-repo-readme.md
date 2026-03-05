---
## Task: Create repository README as the front-door quick-start and project overview <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Add a normal, useful root `README.md` that explains what this project is, how to get started quickly, where to go next for deeper docs, and what the license status is.

**Scope:**
- Create a new root `README.md` for the repository.
- Keep it concise and practical: brief product explanation, quick-start-oriented getting-started section, common repo-entry information, and links into the mdBook docs for deeper operator/contributor material.
- Include a clear `License` section that states `All Rights Reserved Joshua Azimullah`.

**Context from research:**
- The repository currently has no root `README` file.
- Existing docs already contain deeper material for quick start and operator guidance under `docs/src/quick-start/` and other book sections; the README should act as the front door, not duplicate the whole book.
- This belongs at the end of the docs usefulness story because the README should reflect the improved docs direction and point at the cleaned-up docs structure rather than freezing older wording.

**Expected outcome:**
- The repo has a normal top-level README that gives a newcomer enough context to understand the project and reach first-use documentation quickly.
- The README does not pretend to replace the full docs book; it links readers to the right next sections.
- The README clearly states the repository license status as `All Rights Reserved Joshua Azimullah`.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Create `README.md` at the repository root.
- [ ] `README.md` explains at least:
  - what `pgtuskmaster_rust` is
  - the core problem it solves in plain language
  - a quick first path for a new user or contributor
  - where to find the fuller docs/book
  - the most important repo-level commands or entry points worth knowing initially
- [ ] `README.md` is short, readable, and normal for a repo front page rather than a giant manual or architecture report.
- [ ] `README.md` links to the appropriate deeper docs instead of duplicating large sections of `docs/src/`.
- [ ] `README.md` includes a `License` section that states exactly: `All Rights Reserved Joshua Azimullah`.
- [ ] If the final README introduces links or references that depend on docs naming/navigation, update those references to match the current docs structure.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
