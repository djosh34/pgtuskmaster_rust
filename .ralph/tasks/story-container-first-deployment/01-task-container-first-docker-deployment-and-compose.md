## Task: Container-first deployment baseline with Docker images, Compose stacks, and secrets <status>done</status> <passes>true</passes>

<description>
**Goal:** Make container deployment the default operational path by adding production/development images and turnkey Docker Compose stacks that run etcd3 + pgtuskmaster with config maps and Docker secrets.

**Scope:**
- Add a production container image for `pgtuskmaster` nodes with PostgreSQL server/client binaries included (`postgres`, `pg_ctl`, `initdb`, `pg_basebackup`, `pg_rewind`, `psql`) and `pgtuskmaster` as entrypoint.
- Add a development container image variant that includes development tooling while keeping production image minimal (no Node/mdBook/runtime-unneeded tooling in prod).
- Add compose topology for single-node bring-up (`etcd3 + 1 pgtuskmaster node`) and full multi-node cluster bring-up (`etcd3 + N pgtuskmaster nodes`, minimum 3 nodes).
- Use Docker Compose `configs` for node configs and managed static config payloads; use Docker Compose `secrets` for credential material and private keys.
- Add an example `.env` file used to parameterize secret-file locations that Compose `secrets.file` references, without injecting database secrets directly as plain env vars.
- Constrain network exposure so only required inter-node/service ports are reachable internally, while exposing node API, debug API, and PostgreSQL client ports externally.
- Update operator-facing docs so container deployment is the expected/default path, non-container/manual deployment is secondary, and the main quick-start path for using the project is the Docker Compose setup created by this task.

**Context from research:**
- The repository currently has no `Dockerfile`/Compose assets; Docker support is greenfield.
- Runtime/process wiring requires real PostgreSQL binaries configured under `process.binaries.*`; container images must provide all required binaries for init/bootstrap/rewind/start/stop flows.
- Existing docs include deployment content but are not structured as container-first workflows.

**Expected outcome:**
- A new operator can run a documented `docker compose up` flow and get a working cluster (etcd3 + pgtuskmaster nodes) without ad-hoc local binary setup.
- Production and development images are clearly separated with minimal production footprint.
- Compose assets use `configs` + `secrets` correctly, and secret handling is file-based via Docker secrets.
- Documentation prioritizes containers as the primary deployment path and uses the Compose flow from this task as the real quick-start guide.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [x] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [x] Add production image definition at `Dockerfile` (or `docker/Dockerfile.prod`) with:
- [x] Multi-stage Rust build of `pgtuskmaster` binary
- [x] PostgreSQL runtime binaries installed and discoverable at configured paths
- [x] `pgtuskmaster` container entrypoint defaults to node runtime launch
- [x] No Node.js/mdBook/dev-only toolchain in production image
- [x] Add development image definition at `Dockerfile.dev` (or `docker/Dockerfile.dev`) with:
- [x] Rust/cargo toolchain and developer utilities required for local iteration
- [x] Optional docs/dev tooling allowed only in this image
- [x] Add container startup wrapper at `docker/entrypoint.sh` (or equivalent) with robust error handling and clear required env/config checks
- [x] Update `.dockerignore` to keep build context minimal and deterministic
- [x] Add single-node compose stack at `docker-compose.yml` (or `docker/compose/docker-compose.single.yml`) containing:
- [x] `etcd3` service
- [x] one `pgtuskmaster` node service
- [x] Compose `configs` for node config TOML and static config artifacts
- [x] Compose `secrets` for PostgreSQL role passwords and any TLS private material
- [x] External published ports for API/debug/PG only
- [x] Add multi-node compose stack at `docker/compose/docker-compose.cluster.yml` (or equivalent) containing:
- [x] `etcd3` service (or explicit etcd quorum if required)
- [x] at least three `pgtuskmaster` node services
- [x] per-node Compose `configs` and per-node `secrets` wiring
- [x] inter-node-only network paths for internal communication, with least-port exposure
- [x] Add Compose config assets under `docker/configs/` (or equivalent) for each node:
- [x] node runtime TOML files
- [x] managed `pg_hba`/`pg_ident` sources when applicable
- [x] Add Docker secret source files under `docker/secrets/` with committed `*.example` placeholders only; real secret values must remain git-ignored
- [x] Add `.env.docker.example` (or `.env.example`) documenting required variables for secret file paths and published ports; no plaintext secret values committed
- [x] Add a short reproducible bring-up/teardown helper flow (`make docker-up`, `make docker-down`, or scripted equivalent) for both single-node and multi-node stacks
- [x] Add smoke/integration verification for Compose assets:
- [x] `docker compose ... config` succeeds for all provided stacks
- [x] single-node stack reaches healthy API/debug/PG endpoints
- [x] multi-node stack reaches healthy API/debug/PG endpoints on each node and etcd connectivity is confirmed
- [x] no service relies on non-secret env vars for credential payloads that should be Docker secrets
- [x] Update docs to make containers the primary deployment path:
- [x] `docs/src/operator/deployment.md` rewritten to start with container-first deployment flow
- [x] `docs/src/quick-start/prerequisites.md` and `docs/src/quick-start/first-run.md` rewritten so the main quick-start/first-use path is the Docker Compose flow created by this task
- [x] add/update dedicated container deployment doc page (`docs/src/operator/container-deployment.md` or equivalent) and wire it in `docs/src/SUMMARY.md`
- [x] non-container/manual deployment content explicitly marked as advanced/secondary path
- [x] a new user can follow the docs produced by this task as the primary quick-start guide for the project
- [x] `make docs-lint` passes cleanly
- [x] `make docs-build` passes cleanly
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
</acceptance_criteria>

<plan>

## Detailed Execution Plan (Draft 1, 2026-03-07)

### 1. Lock the plan to the current repo facts before execution starts

- The repository is still greenfield for container deployment support:
  - there are no committed Dockerfiles
  - there are no committed Compose stacks
  - `Makefile` has no container-oriented helper or verification targets
  - docs are still manual-binary-first rather than container-first
- The current runtime contract already supports the file-based inputs this task needs:
  - PostgreSQL role passwords can come from file-backed `SecretSource`
  - `pg_hba` and `pg_ident` can come from file-backed `InlineOrPath`
  - runtime validation requires absolute `process.binaries.*` paths
- The API and debug surfaces share the same listener:
  - there is one `api.listen_addr`
  - `debug.enabled = true` enables `/debug/*` routes on that same port
  - execution must not invent a second debug listener or a second debug port
- Docker and Docker Compose are available in the execution environment today:
  - `docker --version` succeeded on 2026-03-07
  - `docker compose version` succeeded on 2026-03-07
- The official `postgres:16-bookworm` image exposes the required PostgreSQL binaries at absolute paths under `/usr/lib/postgresql/16/bin/`, which matches the runtime validator's absolute-path requirement.

### 2. Product decisions this execution must follow

- Use `docker/Dockerfile.prod` for the production image and `docker/Dockerfile.dev` for the development image. Keep the repo root clear of top-level Dockerfiles.
- Use `postgres:16-bookworm` as the production runtime base so execution does not need to invent a PostgreSQL packaging story and can wire real absolute binary paths:
  - `/usr/lib/postgresql/16/bin/postgres`
  - `/usr/lib/postgresql/16/bin/pg_ctl`
  - `/usr/lib/postgresql/16/bin/initdb`
  - `/usr/lib/postgresql/16/bin/pg_basebackup`
  - `/usr/lib/postgresql/16/bin/pg_rewind`
  - `/usr/lib/postgresql/16/bin/psql`
- Keep the production image minimal:
  - only the compiled `pgtuskmaster` binary, its shell entrypoint, required runtime packages, and PostgreSQL 16 runtime binaries
  - no Node.js, mdBook, cargo, or other dev-only tooling
- Keep the development image on the same PostgreSQL 16 runtime base, then add Rust/cargo and local iteration tooling on top so production and development behavior stay path-compatible.
- Put Compose files under `docker/compose/`:
  - `docker/compose/docker-compose.single.yml`
  - `docker/compose/docker-compose.cluster.yml`
- Use a single etcd v3 service for the single-node and multi-node starter stacks unless execution finds a concrete incompatibility. The task wording requires etcd v3 plus multiple `pgtuskmaster` nodes, not necessarily a multi-member etcd quorum.
- Treat the Compose assets as container-first quick-start/lab deployments:
  - keep PostgreSQL role passwords and any private keys file-based via Docker secrets
  - keep API auth disabled in the starter stacks unless execution also adds a safe file-backed rendering path for role tokens
  - explain clearly in docs that debug routes ride the same API port and that production hardening still requires deliberate TLS/auth choices
- Make container verification non-optional by wiring it into repo-owned automation rather than leaving it as a README-only manual step.

### 3. Files and modules to add or edit during `NOW EXECUTE`

- Container image and entrypoint surface:
  - `docker/Dockerfile.prod`
  - `docker/Dockerfile.dev`
  - `docker/entrypoint.sh`
  - `.dockerignore`
- Compose, config, env, and secrets surface:
  - `docker/compose/docker-compose.single.yml`
  - `docker/compose/docker-compose.cluster.yml`
  - `docker/configs/single/node-a/runtime.toml`
  - `docker/configs/cluster/node-a/runtime.toml`
  - `docker/configs/cluster/node-b/runtime.toml`
  - `docker/configs/cluster/node-c/runtime.toml`
  - `docker/configs/common/pg_hba.conf`
  - `docker/configs/common/pg_ident.conf`
  - `docker/secrets/*.example`
  - `.env.docker.example`
  - `.gitignore`
- Verification automation and developer entrypoints:
  - `Makefile`
  - `tools/docker/compose-config-check.sh`
  - `tools/docker/smoke-single.sh`
  - `tools/docker/smoke-cluster.sh`
  - any small helper scripts needed by those smoke checks, kept under `tools/docker/`
- Docs that must become container-first:
  - `docs/src/SUMMARY.md`
  - `docs/src/operator/deployment.md`
  - `docs/src/operator/container-deployment.md`
  - `docs/src/operator/configuration.md`
  - `docs/src/quick-start/prerequisites.md`
  - `docs/src/quick-start/first-run.md`
  - `docs/src/quick-start/initial-validation.md`
  - `docs/src/contributors/testing-system.md`
- Only patch runtime Rust code if execution discovers a concrete blocker for file-backed secret/config consumption inside containers. Do not invent schema churn without evidence.

### 4. Execution phase A: build the image surface first and keep binary paths deterministic

- Create `docker/Dockerfile.prod` as a multi-stage build:
  - builder stage compiles `pgtuskmaster`
  - runtime stage starts from `postgres:16-bookworm`
  - copy the compiled binary into a stable path such as `/usr/local/bin/pgtuskmaster`
  - copy `docker/entrypoint.sh`
  - run as the `postgres` user so managed PostgreSQL ownership remains coherent with the base image defaults
- Create `docker/Dockerfile.dev` from the same PostgreSQL 16 runtime base and layer on:
  - Rust toolchain
  - cargo
  - common developer utilities needed for local iteration in this repo
  - optional docs tooling only here, never in the production image
- Implement `docker/entrypoint.sh` with strict shell behavior and explicit checks:
  - require a readable config file path, defaulting to a container path such as `/etc/pgtuskmaster/runtime.toml`
  - verify the expected secret/config files referenced by the containerized runtime configs exist before launch
  - create required runtime directories if they are missing and permissions allow it
  - emit precise errors for missing config, missing secrets, or unreadable directories
  - `exec` into `pgtuskmaster --config ...`
- Expand `.dockerignore` so build context excludes:
  - `target/`
  - `.tools/`
  - docs build output
  - logs
  - repo-local state directories that do not belong in image builds

### 5. Execution phase B: add Compose stacks, configs, volumes, and file-based secret handling

- Create a single-node stack in `docker/compose/docker-compose.single.yml` with:
  - one etcd service, internal-only
  - one `pgtuskmaster` node service
  - named volumes for PGDATA and any runtime logs that should persist across restarts
  - published host ports for:
    - PostgreSQL client traffic
    - the node API port, which also serves debug routes when `debug.enabled = true`
  - no published etcd port by default
- Create a cluster stack in `docker/compose/docker-compose.cluster.yml` with:
  - one etcd service
  - three node services: `node-a`, `node-b`, `node-c`
  - distinct named volumes per node
  - distinct published API and PostgreSQL ports per node
  - an internal bridge network used for inter-node PostgreSQL reachability and etcd access
- Use Compose `configs` for static, tracked configuration artifacts:
  - per-node runtime TOML files
  - shared `pg_hba.conf`
  - shared `pg_ident.conf`
- Use Compose `secrets` for sensitive, untracked inputs:
  - superuser password
  - replicator password
  - rewinder password
  - optional TLS private material if execution adds a hardening variant
- Commit only placeholder/example secret files under `docker/secrets/` and ignore live secret material in `.gitignore`.
- Add `.env.docker.example` at repo root for Compose interpolation of:
  - secret source file paths used by `secrets.file`
  - published host ports
  - image tag overrides if needed for local iteration
- Because `.gitignore` currently ignores `.env*`, execution must add an explicit unignore rule for `.env.docker.example` so the example file is tracked without weakening the ignore policy for real env files.
- Keep the runtime TOML files pointing at container-internal secret paths such as `/run/secrets/...`; do not pass database secrets to containers via plain environment variables.

### 6. Execution phase C: make the runtime configs explicitly container-aware but not schema-divergent

- Author runtime TOML files under `docker/configs/` that use the existing `config_version = "v2"` schema without special container-only code paths.
- For every node config:
  - set `process.binaries.*` to the real absolute paths from the PostgreSQL 16 runtime image under `/usr/lib/postgresql/16/bin/`
  - use container-local data, socket, and log paths
  - set `dcs.endpoints` to the Compose-internal etcd URL
  - set `cluster.name` and `dcs.scope` consistently inside each topology
  - set unique `cluster.member_id` values per node
  - set `postgres.listen_host` to the service hostname the other nodes can actually reach
  - set `api.listen_addr` to bind inside the container on the published API port
  - set `debug.enabled = true` in the starter stacks so the externally published API port really exposes both operational and debug routes
- Use Compose configs for `pg_hba` and `pg_ident` source files rather than embedding them inline inside the runtime TOML.
- Reuse the existing configuration guide semantics instead of adding container-only parser affordances. If execution hits a genuine ergonomics gap, record it precisely and only then decide whether code changes are justified.

### 7. Execution phase D: add repo-owned helper and verification targets, and make container smoke checks mandatory

- Add explicit container helper targets to `Makefile`, for example:
  - `docker-compose-config`
  - `docker-up`
  - `docker-down`
  - `docker-up-cluster`
  - `docker-down-cluster`
  - `docker-smoke-single`
  - `docker-smoke-cluster`
- Back those targets with checked-in scripts under `tools/docker/` so the logic stays readable and can fail with strong diagnostics.
- Add a new `ensure-docker` Make target that fails clearly if Docker or Compose are unavailable.
- The compose verification scripts must cover:
  - `docker compose ... config` for both stack files
  - image build success
  - stack bring-up and teardown with cleanup on failure
  - API health checks against `/ha/state`
  - debug route checks against `/debug/verbose` or `/debug/snapshot` on the same published API port
  - PostgreSQL reachability with `psql` or equivalent containerized probes
  - etcd connectivity confirmation, preferably via `etcdctl endpoint health` inside the etcd container if available in the chosen image
- Make these checks non-optional by extending `make test-long` to run:
  - the current ultra-long nextest profile
  - the Compose config check target
  - the single-node smoke target
  - the cluster smoke target
- Update the `make test-long` description and evidence output so it no longer claims to run only ultra-long HA scenarios once container smoke checks are part of the target.

### 8. Execution phase E: rewrite docs so container deployment is the default operator path

- Add `docs/src/operator/container-deployment.md` as the main operator-facing container guide:
  - explain image variants
  - explain the secrets/configs layout
  - document single-node and cluster bring-up
  - document teardown and cleanup
  - document how published API and PostgreSQL ports map to the services
  - explain that debug routes are served from the same API port when enabled
- Rewrite `docs/src/quick-start/prerequisites.md` so the primary prerequisites are:
  - Docker
  - Docker Compose
  - copying `.env.docker.example`
  - creating real secret files locally
- Rewrite `docs/src/quick-start/first-run.md` around the single-node Compose flow instead of manual binary execution.
- Update `docs/src/quick-start/initial-validation.md` so validation uses the Compose-exposed API/debug/PG endpoints and the checked-in helper targets.
- Rewrite `docs/src/operator/deployment.md` to start with the container topology and explicitly demote manual/non-container deployment to an advanced/secondary path.
- Touch `docs/src/operator/configuration.md` only where necessary to:
  - align example file paths with the new container docs
  - explain the file-backed secret/config pattern the Compose assets use
  - keep role-token caveats explicit if the quick-start still leaves API auth disabled
- Update `docs/src/contributors/testing-system.md` so it no longer claims `make test-long` is only the ultra-long nextest profile once Compose smoke validation is wired into that gate.
- Wire the new page into `docs/src/SUMMARY.md`.
- Run both `make docs-lint` and `make docs-build` explicitly so the task evidence matches the acceptance criteria rather than relying on `make lint` transitively covering docs lint.

### 9. Parallel execution split for the later `NOW EXECUTE` pass

- Worker A ownership:
  - `docker/Dockerfile.prod`
  - `docker/Dockerfile.dev`
  - `docker/entrypoint.sh`
  - `.dockerignore`
  - responsibility: image construction, runtime binary path alignment, and entrypoint hardening
- Worker B ownership:
  - `docker/compose/docker-compose.single.yml`
  - `docker/compose/docker-compose.cluster.yml`
  - `docker/configs/**`
  - `docker/secrets/**`
  - `.env.docker.example`
  - `.gitignore`
  - responsibility: Compose topology, file-backed configs/secrets wiring, and environment example handling
- Main agent ownership:
  - `Makefile`
  - `tools/docker/**`
  - docs rewrites
  - final integration
  - full verification
  - task checkbox updates and closeout
- If execution discovers heavy overlap between Compose assets and verification scripts, keep the Compose definitions with Worker B and move only pure helper wrappers to the main agent to avoid clobbering shared YAML edits.

### 10. Exact execution order for the later `NOW EXECUTE` pass

1. Spawn Worker A and Worker B on the split above.
2. Locally prepare the verification-script and Makefile changes because those depend on the known acceptance boundary.
3. Integrate Worker A's image and entrypoint changes.
4. Integrate Worker B's Compose/config/secret changes.
5. Run `docker compose ... config` against both stacks and fix structural YAML/config issues before any further doc work.
6. Run the single-node smoke flow and fix startup or secret/config path issues.
7. Run the multi-node cluster smoke flow and fix inter-node reachability, etcd, or role/bootstrap issues.
8. Rewrite docs only after the Compose commands and published ports are stable.
9. Tick acceptance boxes that are satisfied by actual file state and smoke evidence.
10. Run the full required verification sequence in this order:
   - `make docs-lint`
   - `make docs-build`
   - `make check`
   - `make test`
   - `make test-long`
   - `make lint`
11. Only after every required gate passes:
   - set `<passes>true</passes>`
   - run `/bin/bash .ralph/task_switch.sh`
   - commit with `task finished 01-task-container-first-docker-deployment-and-compose: ...`
   - `git push`

### 11. Risks and assumptions the required `TO BE VERIFIED` pass must challenge

- Risk: this draft assumes a single etcd service is enough for the cluster starter stack. The skeptical pass must re-check whether the task intent actually requires a three-member etcd Compose topology and alter the plan if that is the better default.
- Risk: this draft chooses to keep API auth disabled in the starter stacks because role tokens are plain strings in the current schema. The skeptical pass must decide whether execution should instead add a safe secret-rendering path and enable role-token auth by default.
- Risk: this draft wires container smoke checks into `make test-long`. The skeptical pass must challenge whether that is the best mandatory gate or whether another always-run path is more appropriate.
- Risk: docs may need a larger rewrite than the named pages imply, especially if `docs/src/quick-start/index.md` or contributor docs still describe manual-first expectations after the container flow lands.
- Risk: changing `make test-long` from "ultra-long nextest only" into a broader long-running validation gate will create stale contributor documentation unless `docs/src/contributors/testing-system.md` is updated in the same change.
- Risk: `.gitignore` and secret placeholder handling can easily regress into either tracking live secrets or ignoring the committed example files. The skeptical pass must verify the exact ignore pattern strategy.
- The `TO BE VERIFIED` pass must alter at least one concrete part of this plan before replacing the marker with `NOW EXECUTE`.

</plan>

NOW EXECUTE
