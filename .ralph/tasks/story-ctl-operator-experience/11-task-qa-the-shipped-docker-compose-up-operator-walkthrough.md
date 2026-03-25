## Task: QA The Shipped `docker compose up` Operator Walkthrough <status>completed</status> <passes>true</passes>

<priority>low</priority>

<description>
**Goal:** Act like a QA engineer and validate that the shipped local HA stack really works from a fresh host-side operator workflow. The task must start from the documented `docker compose up -d --build` path from the repo root, verify that the shipped `docker/pgtm.toml` operator config and documented commands actually work end-to-end, and fix any config or docs gaps needed to make that experience truthful.

**Higher-order goal:** The repository currently claims a simple Docker-based local cluster experience in `README.md` and the operator docs. That claim is only valuable if a new operator can discover the right steps without implementation spelunking, bring the stack up, use `pgtm -c docker/pgtm.toml` from the host, observe failures and recovery correctly, and understand what worked or failed from one evidence trail. This task is a product-experience QA pass, not just a code/config tweak.

**Decisions already made from user discussion:**
- Treat this as a QA-engineer task, not as an implementation-first feature task.
- The task must begin by installing `pgtm` on the host and documenting that experience before validating Docker workflows.
- The task must begin with the documented Docker path: `docker compose up -d --build`.
- The first pass must try to follow the README and docs as written before falling back to code/config searching. Record whether it was easy to find the right steps without extra digging.
- If the documented flow does not "just work", alter the relevant config files and docs so the shipped experience becomes truthful.
- The task must explicitly test the restart/rejoin operator flow by killing one container, running `pgtm -c ...` checks while it is down, then starting it again and verifying convergence.
- The task must explicitly test negative cases that should fail, including wrong credentials or wrong auth material.
- Every functional gap or misleading doc/config behavior discovered during this pass must become a bug task created with the `$add-bug` skill.
- The task must document the full operator experience in a new evidence directory at `.ralph/evidence/docker-compose-qa-walkthrough/`.

**Scope:**
- Use the shipped Docker HA stack in `docker/compose.yml` as the only environment under test.
- Use the shipped host-side operator config `docker/pgtm.toml` as the main `pgtm` entrypoint under test.
- Use the documented operator workflows in:
  - `README.md`
  - `docs/src/tutorial/first-ha-cluster.md`
  - `docs/src/how-to/check-cluster-health.md`
  - `docs/src/how-to/perform-switchover.md`
- If the experience is broken or misleading, fix the relevant shipped files. Likely candidates include:
  - `docker/compose.yml`
  - `docker/docker-compose.node.yml`
  - `docker/pgtm.toml`
  - `docker/node-a.toml`
  - `docker/node-b.toml`
  - `docker/node-c.toml`
  - `README.md`
  - `docs/src/tutorial/first-ha-cluster.md`
  - `docs/src/how-to/check-cluster-health.md`
  - `docs/src/how-to/perform-switchover.md`
- Capture all evidence in `.ralph/evidence/docker-compose-qa-walkthrough/`, including exact commands, observed outputs or summaries, failures, fixes, and final verification notes.
- Create bug tasks with `$add-bug` for anything that still does not work, any broken negative-path handling, any misleading config/doc behavior, or any rough operator experience that should be tracked separately.

**Out of scope:**
- Do not broaden this into a new test framework or a large acceptance-suite rewrite.
- Do not silently leave known issues in prose only. If something is broken or misleading and is not fully fixed in this task, add a bug task for it.
- Do not rely on private chat context when writing the evidence report or bug tasks. Everything needed must be derivable from repo files plus the evidence you produce.

**Context from research:**
- `README.md` currently tells operators to run:

```bash
docker compose up -d --build
docker compose ps
pgtm -c docker/pgtm.toml status
docker compose down
```

- `docs/src/tutorial/first-ha-cluster.md` says the local tutorial should work with:

```bash
docker compose up -d --build
until pgtm -c docker/pgtm.toml status >/dev/null 2>&1; do
  sleep 1
done
pgtm -c docker/pgtm.toml status
```

- `docs/src/how-to/check-cluster-health.md` defines the documented cluster-health operator surface that must be validated:

```bash
pgtm -c config.toml status
pgtm -c config.toml --json
pgtm -c config.toml status -v
pgtm -c config.toml status --watch
pgtm -c config.toml primary
pgtm -c config.toml primary --tls
pgtm -c config.toml replicas
```

- `docs/src/how-to/perform-switchover.md` defines additional documented operator flows that must be validated against the Docker stack:

```bash
pgtm -c config.toml switchover request
pgtm -c config.toml switchover request --switchover-to node-b
pgtm -c config.toml switchover clear
```

- `docker/compose.yml` publishes:
  - HTTPS APIs on `18081`, `18082`, `18083`
  - PostgreSQL on `15001`, `15002`, `15003`
  - services `etcd`, `node-a`, `node-b`, `node-c`
- `docker/pgtm.toml` is the shipped host-side operator config. This is the primary truth source for "`pgtm` works remotely from the host" and must be exercised directly.
- `docker/node-{a,b,c}.toml` define the in-container API, TLS, `pgtm.api`, and PostgreSQL role/password setup. If host-side or restart behavior is broken because these configs are misleading or incomplete, fix them rather than documenting around the breakage.

**Required QA flow:**
1. Start from a clean state and first install `pgtm` on the host using the repo's documented or discoverable install path. Treat this as part of the product experience under test, not as setup noise.
2. Try to discover the correct install and verification path from `README.md` and the docs alone before grepping the codebase. Record whether that was enough, what was unclear, whether installation was a simple `cargo install` or something more complicated, and the first point where extra searching became necessary.
3. After installation succeeds, run the documented compose command exactly as written.
4. Verify the baseline host-side operator workflow against the running stack:

```bash
docker compose ps
pgtm -c docker/pgtm.toml status
pgtm -c docker/pgtm.toml --json
pgtm -c docker/pgtm.toml status -v
timeout 10 pgtm -c docker/pgtm.toml status --watch
pgtm -c docker/pgtm.toml primary
pgtm -c docker/pgtm.toml primary --tls
pgtm -c docker/pgtm.toml replicas
```

5. Verify the documented operator action flows:

```bash
pgtm -c docker/pgtm.toml switchover request --switchover-to node-b
timeout 20 pgtm -c docker/pgtm.toml status --watch
pgtm -c docker/pgtm.toml status -v
pgtm -c docker/pgtm.toml switchover clear
```

6. Verify container-loss and recovery behavior with `pgtm` checks in between. Use one node container, not `etcd`. A concrete minimum flow is:

```bash
docker compose kill node-b
pgtm -c docker/pgtm.toml status -v
docker compose start node-b
timeout 20 pgtm -c docker/pgtm.toml status --watch
pgtm -c docker/pgtm.toml status -v
```

7. Verify that `pgtm -c docker/pgtm.toml` works remotely from the host and remains cluster-oriented and truthful across steady state, switchover, node loss, and node recovery.
8. Verify negative cases that should fail cleanly and explainably. At minimum:
  - create an evidence-local bad operator config that points to a wrong API token and confirm `pgtm status` fails with an auth error
  - create an evidence-local bad operator config or temp override that breaks TLS or points at the wrong port and confirm it fails clearly
  - attempt a PostgreSQL login with the wrong password against the resolved primary target and confirm rejection
9. For each failure, decide whether to fix it immediately in this task or create a bug with `$add-bug`. If multiple failures are the same root cause, group them into one bug.
10. End with a clean, truthful report and a clean environment teardown.

**Evidence requirements:**
- Create `.ralph/evidence/docker-compose-qa-walkthrough/`.
- Write a report there that includes:
  - how `pgtm` was installed on the host
  - whether installing `pgtm` was a simple `cargo install` or something substantially more complicated
  - what went wrong, what was unclear, and what install steps were missing or surprising
  - whether the workflow was easy to discover without searching or extra investigation
  - what was confusing or unnecessarily hard
  - how the experience should be improved
  - a step-by-step command log of exactly what you ran
  - for each command or action, what you were trying to verify and what happened
  - which documented features worked
  - which documented features failed
  - which negative-path checks failed correctly
  - which negative-path checks behaved badly or unclearly
  - which configs or docs you changed, if any, and why
  - which bug tasks you created, with links/paths
- Keep the evidence operator-oriented. The point is to show what a QA engineer experienced from the outside, not to dump implementation archaeology.

**Expected outcome:**
- A fresh operator can figure out how to install `pgtm`, and the repo truthfully documents whether that is simple or more involved.
- A fresh operator can follow the shipped Docker instructions and use `pgtm` from the host without hidden steps.
- The documented operator surface for status, verbose status, JSON status, watch mode, primary resolution, TLS connection output, replica resolution, and switchover is verified against the real Docker stack.
- A stopped node container can be observed as failed/degraded, restarted, and observed as recovered again through `pgtm`.
- Negative auth/connectivity cases fail clearly and predictably instead of hanging or producing misleading output.
- The repo contains a QA evidence report under `.ralph/evidence/docker-compose-qa-walkthrough/`.
- Every remaining issue discovered during this QA pass is tracked via `$add-bug`.

</description>

<acceptance_criteria>
- [x] Add a QA evidence directory at `.ralph/evidence/docker-compose-qa-walkthrough/` with a report that answers:
  - how `pgtm` was installed
  - whether installing `pgtm` was easy or overly complicated
  - was it easy to figure out from the shipped docs/configs without searching or extra work
  - how should the operator experience be improved
  - exactly what commands were run and what each one verified
- [x] Install `pgtm` on the host first and validate whether the repo makes that installation path obvious and truthful.
- [x] Start the shipped stack with `docker compose up -d --build` and prove whether that documented entrypoint is truthful from a clean operator workflow.
- [x] Validate the shipped host-side operator config `docker/pgtm.toml` against the running stack.
- [x] Verify the documented `pgtm` feature set against the Docker stack: `status`, `--json`, `status -v`, `status --watch`, `primary`, `primary --tls`, `replicas`, `switchover request`, targeted `switchover request --switchover-to ...`, and `switchover clear`.
- [x] Kill one node container, run `pgtm -c ...` checks while it is down, restart it, and verify the cluster view converges again.
- [x] Exercise negative cases that should fail, including wrong auth material and wrong PostgreSQL password, and document whether those failures are clear and correct.
- [x] Fix any doc/config/compose issues needed to make the shipped experience truthful, or create bug tasks for each remaining gap using the `$add-bug` skill.
- [x] The final evidence report explicitly lists every bug task created during this QA pass.
- [x] Tear the Docker stack down cleanly after the QA pass.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

<boundary_review>
Smell applied from `improve-code-boundaries`: wrong operator-entrypoint/config truth boundary.

Verified repository facts before execution:
- The repository root already has `docker-compose.yml` that simply includes `docker/compose.yml`, so the plain root command `docker compose up -d --build` is a real candidate for the canonical operator entrypoint.
- `README.md` and the checked-in docs currently still teach `docker compose -f docker/compose.yml ...`, so the task text and the shipped docs are already split on which compose entrypoint is the truth.
- `docker/pgtm.toml` is the shipped host-side operator config, and `docs/examples/docker-cluster-node-a.toml` is currently the same config shape with the same TLS/token material and the same `base_url`.
- `docs/examples/docker-cluster-node-b.toml` and `docs/examples/docker-cluster-node-c.toml` differ only by `base_url`, which means the docs currently carry a duplicated operator-config surface instead of one canonical entrypoint plus the minimal alternates genuinely needed for cross-seed validation.
- The host install path for `pgtm` is discoverable in `Makefile` as `make install-pgtm`, but the README/tutorial/how-to surface currently assumes the binary already exists instead of teaching that install step explicitly.
- No QA evidence directory exists yet for this walkthrough, so execution must create one from scratch and keep it operator-oriented.

Verified reduction / reuse constraints for execution:
- Prefer one canonical host-side operator config surface for the Docker walkthrough. Do not preserve docs-owned duplicates if `docker/pgtm.toml` already serves the same role truthfully.
- Keep per-node alternate seed configs only if they are still needed for real cross-seed verification; otherwise collapse or rewrite docs to point at the shipped canonical config.
- Treat the install surface as part of the same boundary: the docs should not require code spelunking to discover how to put `pgtm` on the host.
- If execution shows the current docs/config split still hides indispensable operator behavior, switch this task back to `TO BE VERIFIED`, document the exact missing boundary here, and stop immediately instead of papering over it with more duplicated examples.

Execution findings that force this task back to design:
- Fixing the shipped Docker runtime configs was enough to make the repo-root `docker compose up -d --build` workflow boot and to make host-side `pgtm -c docker/pgtm.toml status`, `--json`, `status -v`, and `status --watch` work again.
- It was not enough to make `pgtm primary`, `pgtm primary --tls`, and `pgtm replicas` truthful from the host. Those commands currently resolve the single PostgreSQL endpoint stored in DCS member state, which in the Docker stack must remain the internal HA routing target (`node-x:5432`) so replicas and rewinding work.
- The host-side walkthrough, however, needs a distinct operator-routable PostgreSQL target (`127.0.0.1:1500x`, or equivalent host plus `hostaddr` representation) so `psql "$(pgtm -c docker/pgtm.toml primary --tls)"` works from the host.
- The current types cannot represent both truths at once:
  - `src/dcs/state.rs` stores only `postgres_endpoint: PgEndpoint` per member
  - `src/cli/connect.rs` renders that endpoint directly into `primary` and `replicas` with `hostaddr: None`
- This is therefore not just a docs/config cleanup anymore. It is a boundary/type problem between internal HA routing data and operator-facing PostgreSQL routing output.
- Chosen boundary fix for execution:
  - extract one shared PostgreSQL route type, `PgRoute { endpoint, hostaddr }`, so DCS member state and conninfo rendering stop carrying the same address facts in separate shapes
  - replace the single DCS member PostgreSQL route with two explicit roles:
    - `cluster_postgres`: the intra-cluster HA/replication route that rewind, replica follow, and upstream matching continue to use
    - `operator_postgres`: an optional operator-routable route that `pgtm primary` and `pgtm replicas` prefer when present, with fallback to `cluster_postgres` for same-network deployments
  - replace runtime `postgres.network.advertise_port` with typed publish routes:
    - `postgres.network.cluster_advertise`
    - `postgres.network.operator_advertise`
  - keep `docker/pgtm.toml` and the docs examples focused on API seed URL plus TLS/auth material only; per-member PostgreSQL routing truth must come from the node-published DCS payload
  - for the Docker walkthrough, each node runtime config should publish:
    - cluster route: `host = "node-x"`, `port = 5432`
    - operator route: `host = "node-x"`, `port = 1500x`, `hostaddr = "127.0.0.1"`
</boundary_review>

<plan>
- [x] Start from the operator surface only and record discovery friction before fixing anything:
  - read `README.md`, `docs/src/tutorial/first-ha-cluster.md`, `docs/src/how-to/check-cluster-health.md`, and `docs/src/how-to/perform-switchover.md` as written
  - identify the first point where host install or compose usage stops being discoverable without code/config spelunking
  - create `.ralph/evidence/docker-compose-qa-walkthrough/` and begin a command/evidence log immediately
- [x] Verify and document the host install path first:
  - install `pgtm` from the host using the documented-or-discoverable repo path
  - capture whether that was a simple one-command install or a more complicated flow
  - if the docs omit or misstate the install path, plan doc fixes around the real supported command instead of adding workaround prose
- [x] Prove the compose entrypoint from the repo root:
  - start from a clean environment
  - run `docker compose up -d --build` from the repo root first, because the root wrapper file exists and the task requires validating that top-level workflow
  - if the plain root command and the currently documented `-f docker/compose.yml` form differ materially, treat that mismatch as a product issue to fix or bug-track explicitly
- [x] Exercise the shipped host-side operator config and the documented command surface against the live stack:
  - baseline: `docker compose ps`, `pgtm -c docker/pgtm.toml status`, `--json`, `status -v`, `status --watch`, `primary`, `primary --tls`, `replicas`
  - switchover: generic visibility plus targeted `switchover request --switchover-to node-b`, watch output, and `switchover clear`
  - failure/recovery: kill `node-b`, observe degraded truth through `pgtm`, restart `node-b`, and observe convergence back to healthy
- [x] Exercise the required negative paths with evidence-local configs only:
  - copy the shipped operator config into the evidence directory and break auth material to confirm a clear auth failure
  - break TLS or connectivity by using the wrong port or unusable TLS material and confirm the failure is explicit instead of hanging/misleading
  - use the resolved primary target with a wrong PostgreSQL password and confirm server-side rejection is clear
- [x] Fix the operator-truth boundary instead of documenting around it:
  - if `docker/pgtm.toml` is sufficient for the host-side walkthrough, rewrite README/tutorial/how-to guidance to center that one canonical config
  - keep or adjust `docs/examples/docker-cluster-node-{b,c}.toml` only if execution proves they add real operator value for alternate seed-node checks
  - remove or rewrite misleading duplicated instructions rather than preserving parallel doc paths
  - create `$add-bug` tasks for every remaining gap that is not fully fixed in this task, and list those bug paths in the evidence report
- [x] Redesign the PostgreSQL routing boundary before resuming execution:
  - land the shared `PgRoute { endpoint, hostaddr }` type and make `PgConnInfo` reuse it instead of storing endpoint plus hostaddr separately
  - change `DcsMemberState` to publish `cluster_postgres` plus optional `operator_postgres`, and make CLI connection rendering prefer `operator_postgres` with fallback to `cluster_postgres`
  - replace runtime `postgres.network.advertise_port` with `cluster_advertise` and optional `operator_advertise` so the node can publish both truths explicitly
  - update the Docker node configs to publish `node-x:5432` for HA and `node-x:1500x` plus `hostaddr = 127.0.0.1` for operator DSNs
  - update DCS/API/CLI/docs/examples to the new canonical routing model before resuming the QA walkthrough
- [x] Finish with the required validation and task bookkeeping:
  - tear the Docker stack down cleanly after the QA pass
  - run `make check`, `make lint`, `make test`, and `make test-long` in repo order, and tick the acceptance criteria only after they pass
  - set `<passes>true</passes>`, run `/bin/bash .ralph/task_switch.sh`, commit all tracked Ralph/task/doc/code/evidence changes, push, and stop
- [ ] If execution shows the canonical operator surface cannot be made truthful without a different config/doc boundary, switch this task back to `TO BE VERIFIED`, document the exact mismatch here, and stop immediately

NOW EXECUTE
</plan>
