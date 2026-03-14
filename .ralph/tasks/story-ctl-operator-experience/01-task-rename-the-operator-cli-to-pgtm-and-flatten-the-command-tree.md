## Task: Rename The Operator CLI To `pgtm` And Flatten The Command Tree <status>completed</status> <passes>true</passes>

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
- [x] The operator binary is renamed to `pgtm`, or `pgtm` is introduced as the primary supported binary name with a deliberate decision recorded about the old name.
- [x] The routine command tree is flattened so common operations do not require the `ha` prefix.
- [x] Existing supported actions are remapped into the new command tree with docs and tests updated accordingly.
- [x] The CLI reference, how-to guides, examples, and test coverage all use the new `pgtm` surface consistently.
- [x] Any retained alias or migration shim is intentional, documented, and does not keep the old command tree as the primary UX.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Execution Plan

### Product decision
- Make `pgtm` the only operator CLI binary for this repo state. Do not keep `pgtuskmasterctl` as a compatibility alias unless execution discovers a hard repository-local blocker, because the project explicitly prefers the cleaner break over carrying legacy names.
- Flatten the currently supported operator verbs at the root of the CLI instead of introducing speculative new features. In this task the concrete remap should be:
  - `pgtuskmasterctl ha state` -> `pgtm status`
  - `pgtuskmasterctl ha switchover request [--switchover-to ...]` -> `pgtm switchover request [--switchover-to ...]`
  - `pgtuskmasterctl ha switchover clear` -> `pgtm switchover clear`
- Do not invent `primary` or debug/reporting commands in this task, because the current CLI implementation only exposes HA state plus switchover operations and later tasks in this story cover richer operator workflows. This task should establish the flatter root command shape those future commands will join.

### Code changes
- Rename the binary entrypoint from `src/bin/pgtuskmasterctl.rs` to `src/bin/pgtm.rs` so Cargo builds `pgtm` directly.
- Update `src/cli/args.rs`:
  - change the clap command name from `pgtuskmasterctl` to `pgtm`,
  - replace the top-level `Command::Ha(HaArgs)` structure with a flatter top-level command enum,
  - rename the operator read command from `state` to `status`,
  - preserve the existing `switchover request` and `switchover clear` subcommands and their flags,
  - update parser tests so every argv example uses `pgtm` and the flattened command tree.
- Update `src/cli/mod.rs` to match the new enum layout while preserving the existing API client behavior:
  - `status` should continue to call `client.get_ha_state()`,
  - `switchover request` should continue to call `client.post_switchover(...)`,
  - `switchover clear` should continue to call `client.delete_switchover()`.
- Keep output rendering and exit-code behavior unchanged unless command renaming requires small wording fixes in help text or tests.

### Test and harness changes
- Update `tests/cli_binary.rs` to resolve `CARGO_BIN_EXE_pgtm` / `target/debug/pgtm` instead of the old binary name.
- Rewrite the CLI integration assertions around the flattened surface:
  - help output should advertise `status` and `switchover` instead of `ha`,
  - connection-refused coverage should invoke `pgtm status`,
  - usage-error coverage should target an invalid root-level command path instead of `ha ...`.
- Update any in-process CLI parsing or harness command construction that still hardcodes the old argv layout, especially `tests/ha/support/multi_node.rs`, so switchover requests are exercised through `pgtm switchover request`. Treat these call sites as first-class surface coverage, not incidental helpers, because they use `Cli::try_parse_from(...)` directly and will otherwise silently lag behind the public CLI.
- Keep the existing env-var names (`PGTUSKMASTER_READ_TOKEN` and `PGTUSKMASTER_ADMIN_TOKEN`) unless execution finds an explicit product decision to rename them too. They are not part of the requested operator UX simplification, and changing them would add a separate compatibility/breaking-change axis without improving the command tree itself.
- After the edits, run a targeted search across live source trees such as `src`, `tests`, `docs/src`, `tools`, `docker`, `Cargo.toml`, and `Makefile` for `pgtuskmasterctl`, `CARGO_BIN_EXE_pgtuskmasterctl`, `target/debug/pgtuskmasterctl`, and the old `ha` CLI paths. Explicitly ignore generated or historical trees like `.ralph` and `docs/book` so execution distinguishes real remaining work from archived noise.

### Documentation changes
- Rename the CLI reference page from `docs/src/reference/pgtuskmasterctl-cli.md` to a `pgtm`-named page and update every mdBook link that points at it, including `docs/src/SUMMARY.md`, overview pages, and any chapter landing pages.
- Rewrite operator-facing docs and examples to use the new command surface consistently:
  - tutorials that currently use `cargo run --bin pgtuskmasterctl -- ... ha state`,
  - how-to guides that currently use `pgtuskmasterctl ... ha state` or `ha switchover ...`,
  - overview/explanation pages that describe the CLI by its old name.
- Update command trees, synopsis blocks, and prose so they describe `status` as the direct observation verb and `switchover` as the direct action namespace.
- Do not hand-edit `docs/book`; it is generated output. Update `docs/src/...` and then rebuild the book so the renamed page and links are regenerated from source.
- Because the requested `update-docs` skill is not present in the current session, do the doc updates directly in the repository during execution and remove any stale wording instead of leaving follow-up debt.

### Verification and completion
- Run `make docs-build` after the docs rename/update so mdBook catches any broken links or renamed-page mistakes before the heavier Rust gates.
- Run `make check`.
- Run `make test`.
- Run `make test-long`.
- Run `make lint`.
- Confirm the docs build inputs are consistent with the renamed CLI reference page and command examples, and verify the targeted post-edit search is clean for live code/docs/tooling trees.
- If all gates pass, mark the acceptance boxes and `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit all tracked changes with the required `task finished ...` message format, push, and stop immediately.

NOW EXECUTE
