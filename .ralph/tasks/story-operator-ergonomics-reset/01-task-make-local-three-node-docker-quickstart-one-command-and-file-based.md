## Task: Make The Local Three-Node Docker Quickstart One Command And File-Based <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Replace the current local-cluster bring-up flow with one obvious, file-based path that a new operator can run from the repo root without copying `.env` files, reading wrapper scripts, or guessing ports. The higher-order goal is to make "I have a healthy 3-node cluster" feel like a normal first-run experience instead of a repo-internals exercise.

**Scope:**
- Define one canonical local cluster entry point at the repo root: `docker compose -f docker/compose.yml up -d --build`.
- Remove all other shipped operator-facing Docker Compose variants for local onboarding. Keep exactly one canonical compose file for the product quickstart: `docker/compose.yml`.
- Remove the requirement for `.env.docker` or `.env.docker.example` in the canonical local path.
- Flatten the shipped runnable asset layout under a shallow repo-owned `docker/` directory. Do not keep the canonical path buried under `docker/configs/cluster/node-a/runtime.toml`-style nesting.
- Ship default local file assets in-repo for the canonical path with this exact target layout unless implementation discovers a hard blocker and updates every touched doc/task consistently:

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

- dumb-but-real password secret files
- dumb-but-real API and PostgreSQL TLS cert/key material
- stable runtime config files for node-a, node-b, and node-c
- one minimal operator CLI config file for host-side `pgtm`
- memorable published ports that are documented as part of the product surface, not an implementation detail.
- secure default PostgreSQL access controls (`pg_hba` and `pg_ident`) with no `trust` auth in the canonical stack
- secure default API and PostgreSQL auth/TLS settings that prove the intended real-world product shape instead of a wide-open demo mode
- Make the local path three-node-first. Do not keep single-node as the quickstart or the fallback beginner flow.
- Ensure a freshly started local stack is immediately operable with `pgtm` from the host using the same shipped configs.

**Context from research:**
- The current README still routes multi-node users through `make docker-up-cluster` or `tools/docker/cluster.sh ... --env-file ...`.
- `docker/compose/docker-compose.cluster.yml` depends on env-file variables for secrets and ports even though the example values are stable.
- The current local cluster uses `18081`, `18082`, `18083` for APIs and `15433`, `15434`, `15435` for PostgreSQL; those values are stable already but not presented as a clean contract.
- The docs/tutorial flow still splits attention between docker runtime configs, docs-owned operator configs, helper scripts, and make targets.
- The user explicitly wants "a simple docker compose up with some default config" and wants env-file-centric local setup removed.

**Expected outcome:**
- A new user can clone the repo, optionally install `pgtm`, run `docker compose -f docker/compose.yml up -d --build`, and have a healthy local 3-node cluster without creating or editing an env file first.
- The shipped local cluster includes usable default secrets and full TLS for both API and PostgreSQL so the default experience reflects the real product shape.
- The shipped local cluster uses secure defaults by default:
- no PostgreSQL `trust` auth in the canonical shipped stack
- strict shipped `pg_hba.conf` and `pg_ident.conf`
- API auth enabled
- API TLS enabled
- PostgreSQL TLS enabled
- certificate validation plus credentials required where the protocol supports both cleanly
- For the canonical local Docker stack, prefer the strictest PostgreSQL verification mode that remains ergonomic without hostname/SAN friction. The current target is `verify-ca` for local Docker unless implementation proves `verify-full` can be made equally smooth. Real-VM docs should still prefer `verify-full`.
- The canonical local ports are easy to remember and documented as fixed defaults. Use this contract unless execution proves a stronger reason to change it:
- APIs: `18081`, `18082`, `18083`
- PostgreSQL: `15001`, `15002`, `15003`
- The follow-up operator command works directly from the repo without reading any node runtime internals:

```bash
pgtm -c docker/pgtm.toml status
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
- [ ] A canonical repo-root local cluster path exists and is documented as `docker compose -f docker/compose.yml up -d --build`.
- [ ] `docker/compose.yml` is the only shipped operator-facing Docker Compose file used by the canonical local quickstart.
- [ ] The canonical local path does not require `.env.docker`, `.env.docker.example`, or any copied env file.
- [ ] The canonical local deployment assets are flattened under a shallow `docker/` directory rather than a deep nested config tree.
- [ ] The canonical local file layout is exactly or materially equivalent to:
- `docker/compose.yml`
- `docker/node-a.toml`
- `docker/node-b.toml`
- `docker/node-c.toml`
- `docker/pgtm.toml`
- `docker/secrets/`
- `docker/tls/`
- [ ] The repo ships default file-based local secrets and pregenerated local TLS materials for both API and PostgreSQL.
- [ ] The canonical shipped local stack does not use PostgreSQL `trust` auth anywhere in its default runtime path.
- [ ] The canonical shipped local stack includes strict default `pg_hba.conf` and `pg_ident.conf` files, and those files are documented as intentional secure defaults rather than placeholders.
- [ ] The canonical shipped local stack enables API auth and API TLS by default.
- [ ] The canonical shipped local stack enables PostgreSQL TLS by default.
- [ ] The canonical shipped local stack requires certificate validation plus credentials where the protocol supports both cleanly.
- [ ] The canonical local PostgreSQL TLS verification mode is explicitly chosen and justified in docs; the target is `verify-ca` for local Docker unless `verify-full` can be made equally smooth.
- [ ] The local quickstart is three-node-first and no longer teaches single-node as the default beginner path.
- [ ] The local quickstart publishes memorable fixed ports, and the implementation/documentation uses `18081`, `18082`, `18083` for APIs plus `15001`, `15002`, `15003` for PostgreSQL unless a documented product decision replaces them everywhere consistently.
- [ ] `pgtm -c docker/pgtm.toml status` works from the host against the default local stack.
- [ ] `pgtm -c docker/pgtm.toml primary` returns a host-usable DSN against the default local stack.
- [ ] The node runtime configs remain directly usable by the daemon and may also be usable by `pgtm`, but the canonical host-side CLI example uses the smaller `docker/pgtm.toml` config.
- [ ] The canonical quickstart no longer requires `make` or `tools/docker/*.sh`.
- [ ] The task includes an explicit manual usability verification pass that follows the quickstart from a clean checkout and records whether the flow worked directly without hiccups, hidden prerequisites, or doc gaps.
- [ ] The task adds or updates automated smoke/integration coverage for the canonical one-compose secure local stack.
- [ ] All replaced local docs and examples are updated so there is only one obvious first-run path.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
