# Docker Compose QA Walkthrough

## Scope

Task: QA the shipped `docker compose up` operator walkthrough from a fresh host-side operator perspective.

Date: 2026-03-24

Repo root: `/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust`

Evidence directory:

- `.ralph/evidence/docker-compose-qa-walkthrough/`

## Discovery Notes

### What I read before searching

- `README.md`
- `docs/src/tutorial/first-ha-cluster.md`
- `docs/src/how-to/check-cluster-health.md`
- `docs/src/how-to/perform-switchover.md`

### What was easy to discover

- The repository ships a local Docker HA stack.
- The intended host-side operator config is `docker/pgtm.toml`.
- The main operator commands to validate are `status`, `primary`, `replicas`, and `switchover`.

### First discovery failure

The docs assumed `pgtm` was already installed. The first supported install path I could discover only after leaving the docs surface was:

```bash
make install-pgtm
```

That gap is fixed in this task by updating the README and tutorial surface to teach the host install step explicitly.

### Additional truth gaps discovered

- The shipped Docker node runtime configs had drifted from the active runtime schema.
- The runtime and DCS types originally published only one PostgreSQL route per member, which made `pgtm primary` and `pgtm replicas` output in-cluster addresses like `node-c:5432` instead of host-routable DSNs.
- `pgtm switchover request` cannot be issued through a non-primary seed config even when the CLI already knows which member is the authoritative primary.
- The generic switchover flow can strand the cluster without an authoritative primary until the operator manually clears the switchover request.

## Install Experience

Commands:

```bash
make install-pgtm
pgtm --help
```

Outcome:

- `make install-pgtm` succeeded and installed the current `pgtm` binary to `~/.cargo/bin/pgtm`.
- The install itself was one command once discovered.
- The discovery path was not obvious from the original docs, so the operator-facing docs were updated in this task.
- `pgtm` still has `--help`; I did not rely on any hidden install steps or private local state.

## Fixes Applied Before Re-running QA

### Code and config boundary fixes

- Added a shared `PgRoute` model and moved `PgConnInfo` to use `route`.
- Split DCS member PostgreSQL routing into:
  - `cluster_postgres`
  - `operator_postgres`
- Changed CLI connection rendering so `primary` and `replicas` prefer `operator_postgres` and fall back to `cluster_postgres`.
- Replaced runtime `postgres.network.advertise_port` with:
  - `postgres.network.cluster_advertise`
  - `postgres.network.operator_advertise`

### Shipped Docker config fixes

- Updated `docker/node-a.toml`, `docker/node-b.toml`, and `docker/node-c.toml` to the current runtime schema.
- Published both internal and host-routable PostgreSQL routes:
  - `cluster_advertise = { host = "node-x", port = 5432 }`
  - `operator_advertise = { host = "node-x", port = 1500x, hostaddr = "127.0.0.1" }`

### Operator docs fixes

- Centered the README and `first-ha-cluster` tutorial on:
  - `make install-pgtm`
  - repo-root `docker compose up -d --build`
  - canonical host config `docker/pgtm.toml`
- Updated the runtime, DCS-state, and CLI reference pages to describe the new dual-route model truthfully.
- Corrected `docs/src/how-to/check-cluster-health.md` so it no longer claims peer API sampling that the current CLI does not perform.

## Command Log

### 1. Fresh host install and clean stack reset

Commands:

```bash
make install-pgtm
docker compose down -v --remove-orphans
docker compose up -d --build
docker compose ps
```

Intent:

- install the current host CLI
- start from the repo-root compose entrypoint
- verify the shipped stack boots cleanly

Outcome:

- install succeeded
- `docker compose up -d --build` from the repo root succeeded
- the shipped stack came up with:
  - APIs on `18081`, `18082`, `18083`
  - PostgreSQL on `15001`, `15002`, `15003`

### 2. Baseline host-side operator view

Commands:

```bash
pgtm -c docker/pgtm.toml status
pgtm -c docker/pgtm.toml --json
pgtm -c docker/pgtm.toml status -v
timeout 10 pgtm -c docker/pgtm.toml status --watch
pgtm -c docker/pgtm.toml primary
pgtm -c docker/pgtm.toml primary --tls
pgtm -c docker/pgtm.toml replicas
```

Intent:

- validate the documented read-only operator surface from the host

Outcome:

- `status`, `--json`, `status -v`, and `status --watch` all worked
- `primary` returned a host-routable DSN such as:
  - `host=node-c hostaddr=127.0.0.1 port=15003 ...`
- `replicas` returned host-routable DSNs such as:
  - `host=node-a hostaddr=127.0.0.1 port=15001 ...`
  - `host=node-b hostaddr=127.0.0.1 port=15002 ...`
- The DCS JSON payload now truthfully showed both:
  - `cluster_postgres`
  - `operator_postgres`

### 3. Direct host PostgreSQL validation

Commands:

```bash
env PGPASSWORD="$(cat docker/secrets/postgres-superuser-password)" \
  psql "$(pgtm -c docker/pgtm.toml primary --tls)" \
  -c 'SELECT pg_is_in_recovery();'
```

Intent:

- prove that the resolved host-side DSN can actually be used with `psql`

Outcome:

- the command succeeded
- `pg_is_in_recovery()` returned `f`
- this confirmed that the host-resolved primary DSN was now directly usable from the host

### 4. Targeted switchover validation

Observed starting condition:

- `docker/pgtm.toml` was seeded through `node-a`
- the authoritative primary was `node-c`

Commands:

```bash
pgtm -c docker/pgtm.toml switchover request --switchover-to node-b
pgtm -c docs/examples/docker-cluster-node-c.toml switchover request --switchover-to node-b
timeout 20 pgtm -c docker/pgtm.toml status --watch
pgtm -c docker/pgtm.toml status -v
pgtm -c docker/pgtm.toml primary --tls
pgtm -c docker/pgtm.toml replicas --tls
pgtm -c docs/examples/docker-cluster-node-b.toml switchover clear
pgtm -c docker/pgtm.toml status -v
```

Intent:

- validate targeted switchover
- verify DSN truth after role changes
- verify manual clear flow

Outcome:

- `pgtm -c docker/pgtm.toml switchover request --switchover-to node-b` failed because the seed node was not the authoritative primary
- the same targeted request succeeded through `docs/examples/docker-cluster-node-c.toml`, which pointed at the actual primary
- the cluster converged with `node-b` as primary
- `primary --tls` and `replicas --tls` updated to the correct host-mapped routes after the handoff
- the switchover marker stayed pending until I manually cleared it via the new primary's seed config

Assessment:

- targeted switchover itself worked
- the canonical config not being able to issue the request is a real bug
- the lingering pending marker is part of the broader switchover-lifecycle bug tracked below

### 5. Failure and recovery validation

Commands:

```bash
docker compose kill node-b
pgtm -c docker/pgtm.toml status -v
timeout 20 pgtm -c docker/pgtm.toml status --watch
docker compose start node-b
timeout 20 pgtm -c docker/pgtm.toml status --watch
pgtm -c docker/pgtm.toml status -v
```

Intent:

- verify the cluster view stays truthful during node loss and rejoin

Outcome:

- immediately after the kill, `status -v` still showed the last authoritative view, which is consistent with lease-based DCS publication
- within the watch window, the cluster converged with `node-a` promoted to primary while `node-b` was down
- after `docker compose start node-b`, the cluster converged back to a healthy three-member state with `node-b` rejoined as replica

### 6. Negative-path validation

Evidence-local bad configs created:

- `.ralph/evidence/docker-compose-qa-walkthrough/bad-api-token.toml`
- `.ralph/evidence/docker-compose-qa-walkthrough/bad-api-port.toml`

Commands:

```bash
pgtm -c .ralph/evidence/docker-compose-qa-walkthrough/bad-api-token.toml status
pgtm -c .ralph/evidence/docker-compose-qa-walkthrough/bad-api-port.toml status
env PGPASSWORD='wrong-password' \
  psql "$(pgtm -c docker/pgtm.toml primary --tls)" \
  -c 'SELECT 1;'
```

Intent:

- verify wrong auth, wrong API connectivity, and wrong PostgreSQL password all fail clearly

Outcome:

- bad API token failed clearly with:
  - `api request failed with status 401: unauthorized`
- bad API port failed clearly with:
  - `transport error: error sending request for url (https://127.0.0.1:18099/state)`
- wrong PostgreSQL password failed clearly with:
  - `FATAL: password authentication failed for user "postgres"`

### 7. Generic switchover validation

Commands:

```bash
pgtm -c docker/pgtm.toml switchover request
timeout 20 pgtm -c docker/pgtm.toml status --watch
pgtm -c docker/pgtm.toml status -v
pgtm -c docs/examples/docker-cluster-node-b.toml status -v
pgtm -c docs/examples/docker-cluster-node-c.toml status -v
pgtm -c docker/pgtm.toml switchover clear
timeout 30 pgtm -c docker/pgtm.toml status --watch
```

Intent:

- validate the documented generic switchover request flow

Outcome:

- the generic request returned `accepted=true`
- the cluster then entered a degraded state with:
  - no authoritative primary
  - a still-pending `switchover: auto` marker
  - node-b oscillating between `replica` and `unknown` while running a promote job
  - node-c eventually reporting `no_quorum` and no DCS member slots from its own seed view
- `switchover clear` immediately allowed the cluster to recover back to a healthy state with `node-b` primary

Assessment:

- the documented generic switchover path is not yet truthful for the shipped Docker stack
- manual clear is currently required as a recovery move

### 8. Teardown

Command:

```bash
docker compose down -v --remove-orphans
```

Outcome:

- all containers, volumes, and the compose network were removed cleanly

## What Worked

- repo-root `docker compose up -d --build`
- host install path `make install-pgtm`
- `pgtm -c docker/pgtm.toml status`
- `pgtm -c docker/pgtm.toml --json`
- `pgtm -c docker/pgtm.toml status -v`
- `timeout 10 pgtm -c docker/pgtm.toml status --watch`
- `pgtm -c docker/pgtm.toml primary`
- `pgtm -c docker/pgtm.toml primary --tls`
- `pgtm -c docker/pgtm.toml replicas`
- direct `psql "$(pgtm -c docker/pgtm.toml primary --tls)"` with the correct password
- targeted switchover through the current primary seed config
- node kill and restart workflow
- negative auth/connectivity/password failures with clear messages
- environment teardown

## What Failed Or Needed Follow-up

- The canonical config `docker/pgtm.toml` cannot submit `switchover request` when its fixed seed is not the authoritative primary.
- Switchover lifecycle handling is not self-clearing:
  - targeted switchover left the pending marker in place until manual clear
  - generic switchover left the cluster degraded without a primary until manual clear

## Bug Tasks Created

- `.ralph/tasks/bugs/switchover-request-fails-via-non-primary-seed.md`
- `.ralph/tasks/bugs/generic-switchover-request-can-stall-without-primary.md`

## Repository Validation

Commands:

```bash
make check
make lint
make test
make test-long
```

Outcome:

- `make check` passed
- `make lint` passed
- `make test` passed
- `make test-long` passed

## Final Assessment

- The shipped Docker walkthrough is now materially better and truthful for the host-side read path:
  - the repo-root compose entrypoint works
  - `docker/pgtm.toml` works from the host
  - `primary` and `replicas` now emit host-routable DSNs
  - direct `psql` connections from the host succeed
- The negative-path handling for wrong auth, wrong port, and wrong PostgreSQL password is clear and operator-usable.
- Two real switchover bugs remain and are now tracked explicitly:
  - writes cannot be issued through a non-primary seed config
  - switchover requests can remain pending or strand the cluster until manual clear
