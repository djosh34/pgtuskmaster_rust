## Task: Move HA Scenario Execution Into A Per-Scenario Runner Container And Remove Docker-Daemon Polling <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Replace the current `tests/ha` execution model, where the host test process repeatedly uses `docker exec`, `docker inspect`, `docker compose ps`, and short-lived `psql` processes as a high-frequency control and observation path, with a split architecture:

1. a thin host-side pre-test orchestration/setup layer that materializes the fixture, launches the compose project, starts a dedicated scenario runner container, waits for completion, and performs final cleanup and rescue;
2. one long-lived runner container per HA scenario that joins the same per-scenario Docker network as the cluster under test and executes the scenario from inside that network with direct API and PostgreSQL connections rather than daemon-mediated shellouts.

The higher-order goal is to stop using the single Docker daemon as a high-frequency RPC bus. Research on this repo showed that `dockerd` and `containerd` dominate CPU during large HA fanout because the harness repeatedly spawns commands inside containers and repeatedly asks the daemon for state. The desired end state is that full HA parallelism is limited primarily by real cluster work, real PostgreSQL work, and real network fault injection, not by harness-induced Docker control-plane churn.

This task must also make HA runs easier to debug. Each HA scenario should have its own long-lived runner container and its own mounted per-run log/artifact directory, so the runner's stdout/stderr, structured timeline, command results, and failure context are isolated per scenario instead of being reconstructed from repeated `docker exec` calls.

**Decisions already made from research and user discussion:**
- There should be one runner container per HA scenario / compose project. The current suite already creates one isolated compose project per scenario; the new design should preserve that isolation and put the scenario-execution process inside that same network.
- The current passive `observer` sidecar is the natural place to land this redesign. It is acceptable to repurpose the existing `observer` service into an active HA runner service, or replace it with a semantically clearer service name such as `ha-runner`, as long as the result is still one long-lived per-scenario container and not a new burst of short-lived `docker exec` processes.
- The architecture must be explicitly split into host-side setup/orchestration and in-container scenario execution. The host side should remain responsible for fixture materialization, compose startup/shutdown, waiting for completion, and rescue cleanup. The runner container should remain responsible for in-network observation and SQL/data-plane checks.
- The new steady-state observation path must not depend on repeated `docker exec`, `docker inspect`, `docker compose ps`, or `docker ps` polling. Those daemon calls are acceptable only for explicit control actions or final rescue/debug work, not for normal status polling.
- The runner container should directly talk to the node APIs over the scenario network and directly talk to PostgreSQL over the scenario network. It should reuse long-lived HTTP and PostgreSQL client connections where appropriate instead of recreating a `pgtm` or `psql` process on every poll.
- It is acceptable for the runner container to keep access to `/var/run/docker.sock` if that is the cleanest way to retain explicit control actions such as kill/start/stop/restart/container-state fault injection/network fault wiring, but the socket-backed control path must be narrow and typed. It must not become a replacement steady-state polling channel.
- The old host-side cleanup/rescue path must remain available as a fallback. If the runner container wedges, exits unexpectedly, times out, or fails to clean up its own side effects, the host side must still be able to perform the current-style `docker compose down -v --remove-orphans`, artifact rescue, and forceful cleanup. The task must preserve the ability to clean up "the old way if needed".
- Do not preserve the old host-side observation/execution model as a long-lived parallel implementation "for safety". Keep only the old cleanup/rescue capabilities as fallback. The normal HA scenario path should move fully to the in-network runner-container model.
- This is a test-only architecture change. Do not expand the production library/API surface under `src/` just to host the runner. If a new binary/crate is needed, it should live in a dev/test-only boundary such as a dedicated support crate or another non-production surface.
- No backwards compatibility is required for the old HA harness internals. This is greenfield test infrastructure. Large rewrites are acceptable if they reduce code, simplify ownership, and remove daemon-churn anti-patterns.

**Scope:**
- Redesign `tests/ha` around a two-part execution model:
  - a host orchestrator layer that prepares the run workspace and drives compose lifecycle;
  - an in-network scenario runner process/container that performs the actual HA observation, SQL probing, and scenario progression.
- Replace current steady-state observation and SQL probing that go through `docker exec` inside the passive observer container with direct network clients that run inside the runner container and reuse connections.
- Rework compose materialization so each scenario includes a runner container with mounts for:
  - rendered runner config or scenario metadata;
  - the scenario run/artifact directory;
  - TLS material and tokens needed for direct API/SQL access;
  - optional Docker socket access if explicit control actions are kept in-container.
- Refactor the current HA support code so host-only responsibilities and in-network runner responsibilities are clearly separated instead of mixed through `tests/ha/support/world/mod.rs`.
- Preserve the current artifact and rescue cleanup behavior, including the ability to capture logs and forcefully tear down a compose project even if the new runner container fails partway through execution.
- Keep the suite black-box and end-to-end. The new runner container is still validating a real cluster by using public surfaces and real PostgreSQL connectivity, not internal unit-test-only helpers.

**Out of scope:**
- Do not weaken the HA suite by replacing real PostgreSQL proof/write checks with API-only checks. SQL must remain part of correctness, but it should move to persistent direct connections instead of repeated short-lived `psql` processes.
- Do not solve this by simply putting the existing host harness into a container while continuing to use `docker exec` as the normal observation mechanism. The point of the redesign is to remove Docker daemon mediation from the steady-state read path.
- Do not introduce a permanent parallel "old harness path" and "new runner-container path" toggle. Only the old cleanup/rescue mechanics may survive as fallback.
- Do not move large test-only harness code into the production runtime surface.

**Context from research:**
- `tests/ha/support/world/mod.rs` currently owns too much at once: per-scenario state, fixture materialization, compose startup, docker control actions, health waits, artifact capture, and cleanup.
- `tests/ha/support/docker/cli.rs` currently shells out to Docker/Compose for operations such as `compose up`, `compose down`, `compose ps`, `docker inspect`, `docker exec`, and `docker logs`.
- `tests/ha/support/world/mod.rs` currently resolves service container IDs through `service_container_id(...)`. This was recently improved to cache service IDs per scenario, but the larger daemon-churn problem remains elsewhere.
- The existing passive observer sidecar is materialized in `tests/ha/support/world/mod.rs` compose rendering. It is one container per scenario, not three containers, but it mounts three per-node observer config files and currently serves as the place where repeated command execs occur.
- `tests/ha/support/observer/pgtm.rs` is a major hotspot:
  - `state()` fans out across all three observer configs and runs `pgtm status --json` once per config.
  - `primary_tls_json()` does the same for `pgtm primary --json --tls`.
  - `replicas_tls_json()` does the same for `pgtm replicas --json --tls`.
  - All of these are currently implemented as `docker exec` into the observer container.
- `tests/ha/support/observer/sql.rs` is another hotspot:
  - every SQL probe runs `/usr/lib/postgresql/16/bin/psql` through `docker exec`;
  - this recreates a process, connection setup, and TLS/session state for every probe.
- Hot loops in `tests/ha/support/steps/mod.rs` currently compose these calls together. For example:
  - authoritative-primary polling does `observer().state()` plus `current_primary_target()` plus `sql().execute("SELECT 1")` in a single iteration;
  - that means a single steady-state loop iteration can trigger approximately 7 daemon-mediated command executions before counting any additional failure checks.
- The current helper architecture explains why `dockerd` and `containerd` remain at the top of the process list even after low-risk improvements like relaxed poll intervals and compose-service ID caching. The remaining problem is not mainly `docker compose ps`; it is repeated command execution and state checks being routed through Docker.
- Current render templates under `tests/ha/support/world/mod.rs` write node API endpoints such as `https://node-a:8443` and similar PostgreSQL targets intended for in-network resolution. Those names are directly reachable from a container on the scenario network but are not directly reachable from the host today without extra port publishing and TLS plumbing.
- Because of that topology, the most coherent redesign is not "host harness plus more Docker exec". It is "runner process inside the network plus direct clients".
- `src/cli/connect.rs` already defines connection DTOs such as `ConnectionTarget`/`ConnectionView`, but the current HA harness uses them through command execution rather than through a long-lived in-process client.
- Current cleanup lives in `tests/ha/support/world/mod.rs` and includes artifact capture plus `docker compose down -v --remove-orphans`. That behavior must remain available as a host-side rescue path even after the new runner lands.
- Current nextest/image setup already builds a dedicated HA/cucumber image in `tools/nextest/setup-ha-cucumber-image.sh`. The new runner-container design should reuse or extend that pipeline instead of inventing a completely separate image flow if reuse is reasonable.

**Required target design:**
- Introduce a dedicated test-only HA runner binary/crate/service. It must not live as production API surface. It may be a new dev-support crate, another dedicated test-support binary, or another equivalent test-only ownership boundary.
- The host-side test binary should no longer directly drive repeated cluster observation logic. Instead it should:
  - materialize the run directory and compose file;
  - start the compose project and the runner container;
  - provide the runner with scenario identity and any needed config/artifact mounts;
  - wait for the runner result and map that to the test result;
  - perform rescue artifact collection and old-style cleanup on failure or timeout.
- The runner container should:
  - live on the same per-scenario network as the nodes and etcd services;
  - directly call node APIs using persistent HTTP clients and the mounted TLS/auth material;
  - directly call PostgreSQL using long-lived PostgreSQL client connections or a bounded connection pool;
  - write structured logs, timeline, and step/scenario result artifacts into the mounted run directory;
  - report a clear success/failure result that the host orchestrator can consume.
- The runner should no longer spawn `pgtm` or `psql` as a normal polling mechanism. If there is still any use of CLI processes, it must be a narrow, justified exception and not the normal steady-state path.
- Explicit cluster-control actions remain required:
  - kill/start/restart individual node containers;
  - stop/start DCS services;
  - fault injection such as network isolation or path blocking;
  - log and state capture on failure.
  These actions may still use Docker, but they must be routed through a narrow typed control surface and must not become the steady-state read path.
- If Docker socket access is mounted into the runner container, the task must document and implement a narrow control adapter that is used only for explicit control operations. Read-only steady-state cluster observation must still happen through direct API/SQL connections, not through the daemon.
- If the runner container crashes or becomes wedged, the host side must still be able to:
  - collect whatever logs/artifacts exist;
  - run the current cleanup path;
  - remove remaining compose resources.

**Expected outcome:**
- Each HA scenario runs with one long-lived in-network runner container instead of a passive observer plus repeated host-driven `docker exec`.
- Each scenario has cleaner, scenario-scoped logs and artifacts because the runner is long-lived and mounted into the run directory.
- `tests/ha` steady-state polling stops depending on Docker daemon round-trips for normal state/SQL reads.
- `dockerd` and `containerd` are no longer the dominant cost simply because the harness is polling; remaining daemon work should primarily reflect explicit cluster-control operations and final lifecycle cleanup.
- The old host-side cleanup path still works as fallback/rescue and is not regressed by the rewrite.
</description>

<acceptance_criteria>
- [ ] Add a dedicated test-only HA runner ownership boundary. This must be a new dev/test-only binary or crate and must not enlarge the production runtime/library surface under `src/` as public API. The task implementation must document where the runner lives and why that ownership boundary is correct.
- [ ] Refactor the current `tests/ha` architecture so host-side orchestration and in-network scenario execution are separate responsibilities instead of being mixed through the current host-driven harness.
- [ ] Update the HA compose materialization path under `tests/ha/support/world/mod.rs` (and any adjacent materialization modules) so each per-scenario compose project includes one long-lived runner container/service attached to the scenario network and mounted to the per-run artifacts directory.
- [ ] Replace the current passive observer-sidecar usage. It is acceptable either to repurpose the existing `observer` service into the active scenario runner or to rename it to a clearer runner service, but the final design must have one long-lived per-scenario runner container and must not preserve the old passive-tail-plus-docker-exec model.
- [ ] Remove normal steady-state reliance on `docker exec` for `pgtm status`, `pgtm primary`, `pgtm replicas`, and SQL polling. Current logic in `tests/ha/support/observer/pgtm.rs` and `tests/ha/support/observer/sql.rs` must be rewritten so normal cluster observation happens through direct network clients running inside the runner container.
- [ ] Eliminate repeated short-lived `psql` probing as the normal SQL path. The new runner must use long-lived PostgreSQL connections or a bounded connection pool so SQL/data-plane validation is not implemented as a fresh process and connection for every poll iteration.
- [ ] Eliminate repeated all-three-seed polling as the mandatory steady-state path. The new runner may still perform all-node or all-seed checks when needed for ambiguity resolution, explicit invariants, or failure diagnostics, but the normal observation path must not fan out across all three nodes on every poll by default.
- [ ] Preserve real HA correctness checks. The rewrite must keep SQL proof-row and writeability checks, real API observation, and real failover/recovery behavior. Do not replace those with weaker mock or API-only assertions.
- [ ] Preserve explicit Docker-backed control actions for cluster mutation and fault injection. The new design must still support kill/start/restart/network-fault/DCS-fault actions. If Docker socket access is mounted into the runner container, the implementation must use a narrow typed control adapter for explicit actions only and must not reintroduce daemon polling as a read path.
- [ ] Preserve old-style rescue cleanup. If the runner container fails, times out, or wedges, the host-side harness must still be able to run the current cleanup path, capture artifacts/logs as best-effort rescue data, and tear down the compose project with the old mechanism.
- [ ] Preserve or improve artifact quality. The runner container must write per-scenario logs, structured result/timeline data, and failure context into the mounted run directory so each scenario is easier to inspect independently after failure.
- [ ] Update nextest/image setup (`tools/nextest/setup-ha-cucumber-image.sh` and any related Docker/image definitions) so the runner binary/service is built into the HA test image or otherwise made available in a coherent, documented way without inventing an ad hoc parallel image flow unless that is clearly simpler.
- [ ] Update all affected HA support modules and tests under `tests/ha/support/`, including `world`, `observer`, `docker`, `runner`, and any new runner-control/config modules, so there is one coherent architecture rather than a long-lived parallel old/new implementation.
- [ ] Add focused tests for the new split architecture. At minimum this must include:
  - unit/integration coverage for runner config/materialization and host/runner contract parsing;
  - focused validation that the runner container is rendered with the correct mounts/network wiring;
  - focused validation that fallback cleanup still executes when the runner exits unsuccessfully or cannot be reached.
- [ ] Validate the new architecture with at least one focused real HA scenario run through `cargo nextest` during development, then finish with the required repo gates.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
