## Task: Delete Unnecessary Docker Shell Wrappers And Shrink The Makefile To Gates <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Remove shell and make indirection that obscures the operator workflow instead of helping it. The higher-order goal is to make the repo's public surface look intentional: operator commands should be `docker compose`, `pgtm`, and explicit install commands, while Makefile and shell scripts retreat to narrowly justified engineering support roles.

**Scope:**
- Audit all current operator-facing shell entrypoints under `tools/docker/` and all operator-facing Make targets.
- Delete shell wrappers that exist only to smuggle env files, hidden defaults, or status printing around otherwise normal `docker compose` usage.
- Delete path-depth churn that exists only because scripts grew around the current layout. The public runnable asset tree should be shallow enough that commands naturally look like `docker compose -f docker/compose.yml ...` and `pgtm -c docker/pgtm.toml ...`.
- Keep a shell script only when it has a narrow, defensible purpose that cannot be expressed cleanly through compose files, Rust code, or simple documented commands.
- Audit installer scripts under `tools/install-*.sh`.
- Delete installer scripts that no longer provide meaningful value after the onboarding reset.
- For any installer script kept, document why it remains, who it is for, and why `cargo install`, package-manager install, or a direct download is not enough.
- Reduce the Makefile so it primarily owns quality gates and a very small number of clearly justified developer aliases.

**Context from research:**
- The current public local cluster path still leans on `tools/docker/cluster.sh`, `tools/docker/common.sh`, `make docker-up-cluster`, and env-file defaults.
- The current Makefile mixes strict gate orchestration with operator lifecycle aliases, which makes "how do I use this product?" harder to answer.
- The user explicitly called out the repo as having "a weird amount of bash files" and an "overengineered makefile".
- Existing docker helper logic already has at least one tracked bug for ignored-error behavior, which is another reason to reduce shell surface rather than grow it.

**Expected outcome:**
- A new user does not need `make` or `tools/docker/*.sh` to start or inspect the canonical local cluster.
- The README's public command vocabulary is small and obvious:
- `cargo install ...`
- `docker compose ...`
- `pgtm ...`
- The public repo paths are small and obvious too:
- `docker/compose.yml`
- `docker/node-a.toml`
- `docker/node-b.toml`
- `docker/node-c.toml`
- `docker/pgtm.toml`
- Remaining shell scripts are either test harness/support tooling or narrowly justified installers, not the normal operator path.
- The Makefile reads like a quality-gates entry point, not a second product CLI.

</description>

<acceptance_criteria>
- [ ] Every operator-facing shell wrapper under `tools/docker/` is either deleted or justified in-code and in docs as non-canonical support tooling.
- [ ] The canonical README and tutorial flows do not require `tools/docker/*.sh`.
- [ ] The canonical README and tutorial flows do not require `make` for operator workflows.
- [ ] The canonical README and tutorial flows use shallow public paths under `docker/` rather than deep nested runtime-config directories.
- [ ] The Makefile is reduced to quality gates plus a small, documented alias surface; it no longer acts as the main operator UX.
- [ ] Installer scripts are audited one by one, with each script either deleted or explicitly justified in docs/comments.
- [ ] The repo no longer requires env-file plumbing shell wrappers to achieve the canonical local Docker experience.
- [ ] Any script kept has explicit error handling and does not rely on swallowed failures or hidden fallbacks.
- [ ] The resulting shell/Make surface is documented so contributors know which commands are public UX versus internal support tooling.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
