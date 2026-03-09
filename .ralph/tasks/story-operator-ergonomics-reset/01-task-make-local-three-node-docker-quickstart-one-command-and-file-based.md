## Task: Make The Local Three-Node Docker Quickstart One Command And File-Based <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Replace the current local-cluster bring-up flow with one obvious, file-based path that a new operator can run from the repo root without copying `.env` files, reading wrapper scripts, or guessing ports. The higher-order goal is to make "I have a healthy 3-node cluster" feel like a normal first-run experience instead of a repo-internals exercise.

**Scope:**
- Define one canonical local cluster entry point at the repo root: `docker compose up -d --build`.
- Remove the requirement for `.env.docker` or `.env.docker.example` in the canonical local path.
- Ship default local file assets in-repo for the canonical path:
- dumb-but-real password secret files
- dumb-but-real API and PostgreSQL TLS cert/key material
- stable runtime config files for node-a, node-b, and node-c
- memorable published ports that are documented as part of the product surface, not an implementation detail.
- Make the local path three-node-first. Do not keep single-node as the quickstart or the fallback beginner flow.
- Ensure a freshly started local stack is immediately operable with `pgtm` from the host using the same shipped configs.

**Context from research:**
- The current README still routes multi-node users through `make docker-up-cluster` or `tools/docker/cluster.sh ... --env-file ...`.
- `docker/compose/docker-compose.cluster.yml` depends on env-file variables for secrets and ports even though the example values are stable.
- The current local cluster uses `18081`, `18082`, `18083` for APIs and `15433`, `15434`, `15435` for PostgreSQL; those values are stable already but not presented as a clean contract.
- The docs/tutorial flow still splits attention between docker runtime configs, docs-owned operator configs, helper scripts, and make targets.
- The user explicitly wants "a simple docker compose up with some default config" and wants env-file-centric local setup removed.

**Expected outcome:**
- A new user can clone the repo, optionally install `pgtm`, run `docker compose up -d --build`, and have a healthy local 3-node cluster without creating or editing an env file first.
- The shipped local cluster includes usable default secrets and full TLS for both API and PostgreSQL so the default experience reflects the real product shape.
- The canonical local ports are easy to remember and documented as fixed defaults. Use this contract unless execution proves a stronger reason to change it:
- APIs: `18081`, `18082`, `18083`
- PostgreSQL: `15431`, `15432`, `15433`
- The follow-up operator command works directly from the repo:

```bash
pgtm -c docker/configs/local/node-a/config.toml status
```

- The expected first healthy operator output direction is explicit and stable enough for docs and tests:

```text
cluster: local-docker  health: healthy
queried via: node-a

NODE    SELF  ROLE     TRUST         PHASE    API
node-a  *     primary  full_quorum   primary  ok
node-b        replica  full_quorum   replica  ok
node-c        replica  full_quorum   replica  ok
```

</description>

<acceptance_criteria>
- [ ] A canonical repo-root local cluster path exists and is documented as `docker compose up -d --build`.
- [ ] The canonical local path does not require `.env.docker`, `.env.docker.example`, or any copied env file.
- [ ] The repo ships default file-based local secrets and pregenerated local TLS materials for both API and PostgreSQL.
- [ ] The local quickstart is three-node-first and no longer teaches single-node as the default beginner path.
- [ ] The local quickstart publishes memorable fixed ports, and the implementation/documentation uses `18081`, `18082`, `18083` for APIs plus `15431`, `15432`, `15433` for PostgreSQL unless a documented product decision replaces them everywhere consistently.
- [ ] `pgtm -c docker/configs/local/node-a/config.toml status` works from the host against the default local stack without requiring a second operator-overlay config file.
- [ ] `pgtm -c docker/configs/local/node-a/config.toml primary` returns a host-usable DSN against the default local stack.
- [ ] The canonical quickstart no longer requires `make` or `tools/docker/*.sh`.
- [ ] All replaced local docs and examples are updated so there is only one obvious first-run path.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
