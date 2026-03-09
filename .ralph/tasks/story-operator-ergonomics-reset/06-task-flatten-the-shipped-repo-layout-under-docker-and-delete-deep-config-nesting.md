## Task: Flatten The Shipped Repo Layout Under `docker/` And Delete Deep Config Nesting <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Make the repo layout match the intended operator experience by flattening the shipped local deployment assets into one obvious `docker/` directory. The higher-order goal is to stop forcing users to navigate `docker/configs/cluster/node-a/runtime.toml`-style paths for the canonical quickstart when the product surface can be much shallower and clearer.

**Scope:**
- Treat the current deep repo layout as accidental complexity, not a requirement.
- Move the canonical local runnable assets under a shallow `docker/` directory.
- Use `docker/`, not `examples/`, for the canonical runnable assets because these files are meant to be executed directly, not merely read as illustrations.
- Keep `examples/` only for clearly non-canonical snippets or alternate reference material if any still remains after the reset.
- Converge all canonical local compose usage on one file only: `docker/compose.yml`.
- Delete or redirect old deep paths so docs, scripts, and tests converge on the shallow layout.

**Context from research:**
- The current repo spreads the local operator path across `docker/compose/`, `docker/configs/cluster/`, docs-owned example configs, and helper scripts.
- The user explicitly asked why the config needs to live "so incredibly deep" and proposed a much shallower layout under either `examples/` or `docker/`.
- Nothing in the product model requires the canonical local assets to stay deeply nested; the depth is coming from repository history and script layering.

**Expected outcome:**
- The canonical local layout is:

```text
docker/
  compose.yml
  node-a.toml
  node-b.toml
  node-c.toml
  pgtm.toml
  secrets/
  tls/
```

- The repo no longer teaches or prefers deeper canonical paths for local onboarding.
- Any remaining deeper directories are either deleted or clearly labeled as internal/test-only support assets.

</description>

<acceptance_criteria>
- [ ] The canonical local deployment assets live under a shallow `docker/` directory with filenames short enough to type and remember.
- [ ] `docker/` is the canonical home for runnable shipped assets; `examples/` is not used as the primary quickstart location.
- [ ] `docker/compose.yml` is the sole canonical operator-facing local compose file.
- [ ] Old deep local-config paths are deleted, redirected, or demoted out of the canonical onboarding flow.
- [ ] README, tutorials, tests, and any surviving scripts all use the shallow `docker/` paths for the canonical local cluster.
- [ ] The resulting local commands look like `docker compose -f docker/compose.yml ...` and `pgtm -c docker/pgtm.toml ...`, not deep nested path invocations.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
