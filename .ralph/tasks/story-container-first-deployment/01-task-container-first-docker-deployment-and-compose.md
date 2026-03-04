---
## Task: Container-first deployment baseline with Docker images, Compose stacks, and secrets <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Make container deployment the default operational path by adding production/development images and turnkey Docker Compose stacks that run etcd3 + pgtuskmaster with config maps and Docker secrets.

**Scope:**
- Add a production container image for `pgtuskmaster` nodes with PostgreSQL server/client binaries included (`postgres`, `pg_ctl`, `initdb`, `pg_basebackup`, `pg_rewind`, `psql`) and `pgtuskmaster` as entrypoint.
- Add a development container image variant that includes development tooling while keeping production image minimal (no Node/mdBook/runtime-unneeded tooling in prod).
- Add compose topology for single-node bring-up (`etcd3 + 1 pgtuskmaster node`) and full multi-node cluster bring-up (`etcd3 + N pgtuskmaster nodes`, minimum 3 nodes).
- Use Docker Compose `configs` for node configs and managed static config payloads; use Docker Compose `secrets` for credential material and private keys.
- Add an example `.env` file used to parameterize secret-file locations that Compose `secrets.file` references, without injecting database secrets directly as plain env vars.
- Constrain network exposure so only required inter-node/service ports are reachable internally, while exposing node API, debug API, and PostgreSQL client ports externally.
- Update operator-facing docs so container deployment is the expected/default path, and non-container/manual deployment is secondary.

**Context from research:**
- The repository currently has no `Dockerfile`/Compose assets; Docker support is greenfield.
- Runtime/process wiring requires real PostgreSQL binaries configured under `process.binaries.*`; container images must provide all required binaries for init/bootstrap/rewind/start/stop flows.
- Existing docs include deployment content but are not structured as container-first workflows.

**Expected outcome:**
- A new operator can run a documented `docker compose up` flow and get a working cluster (etcd3 + pgtuskmaster nodes) without ad-hoc local binary setup.
- Production and development images are clearly separated with minimal production footprint.
- Compose assets use `configs` + `secrets` correctly, and secret handling is file-based via Docker secrets.
- Documentation prioritizes containers as the primary deployment path.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Add production image definition at `Dockerfile` (or `docker/Dockerfile.prod`) with:
- [ ] Multi-stage Rust build of `pgtuskmaster` binary
- [ ] PostgreSQL runtime binaries installed and discoverable at configured paths
- [ ] `pgtuskmaster` container entrypoint defaults to node runtime launch
- [ ] No Node.js/mdBook/dev-only toolchain in production image
- [ ] Add development image definition at `Dockerfile.dev` (or `docker/Dockerfile.dev`) with:
- [ ] Rust/cargo toolchain and developer utilities required for local iteration
- [ ] Optional docs/dev tooling allowed only in this image
- [ ] Add container startup wrapper at `docker/entrypoint.sh` (or equivalent) with robust error handling and clear required env/config checks
- [ ] Update `.dockerignore` to keep build context minimal and deterministic
- [ ] Add single-node compose stack at `docker-compose.yml` (or `docker/compose/docker-compose.single.yml`) containing:
- [ ] `etcd3` service
- [ ] one `pgtuskmaster` node service
- [ ] Compose `configs` for node config TOML and static config artifacts
- [ ] Compose `secrets` for PostgreSQL role passwords and any TLS private material
- [ ] External published ports for API/debug/PG only
- [ ] Add multi-node compose stack at `docker/compose/docker-compose.cluster.yml` (or equivalent) containing:
- [ ] `etcd3` service (or explicit etcd quorum if required)
- [ ] at least three `pgtuskmaster` node services
- [ ] per-node Compose `configs` and per-node `secrets` wiring
- [ ] inter-node-only network paths for internal communication, with least-port exposure
- [ ] Add Compose config assets under `docker/configs/` (or equivalent) for each node:
- [ ] node runtime TOML files
- [ ] managed `pg_hba`/`pg_ident` sources when applicable
- [ ] Add Docker secret source files under `docker/secrets/` with committed `*.example` placeholders only; real secret values must remain git-ignored
- [ ] Add `.env.docker.example` (or `.env.example`) documenting required variables for secret file paths and published ports; no plaintext secret values committed
- [ ] Add a short reproducible bring-up/teardown helper flow (`make docker-up`, `make docker-down`, or scripted equivalent) for both single-node and multi-node stacks
- [ ] Add smoke/integration verification for Compose assets:
- [ ] `docker compose ... config` succeeds for all provided stacks
- [ ] single-node stack reaches healthy API/debug/PG endpoints
- [ ] multi-node stack reaches healthy API/debug/PG endpoints on each node and etcd connectivity is confirmed
- [ ] no service relies on non-secret env vars for credential payloads that should be Docker secrets
- [ ] Update docs to make containers the primary deployment path:
- [ ] `docs/src/operator/deployment.md` rewritten to start with container-first deployment flow
- [ ] `docs/src/quick-start/prerequisites.md` and `docs/src/quick-start/first-run.md` updated to include container-first quick start
- [ ] add/update dedicated container deployment doc page (`docs/src/operator/container-deployment.md` or equivalent) and wire it in `docs/src/SUMMARY.md`
- [ ] non-container/manual deployment content explicitly marked as advanced/secondary path
- [ ] `make docs-lint` passes cleanly
- [ ] `make docs-build` passes cleanly
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
