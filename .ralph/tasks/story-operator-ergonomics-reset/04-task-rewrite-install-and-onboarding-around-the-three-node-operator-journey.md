## Task: Rewrite Install And Onboarding Around The Three-Node Operator Journey <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Redesign the first-time documentation and examples around the actual operator journey: install `pgtm`, bring up a 3-node cluster, read status, inspect debug only when needed, and connect to the primary without scraping internals. The higher-order goal is to make the first ten minutes with the repo coherent and self-explanatory.

**Scope:**
- Treat tasks 01 and 02 as the product-surface prerequisites for this documentation rewrite. This task should explain the chosen shape crisply, not invent a third competing shape inside the docs.
- Rewrite the README quickstart and the leading docs/tutorial pages around a single canonical first-time flow.
- Make `cargo install` for `pgtm` explicit, simple, and correct for the current package layout.
- Add a dedicated quickstart guide that uses only the canonical one-compose local flow and gets the operator from zero to a healthy cluster with no detours.
- Document the difference between `pgtm status` and `pgtm debug verbose` as a product decision, not a side note.
- Add one explicit local-Docker journey and one explicit 3-real-VM Docker journey.
- Ensure both journeys use one config file per node and file-based secrets/TLS.
- Explain path conventions explicitly:
- shallow `docker/` paths are for repo-shipped runnable assets and local onboarding
- `/etc/pgtuskmaster/config.toml` is for installed Linux VM/service layouts only
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
docker compose -f docker/compose.yml up -d --build
pgtm -c docker/pgtm.toml status
psql "$(pgtm -c docker/pgtm.toml primary)"
docker compose -f docker/compose.yml down
docker compose -f docker/compose.yml down -v
```

- The quickstart guide is explicitly based on the same commands and does not branch into alternate compose files, helper scripts, or env-file preparation.

- The first-time 3-real-VM journey is equally explicit:

```bash
# on each VM
docker compose -f /opt/pgtuskmaster/docker/compose.yml up -d

# from any VM with its local node config
pgtm -c /etc/pgtuskmaster/config.toml status
pgtm -c /etc/pgtuskmaster/config.toml primary
```

- The docs explain the command split with exact wording direction:
- `pgtm status` answers "what is the cluster doing right now?"
- `pgtm debug verbose` answers "why does this one node think that?"
- The docs explain the config split with exact wording direction:
- daemon runtime config: full file because the daemon needs cluster, postgres, DCS, process, logging, API, and debug settings
- operator config: small file because `pgtm` only needs API target, auth, and TLS/client connection context
- The docs explicitly say that the canonical local Docker assets were derived from the HA test givens and then simplified for product use, so operators are not told to study test harness internals to get started.
- New users do not need to understand internal helper scripts, env files, or raw HTTP before they get value.

</description>

<acceptance_criteria>
- [ ] The README quickstart is rewritten around the canonical 3-node local path, not around `make`, env files, or helper scripts.
- [ ] The README or primary install doc includes a correct `cargo install --path . --bin pgtm` path for the operator CLI.
- [ ] The README or primary install doc shows the canonical local commands with `docker/compose.yml` and `docker/pgtm.toml`.
- [ ] A dedicated quickstart guide exists and uses only the canonical one-compose local flow.
- [ ] The quickstart guide documents both `docker compose -f docker/compose.yml down` and `docker compose -f docker/compose.yml down -v`, including that `down -v` resets local cluster state.
- [ ] The docs contain one explicit "3 real VMs, Docker on each VM, one config file per VM" walkthrough with concrete file paths and commands.
- [ ] The docs explain `status` versus `debug verbose` clearly and consistently across README, tutorials, and CLI reference material.
- [ ] The docs explain the daemon-config versus minimal-operator-config split clearly and consistently across README, tutorials, and configuration reference material.
- [ ] The docs explain why the canonical local path now resembles the HA givens structurally while remaining a product path rather than a harness path.
- [ ] The docs explain that `/etc/pgtuskmaster/config.toml` is an installation convention for deployed VMs, while the repo itself uses shallow `docker/` paths for shipped runnable assets.
- [ ] The docs explain the shipped secure-default local posture explicitly: strict `pg_hba`/`pg_ident`, no `trust`, API TLS/auth on, PostgreSQL TLS on, and the chosen local verification mode rationale.
- [ ] The main onboarding path teaches `pgtm` and `psql "$(pgtm ... primary)"` instead of curl loops or manual DSN assembly.
- [ ] Local Docker examples use the same canonical config files as the shipped deployment assets rather than separate docs-only operator overlays.
- [ ] Stale beginner-facing references to env-file copying, docker helper scripts, or non-canonical cluster bring-up commands are removed.
- [ ] Stale beginner-facing references to `docs/examples/docker-*.toml` as the canonical local operator path are removed.
- [ ] The final docs make the first ten minutes with the repo possible without reading internal shell scripts.
- [ ] The documentation work includes a judgment pass on ease of use from a clean checkout, and any hiccups found during that pass are fixed rather than merely noted.
- [ ] The documentation work verifies that the teardown commands in the quickstart behave exactly as documented.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
