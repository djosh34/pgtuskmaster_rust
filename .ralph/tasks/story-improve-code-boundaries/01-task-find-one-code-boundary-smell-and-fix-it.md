## Task: Find One Code Boundary Smell And Fix It <status>not_started</status> <passes>meta-task</passes>
NEVER TICK OFF THIS TASK. ALWAYS KEEP <passes>meta-task</passes>. This is a recurring deep verification task.

<description>
This is a **RECURRING META-TASK**. Every time this task is picked up, the engineer must do a **FRESH verification** pass. **NEVER set this task's passes to anything other than meta-task**.

**Goal:** Use the `improve-code-boundaries` skill at `.agents/skills/improve-code-boundaries/SKILL.md` to find one smell in `src/` or tests, then fix that smell completely. Pick exactly one smell per run. Follow the skill workflow: identify the boundary problem, flatten it, remove dead transitions/wrappers/overengineering where applicable, and verify the result.

**Scope:**
- Search `src/` and tests for one strong smell covered by the `improve-code-boundaries` skill.
- Fix that one smell fully, not partially.
- Prefer code reduction, flatter boundaries, and fewer duplicate shapes/helpers.

**Expected outcome:**
- One concrete smell is removed.
- The touched area is simpler, flatter, and more direct than before.
- The repo remains green after verification.

## Exploration

### Run log
- Date:
- Smell chosen:
- Files touched:
- What was removed or flattened:
- What verification proved it:

</description>

<acceptance_criteria>
- [ ] A FRESH pass is done using `.agents/skills/improve-code-boundaries/SKILL.md`.
- [ ] Exactly one smell in `src/` or tests is identified and fixed end to end.
- [ ] The fix removes real complexity or duplication rather than only moving code around.
- [ ] `make check` — passes cleanly
- [ ] If the changed area needs focused runtime/test verification: use `cargo nextest ...` appropriately
- [ ] If the changed area is mainly covered by long-running integration behavior: `make test-long` — passes cleanly
- THIS TASK STAYS AS meta-task FOREVER
</acceptance_criteria>
