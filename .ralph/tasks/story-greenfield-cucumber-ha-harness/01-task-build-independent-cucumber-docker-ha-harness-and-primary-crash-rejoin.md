## Task: Build Independent Cucumber Docker HA Harness And Primary Crash Rejoin Feature <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Build a fully separate greenfield HA end-to-end test framework under `cucumber_tests/ha` that does not reuse any of the current HA test harness logic from `tests/` or `src/test_harness/ha_e2e/`. The framework must use `cucumber-rs` feature files, Docker CLI orchestration, real compiled `pgtuskmaster` and `pgtm` binaries, static checked-in fixture files for the 3-node cluster, and an observer flow that inspects the system only through `pgtm` plus `psql`. Deliver the first real feature as a primary-container-crash failover-and-rejoin scenario.

It is explicitly not a requirement that the first HA scenario already passes against the product before this task is considered complete. The requirement is that the harness exists, the first feature exists, and the feature can be executed to a trustworthy outcome. If the run exposes an HA or product failure rather than a harness failure, that failure must create a bug task in `.ralph/tasks/bugs/`, and that bug task must contain `<blocked_by>` tags for all four tasks in `story-greenfield-cucumber-ha-harness`.

**Original user shift / motivation:** The user wants to fully redesign the HA/e2e test approach because the current `tests/` tree has too much opaque custom harness logic, including custom HTTP handling that is hard to read and hard to trust. The new framework must be visibly simple, independent from the old HA harness, centered on `.feature` files, centered on the real binaries, and built so the old harness can later be deleted entirely.

**Higher-order goal:** Replace the existing HA behavioural test infrastructure with a simpler, more operator-realistic system that is easy to inspect, parallel-safe, and grounded in real Dockerized nodes rather than custom in-process control logic.

**Scope:**
- Create a new top-level test framework rooted at `cucumber_tests/ha/`.
- Keep all substantive framework logic, feature files, static givens, Docker files, and run artifacts under `cucumber_tests/ha/`; do not put framework logic under `tests/`.
- Register the cucumber feature wrappers in `Cargo.toml` as explicit `[[test]]` targets because Cargo does not support an alternate auto-discovered integration-test directory outside `tests/`.
- Use one tiny wrapper Rust file per feature file. Each feature gets its own directory containing the `.feature` file and its tiny `.rs` wrapper. The wrapper files must be almost identical and only call the shared feature runner.
- Use checked-in reusable `Given` fixture sets. For this task there must be one static `three_node_plain` Given with shared configs, shared secrets, shared Docker files, and one shared compose file. Do not create a supervised or runtime-only-kill Given in this task.
- Copy the selected Given into a gitignored repo-local run directory for each scenario run so the exact input files used by that run are preserved and inspectable. Avoid generation where copying static files is sufficient.
- Do not generate label files or compose files for Ryuk cleanup unless implementation proves it is strictly necessary. The approved default is to use the automatic Compose project label for Ryuk ownership.
- Use real compiled `pgtuskmaster` and `pgtm` binaries copied into Docker images and rely on Docker layer cache for quick rebuilds.
- Use Docker Compose with a unique project name per feature run. Use direct Docker CLI calls with explicit argument vectors only.
- Implement the first feature file and wrapper for a killed-primary-container failover scenario where a new primary is elected, a proof row can be written through the new primary, and the restarted old primary rejoins as a replica.

**Context from research:**
- The current repo contains legacy HA harness code in `tests/ha/support/*.rs`, `tests/ha_*.rs`, and `src/test_harness/ha_e2e/*.rs`. This task must not reuse those modules or their helpers.
- The current repo already contains Docker assets in `docker/compose/docker-compose.cluster.yml`, `docker/Dockerfile.prod`, and `docker/entrypoint.sh`, but those are part of the old operator/dev flow and are not an approved dependency for the new greenfield harness beyond possible inspiration when reading current runtime expectations.
- The runtime config contract is documented in `src/config/schema.rs` and `docs/src/reference/runtime-configuration.md`. The new harness may rely on the real runtime config contract, but must keep its own static fixture files under `cucumber_tests/ha/givens/...`.
- `pgtm` already exposes the required operator surfaces:
- `status --json` gives a synthesized cluster view
- `debug verbose --json` gives a per-node raw stable payload
- `primary` and `replicas` emit libpq conninfo strings suitable for shell/operator usage
- The new observer path must use `pgtm` instead of custom HTTP handlers or custom cluster-state scraping code.
- `psql` accepts a libpq conninfo string via `--dbname`, so the new framework must feed `psql` with the direct `pgtm primary` / `pgtm replicas` output rather than rebuilding connection arguments.
- Docker Compose automatically labels its resources with `com.docker.compose.project=<project>`. Docker documents this canonical label in the Compose services reference. The approved cleanup design is that the harness chooses a unique project name per feature run and registers that label pair with Ryuk, so no rendered label file is required for this task.
- Docker Compose `label_file` exists and requires Compose 2.32.2+, but it is explicitly not part of the approved default design for this task. Only introduce it in a follow-up if the automatic project label proves insufficient in practice.
- The user explicitly rejected:
- reusing existing HA harness logic from `tests/` or `src/test_harness/ha_e2e/`
- custom HTTP handlers in the new framework
- a supervised/runtime-only-kill test path in this task
- putting substantive framework logic under `tests/`
- shell-based process invocation or PATH-dependent command execution
- vague timeouts not tied to the configured HA/process settings

**Expected outcome:**
- A new independent `cucumber_tests/ha` framework exists and can run the first primary-container-crash feature end to end.
- The new framework uses real Dockerized nodes, real compiled binaries, `pgtm`-driven observation, and `psql` driven by `pgtm` conninfo.
- The first feature provides a readable operator-realistic failover story that proves election, writeability on the new primary, and replica rejoin of the restarted former primary.
- The old HA harness remains untouched by the new framework implementation and is no longer a dependency for the new feature.

</description>

<acceptance_criteria>
- [ ] `cucumber_tests/ha/` exists as a fully separate framework root and does not import or call any code from `tests/ha/`, `tests/ha_*.rs`, or `src/test_harness/ha_e2e/`
- [ ] `Cargo.toml` registers one explicit `[[test]]` target for the first cucumber feature wrapper outside `tests/`, and the wrapper is tiny and delegates entirely to shared runner code
- [ ] `cucumber_tests/ha/givens/three_node_plain/` exists with one static compose file, one static shared config set, one static shared secret set, and one shared `docker_files/` directory; there is no separate supervised Given in this task
- [ ] each feature run copies the selected Given into `cucumber_tests/ha/runs/<feature>/<run-id>/source-copy/` and writes artifacts into `cucumber_tests/ha/runs/<feature>/<run-id>/artifacts/`; `cucumber_tests/ha/runs/.gitignore` keeps the run tree untracked
- [ ] the framework starts Docker Compose with a unique project name per feature run and uses Ryuk ownership based on `com.docker.compose.project=<project>`; no rendered label file or rendered compose file is present in the shipped implementation
- [ ] every host-side and in-container process execution path uses an explicit executable path plus argument list only; no `sh -c`, no shell parsing, and no PATH-based resolution remains in the new framework
- [ ] the observer path uses only `pgtm` plus `psql`; the framework does not ship custom HTTP handlers, custom fake servers, or custom API protocol recreation for HA observation
- [ ] the first feature lives in its own directory at `cucumber_tests/ha/features/primary_crash_rejoin/` and includes both `primary_crash_rejoin.feature` and a tiny `primary_crash_rejoin.rs` wrapper; the scenario expresses a killed-primary-container failover/rejoin story with config-derived waiting semantics rather than magic-number assertions
- [ ] the first wrapper/test passes only by returning `Result` through `?`; failure paths surface as test failure without `panic!`, `unwrap`, `expect`, or shell exit-code swallowing
- [ ] the implementation makes the separation goal explicit in code comments and/or task notes: this framework is independent by design and is intended to make the legacy HA harness deletable later
- [ ] old harness reuse is fully rejected in code and repo-wide verification confirms the new framework files do not import legacy test harness modules accidentally
- [ ] docs are updated with the new cucumber HA test entrypoint and layout, and stale or misleading references are corrected where relevant
- [ ] the first feature wrapper can be executed on the new greenfield harness to a trustworthy outcome
- [ ] the first feature run produces enough evidence to distinguish harness failure from HA behavior failure in the system under test
- [ ] if the first feature exposes a trustworthy product or HA failure, a bug task is created under `.ralph/tasks/bugs/` and that bug contains `<blocked_by>` tags for:
- [ ] `.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md`
- [ ] `.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md`
- [ ] `.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md`
- [ ] `.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md`
- [ ] `<passes>true</passes>` is set only after every acceptance-criteria item and every required implementation-plan checkbox is complete
</acceptance_criteria>

## Detailed implementation plan

### Phase 1: Establish the independent cucumber HA framework root
- [ ] Create the new framework root at `cucumber_tests/ha/`.
- [ ] Create this exact checked-in structure under `cucumber_tests/ha/`:
- [ ] `features/`
- [ ] `features/primary_crash_rejoin/`
- [ ] `givens/three_node_plain/`
- [ ] `givens/three_node_plain/docker_files/`
- [ ] `givens/three_node_plain/configs/common/`
- [ ] `givens/three_node_plain/configs/node-a/`
- [ ] `givens/three_node_plain/configs/node-b/`
- [ ] `givens/three_node_plain/configs/node-c/`
- [ ] `givens/three_node_plain/configs/observer/`
- [ ] `givens/three_node_plain/secrets/`
- [ ] `support/runner/`
- [ ] `support/world/`
- [ ] `support/docker/`
- [ ] `support/givens/`
- [ ] `support/observer/`
- [ ] `support/process/`
- [ ] `support/timeouts/`
- [ ] `runs/.gitignore`
- [ ] Ensure all substantive new logic lives under `cucumber_tests/ha/`; do not add framework code to `tests/`.
- [ ] Add a repo-local ignore rule or tracked `cucumber_tests/ha/runs/.gitignore` so the run tree stays in-repo but untracked.

### Phase 2: Register the first feature as a Cargo/nextest test without using `tests/` as the framework home
- [ ] Create the first feature directory at `cucumber_tests/ha/features/primary_crash_rejoin/`.
- [ ] Create the feature file at `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.feature`.
- [ ] Create one tiny wrapper Rust file for the first feature at `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.rs`.
- [ ] Keep the wrapper almost trivial:
- [ ] import the shared feature runner from `cucumber_tests/ha/support/...`
- [ ] define one async test function returning `Result<(), String>` or a similarly explicit error type
- [ ] call the shared runner with the feature file path
- [ ] rely on `?` for failure propagation instead of `panic!`
- [ ] Update `Cargo.toml` with one explicit `[[test]]` entry pointing at `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.rs`.
- [ ] Do not attempt to use `tests/` discovery, alternate Cargo test-dir globs, or a single shared target that discovers many feature files at runtime; those approaches are explicitly rejected for this task.
- [ ] Add the necessary dev-dependencies for `cucumber-rs` and any minimal runner support crates actually required by the approved design.

### Phase 3: Build the reusable shared Given for the plain 3-node cluster
- [ ] Create `cucumber_tests/ha/givens/three_node_plain/compose.yml` as the one shared compose file for this Given.
- [ ] Keep the compose topology static and readable:
- [ ] `etcd`
- [ ] `node-a`
- [ ] `node-b`
- [ ] `node-c`
- [ ] `observer`
- [ ] Do not create an alternative supervised compose file.
- [ ] Create one static shared config set under `cucumber_tests/ha/givens/three_node_plain/configs/`:
- [ ] one `runtime.toml` per node
- [ ] one shared `pg_hba.conf`
- [ ] one shared `pg_ident.conf`
- [ ] one observer config per node seed
- [ ] Create one static shared secret set under `cucumber_tests/ha/givens/three_node_plain/secrets/`.
- [ ] Create one shared Docker files directory under `cucumber_tests/ha/givens/three_node_plain/docker_files/`:
- [ ] `node.Dockerfile`
- [ ] `observer.Dockerfile`
- [ ] Keep image naming/tagging local to the harness logic, but rely on Docker cache by copying the compiled binaries into the images.
- [ ] Do not create separate Docker files, compose files, configs, or secrets for a runtime-only-kill/supervised path in this task.

### Phase 4: Make run directories explicit and inspectable without unnecessary generation
- [ ] In the shared runner/world setup, create a run workspace at `cucumber_tests/ha/runs/<feature-name>/<run-id>/`.
- [ ] Copy the entire selected Given into `.../source-copy/` for that run so the exact input files are visible in-repo.
- [ ] Create `.../artifacts/` for captured outputs.
- [ ] Do not generate runtime configs, secrets, or compose files when copying the static Given is sufficient.
- [ ] Do not render a label file unless implementation proves Ryuk cannot reliably use the automatic Compose project label.
- [ ] If any rendered file proves strictly necessary after implementation investigation, keep it limited to one small Rust renderer module returning one `String`, write the file under that run’s `source-copy/`, and document exactly why copying was insufficient.

### Phase 5: Implement strict no-shell process execution
- [ ] Build a dedicated process-execution layer under `cucumber_tests/ha/support/process/`.
- [ ] Require every process call to provide:
- [ ] an explicit executable path
- [ ] an explicit argument vector
- [ ] explicit working directory when relevant
- [ ] explicit environment variables when relevant
- [ ] no shell invocation and no shell parsing
- [ ] For host-side Docker execution, do not rely on shell strings or implicit PATH search. Use a resolved absolute Docker binary path from a narrowly scoped config source or explicit default such as `/usr/bin/docker`, and fail clearly if it is unavailable.
- [ ] For in-container execution, use `docker exec <container> <absolute-binary-path> <args...>` only. Do not use `sh -c`.
- [ ] Ensure the new framework contains no `unwrap`, `expect`, `panic!`, or silently ignored process failures.

### Phase 6: Implement Docker and Ryuk support around the Compose project label
- [ ] Build `cucumber_tests/ha/support/docker/cli.rs` (or equivalent) to wrap only Docker CLI operations needed by the harness:
- [ ] image build
- [ ] compose up
- [ ] compose down
- [ ] compose ps / inspect / logs
- [ ] container kill
- [ ] container start
- [ ] exec in container
- [ ] Build `cucumber_tests/ha/support/docker/ryuk.rs` (or equivalent) to start one Ryuk per feature run and register ownership by `com.docker.compose.project=<unique-project>`.
- [ ] Use a unique Compose project name per feature run and make that the sole approved ownership key for this task.
- [ ] Explicitly do not render custom labels or use Compose `label_file` in the implementation unless the automatic project label proves insufficient during execution.
- [ ] Capture Docker metadata and logs into the run artifacts on failure, and preferably on success when cheap enough to keep the run inspectable.

### Phase 7: Implement the observer path entirely through `pgtm` and `psql`
- [ ] Build `cucumber_tests/ha/support/observer/pgtm.rs` (or equivalent) so all cluster observation goes through:
- [ ] `pgtm status --json`
- [ ] `pgtm debug verbose --json`
- [ ] `pgtm primary`
- [ ] `pgtm replicas`
- [ ] Build `cucumber_tests/ha/support/observer/sql.rs` (or equivalent) so SQL access uses `psql --dbname "<conninfo>"` where the conninfo comes directly from `pgtm`.
- [ ] Do not reconstruct DSNs or TLS flags in the harness once `pgtm` has resolved them.
- [ ] Do not add custom HTTP handlers or fake HA API clients.

### Phase 8: Implement the shared world and timeout model
- [ ] Build the shared cucumber world under `cucumber_tests/ha/support/world/`.
- [ ] World state must include:
- [ ] selected Given
- [ ] run-id
- [ ] run directory paths
- [ ] compose project name
- [ ] Docker/Ryuk handles
- [ ] remembered killed-node identity
- [ ] remembered new-primary identity
- [ ] timeout/config access for the selected Given
- [ ] Build timeout helpers under `cucumber_tests/ha/support/timeouts/`.
- [ ] Derive wait deadlines from the actual checked-in config values:
- [ ] failover waits based on `ha.lease_ttl_ms` plus defined slack
- [ ] recovery waits based on the relevant process timeout plus defined slack
- [ ] Do not embed unexplained magic-number deadlines directly in the feature file or the step code.

### Phase 9: Implement the first feature file and step definitions
- [ ] Create `cucumber_tests/ha/features/primary_crash_rejoin/primary_crash_rejoin.feature`.
- [ ] Write the scenario so it says, in operator terms, that after waiting for the configured HA lease TTL plus slack a different node becomes the only primary.
- [ ] Do not phrase the feature in terms of an implementation detail such as “after leader lease expires”.
- [ ] Implement step definitions under `cucumber_tests/ha/support/...` for this scenario only:
- [ ] start `three_node_plain`
- [ ] wait for exactly one stable primary
- [ ] kill the current primary container
- [ ] wait for a different single primary using config-derived deadline
- [ ] write a proof row through the new primary
- [ ] start the killed node container again
- [ ] wait for the restarted node to rejoin as a replica using config-derived deadline
- [ ] verify the proof row is visible from the restarted node
- [ ] verify there is exactly one primary again
- [ ] Keep SQL proofing minimal and readable: one proof table, one post-failover row, one replica-read validation.

### Phase 10: Make independence from the old harness explicit and enforce it
- [ ] Ensure the new framework does not import or call anything under:
- [ ] `tests/*`
- [ ] `src/test_harness/ha_e2e/`
- [ ] any legacy custom HTTP helper or observer logic
- [ ] Add a repo-wide verification step, for example `rg -n "(tests/ha|src/test_harness/ha_e2e)" cucumber_tests/ha Cargo.toml`, and confirm no forbidden legacy harness dependencies remain in the new framework.
- [ ] Keep the old harness untouched by the new framework implementation.
- [ ] Make the separation goal explicit in code comments where it clarifies intent: the new framework is independent by design because the legacy HA harness is intended for later deletion.

### Phase 11: Docs and developer entrypoints
- [ ] Update docs with the new cucumber HA test entrypoint, layout, and operator-observer model.
- [ ] Minimum expected docs touch:
- [ ] `docs/src/how-to/run-tests.md`
- [ ] any relevant operator/docker docs if they describe the HA test harness layout or invocation model in a way that would now be stale
- [ ] Add a stable make entrypoint for the cucumber HA suite so it is easy to run the first feature and later the whole suite.
- [ ] Document how to run:
- [ ] the first feature directly
- [ ] the cucumber HA suite through the make entrypoint
- [ ] Ensure docs do not imply the new framework reuses the old HA harness.

### Phase 12: Verification and closeout
- [ ] Run targeted execution of the new first feature and preserve the resulting run artifacts.
- [ ] Run `make check`
- [ ] Run `make test`
- [ ] Run `make test-long`
- [ ] Run `make lint`
- [ ] Run repo-wide stale verification to confirm the new framework did not accidentally import legacy test harness code.
- [ ] Update this task file with completed checkboxes only after the work and gates are actually done.
- [ ] Only after all required work is complete, set `<passes>true</passes>`
- [ ] Run `/bin/bash .ralph/task_switch.sh`
- [ ] Commit all required files, including `.ralph/` updates, with a task-finished commit message that includes verification evidence
- [ ] Push with `git push`

TO BE VERIFIED
- [ ] Run targeted execution of `primary_crash_rejoin`.
- [ ] If the result is a harness failure, keep fixing the harness until the run reaches a trustworthy outcome.
- [ ] If the result is a trustworthy HA or product failure, create a bug task immediately with add-bug and add `<blocked_by>` tags for all four tasks in this story.
