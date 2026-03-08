## Task: Rename The Operator CLI To `pgtm` And Flatten The Command Tree <status>not_started</status> <passes>false</passes>

<priority>medium</priority>

<description>
**Goal:** Replace the long operator binary name with `pgtm` and redesign the command tree around minimal, direct operator verbs instead of nested namespaces such as `ha`. The higher-order goal is to reduce cognitive load so the operator interface feels obvious at a glance rather than requiring users to learn internal subsystem labels before they can do routine work.

**Scope:**
- Introduce the short operator binary name `pgtm`.
- Design a flat or mostly flat command tree for common actions such as `status`, `switchover`, `primary`, and debug/reporting commands.
- Remove or de-emphasize redundant prefixes such as `ha` for routine user-facing commands.
- Update binary packaging, docs, examples, tests, and any developer tooling references to the operator binary name and command surface.
- Decide whether the old long binary name remains as a temporary alias or is removed outright; because this project explicitly allows breaking changes, prefer the cleaner end state unless there is a strong repository-local reason to keep both.

**Context from research:**
- The current operator CLI is `pgtuskmasterctl` and its only public commands sit under `ha`, which makes the UX feel more like a protocol test tool than an operator product.
- The user explicitly wants fewer keywords and does not want `ha` prepended everywhere.
- The repository is greenfield and explicitly does not require backwards compatibility, so this is a good time to make a naming correction instead of carrying an awkward binary and command hierarchy forward.

**Expected outcome:**
- Operators use a short memorable binary name.
- Common commands are direct and self-explanatory.
- The docs and product identity become more coherent around a cluster-first operator UX rather than a thin API wrapper.

</description>

<acceptance_criteria>
- [ ] The operator binary is renamed to `pgtm`, or `pgtm` is introduced as the primary supported binary name with a deliberate decision recorded about the old name.
- [ ] The routine command tree is flattened so common operations do not require the `ha` prefix.
- [ ] Existing supported actions are remapped into the new command tree with docs and tests updated accordingly.
- [ ] The CLI reference, how-to guides, examples, and test coverage all use the new `pgtm` surface consistently.
- [ ] Any retained alias or migration shim is intentional, documented, and does not keep the old command tree as the primary UX.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
