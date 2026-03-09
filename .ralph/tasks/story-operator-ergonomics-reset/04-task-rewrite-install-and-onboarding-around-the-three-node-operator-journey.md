## Task: Rewrite Install And Onboarding Around The Three-Node Operator Journey <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Redesign the first-time documentation and examples around the actual operator journey: install `pgtm`, bring up a 3-node cluster, read status, inspect debug only when needed, and connect to the primary without scraping internals. The higher-order goal is to make the first ten minutes with the repo coherent and self-explanatory.

**Scope:**
- Rewrite the README quickstart and the leading docs/tutorial pages around a single canonical first-time flow.
- Make `cargo install` for `pgtm` explicit, simple, and correct for the current package layout.
- Document the difference between `pgtm status` and `pgtm debug verbose` as a product decision, not a side note.
- Add one explicit local-Docker journey and one explicit 3-real-VM Docker journey.
- Ensure both journeys use one config file per node and file-based secrets/TLS.
- Remove stale docs that tell the reader to reverse-engineer helper scripts or maintain separate operator-only overlay configs for the canonical flows.

**Context from research:**
- The current docs still split new users across README, the local cluster helper script, docs-owned example configs, single-node tutorial material, and scattered explanations of `pgtm.api_url`.
- The repo exposes both `src/bin/pgtm.rs` and `src/bin/pgtuskmaster.rs`, so `cargo install --path . --bin pgtm` is already technically possible and should be part of the canonical operator install story.
- The user explicitly wants to discuss the first-time experience and wants the status/debug distinction explained more clearly.
- The user also wants the 3-real-VM Docker path to feel feasible with Docker on each VM plus one config file.

**Expected outcome:**
- The first-time local journey is short and boring:

```bash
cargo install --path . --bin pgtm
docker compose up -d --build
pgtm -c docker/configs/local/node-a/config.toml status
psql "$(pgtm -c docker/configs/local/node-a/config.toml primary)"
```

- The first-time 3-real-VM journey is equally explicit:

```bash
# on each VM
docker compose up -d

# from any VM with its local node config
pgtm -c /etc/pgtuskmaster/config.toml status
pgtm -c /etc/pgtuskmaster/config.toml primary
```

- The docs explain the command split with exact wording direction:
- `pgtm status` answers "what is the cluster doing right now?"
- `pgtm debug verbose` answers "why does this one node think that?"
- New users do not need to understand internal helper scripts, env files, or raw HTTP before they get value.

</description>

<acceptance_criteria>
- [ ] The README quickstart is rewritten around the canonical 3-node local path, not around `make`, env files, or helper scripts.
- [ ] The README or primary install doc includes a correct `cargo install --path . --bin pgtm` path for the operator CLI.
- [ ] The docs contain one explicit "3 real VMs, Docker on each VM, one config file per VM" walkthrough with concrete file paths and commands.
- [ ] The docs explain `status` versus `debug verbose` clearly and consistently across README, tutorials, and CLI reference material.
- [ ] The main onboarding path teaches `pgtm` and `psql "$(pgtm ... primary)"` instead of curl loops or manual DSN assembly.
- [ ] Local Docker examples use the same canonical config files as the shipped deployment assets rather than separate docs-only operator overlays.
- [ ] Stale beginner-facing references to env-file copying, docker helper scripts, or non-canonical cluster bring-up commands are removed.
- [ ] The final docs make the first ten minutes with the repo possible without reading internal shell scripts.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
