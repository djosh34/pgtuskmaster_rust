## Task: QA The Shipped `docker compose up` Operator Walkthrough <status>not_started</status> <passes>false</passes>

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
- [ ] Add a QA evidence directory at `.ralph/evidence/docker-compose-qa-walkthrough/` with a report that answers:
  - how `pgtm` was installed
  - whether installing `pgtm` was easy or overly complicated
  - was it easy to figure out from the shipped docs/configs without searching or extra work
  - how should the operator experience be improved
  - exactly what commands were run and what each one verified
- [ ] Install `pgtm` on the host first and validate whether the repo makes that installation path obvious and truthful.
- [ ] Start the shipped stack with `docker compose up -d --build` and prove whether that documented entrypoint is truthful from a clean operator workflow.
- [ ] Validate the shipped host-side operator config `docker/pgtm.toml` against the running stack.
- [ ] Verify the documented `pgtm` feature set against the Docker stack: `status`, `--json`, `status -v`, `status --watch`, `primary`, `primary --tls`, `replicas`, `switchover request`, targeted `switchover request --switchover-to ...`, and `switchover clear`.
- [ ] Kill one node container, run `pgtm -c ...` checks while it is down, restart it, and verify the cluster view converges again.
- [ ] Exercise negative cases that should fail, including wrong auth material and wrong PostgreSQL password, and document whether those failures are clear and correct.
- [ ] Fix any doc/config/compose issues needed to make the shipped experience truthful, or create bug tasks for each remaining gap using the `$add-bug` skill.
- [ ] The final evidence report explicitly lists every bug task created during this QA pass.
- [ ] Tear the Docker stack down cleanly after the QA pass.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
