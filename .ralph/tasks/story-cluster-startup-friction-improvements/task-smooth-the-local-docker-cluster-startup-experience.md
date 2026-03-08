## Task: Smooth The Local Docker Cluster Startup Experience <status>completed</status> <passes>true</passes>

<priority>low</priority>

<description>
**Goal:** Reduce operator friction when bringing up the local Docker-based cluster for development, smoke testing, and manual debugging. The higher-order goal is to make the first successful cluster boot feel intentional and self-explanatory rather than requiring the operator to reverse-engineer the smoke scripts or manually inspect generated env files to discover ports, leader state, and connection details.

**Scope:**
- Review the existing local Docker workflow centered on `docker/compose/docker-compose.cluster.yml`, `.env.docker.example`, and `tools/docker/common.sh` plus `tools/docker/smoke-cluster.sh`.
- Design and implement one stable operator-facing cluster-up flow that leaves the cluster running and prints the exact API URLs, debug URLs, PostgreSQL connection endpoints, and current leader/replica state after readiness succeeds.
- Reduce endpoint-discovery friction. Either keep stable well-known local ports by default, or if dynamic free-port allocation remains necessary, make the chosen values obvious in command output and easy to retrieve later.
- Reduce first-run friction where practical. Audit whether image build caching, a documented prebuild step, or a dedicated image-build helper can shorten or clarify the slow initial startup path.
- Ensure the resulting flow includes a matching inspect/status path and a teardown path so operators can manage the same stack without reconstructing the Compose invocation by hand.
- Update the relevant docs or command help text so a new operator can start the cluster and connect to it without reading internal helper scripts.

**Context from research:**
- The current cluster stack is already functional and healthy under `docker/compose/docker-compose.cluster.yml`.
- `tools/docker/smoke-cluster.sh` already proves the expected readiness checks, but it is oriented toward smoke validation and temporary runtime state rather than a durable operator workflow.
- `tools/docker/common.sh` contains useful reusable pieces such as free-port allocation and readiness helpers, but the current path still leaves the operator to inspect the generated env file for connection details.
- In a live run, the cluster reached a healthy topology with one primary and two replicas, and debug endpoints were immediately usable once the ports were known.
- The main friction observed was:
- first-run Docker image build latency before the cluster became usable
- lack of a single persistent `cluster-up` command that clearly announces what is running
- lack of immediate printed connection details for API, debug, and PostgreSQL endpoints
- reliance on generated temp env/secrets paths for discovery and later management commands

**Expected outcome:**
- Operators have one obvious local command or script to start a persistent cluster and get all connection details immediately.
- The startup output clearly states API URLs, debug URLs, PostgreSQL endpoints, cluster topology, and where any generated env/secrets state lives.
- The local cluster workflow feels smoother on first boot and routine to repeat afterward.
- The repo no longer requires operators to read smoke-test internals just to connect to a running local cluster.

</description>

<acceptance_criteria>
- [x] The local Docker cluster startup path is reviewed across `docker/compose/docker-compose.cluster.yml`, `.env.docker.example`, `tools/docker/common.sh`, and `tools/docker/smoke-cluster.sh`
- [x] A stable operator-facing command or script exists for bringing up a persistent local cluster without immediate teardown
- [x] Successful startup output includes the API URL, debug URL, and PostgreSQL endpoint for each node
- [x] Successful startup output includes current cluster topology, including which node is primary and which nodes are replicas
- [x] There is a documented or scripted way to inspect/status the same running stack later without rediscovering hidden temp paths or Compose arguments
- [x] There is a documented or scripted teardown path for the same running stack
- [x] The first-run build path is either improved directly or explained clearly enough that the delay is expected and not confusing
- [x] Relevant docs or command help text are updated so a new operator can start and connect to the cluster without reading helper-script internals
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Implementation plan

1. Review and reuse the existing Docker cluster building blocks rather than inventing a second workflow.
   - Keep `docker/compose/docker-compose.cluster.yml` as the persistent stack definition.
   - Keep `.env.docker.example` as the default stable-port example for operators.
   - Reuse readiness and Compose helpers from `tools/docker/common.sh` where possible so the smoke and operator flows converge instead of drifting.
   - Treat `tools/docker/smoke-cluster.sh` as the main source for proven readiness sequencing, then refactor shared pieces out of it instead of duplicating its polling logic.

2. Introduce one operator-facing cluster lifecycle script under `tools/docker/` and make it the canonical local cluster entry point.
   - Add a new script that accepts explicit subcommands such as `up`, `status`, and `down`.
   - Keep the stack persistent for `up`; do not use temp directories or trap-based teardown in the operator path.
   - Make the default operator contract match the existing Makefile variables: use `.env.docker` when present, and only fall back to `.env.docker.example` if the script intentionally supports that as a no-copy local default.
   - Keep the project name aligned with `DOCKER_CLUSTER_PROJECT` so later `status` and `down` commands target the same stack without inventing a second naming contract.
   - Support explicit overrides such as `--env-file` and possibly `--project-name`, and always print the effective values so later management commands remain obvious.

3. Avoid introducing persistent state files unless execution proves they are strictly necessary.
   - Prefer a stateless design where `up`, `status`, and `down` all derive their behavior from the same stable compose file, env file, and project-name contract.
   - Do not write `.ralph` state just to remember Compose arguments if the script can compute them deterministically.
   - If an execution-time edge case forces persisted state, keep it repo-local and document exactly why the stateless contract was insufficient.

4. Refactor `tools/docker/common.sh` so both smoke and operator flows share the same primitives.
   - Add helper(s) to read cluster port values from an env file without repeating `grep | cut` fragments in every script.
   - Add helper(s) to query HA state and summarize each node’s role, leader, and member count from `/ha/state`.
   - Add helper(s) to print a consistent operator summary including API URL, debug URL, and PostgreSQL endpoint for each node.
   - Add helper(s) for readiness sequencing that can be reused by the smoke script and the new operator script.
   - Keep error handling explicit and fail-fast; do not introduce ignored errors, `|| true` shortcuts in normal control flow, or unchecked parsing.
   - Where the current helpers already swallow errors, fix that as part of execution rather than copying the same pattern into the new operator script.

5. Preserve the existing smoke test while making it consume the shared helpers.
   - Update `tools/docker/smoke-cluster.sh` to reuse the new env parsing and topology/reporting helpers.
   - Keep smoke-specific behavior that still needs ephemeral temp roots, random free ports, and automatic teardown on exit.
   - Ensure the smoke script continues validating one primary plus two replicas, all APIs reachable, PostgreSQL ports reachable, and SQL readiness on each node.

6. Implement the `up` flow to make first successful boot self-explanatory.
   - Start the cluster with `docker compose ... up -d --build` using the stable operator project name and env file.
   - Wait for `/ha/state`, `/debug/verbose`, published PostgreSQL ports, SQL readiness, and a 1-primary/2-replica topology before declaring success.
   - Print an explicit note before the build/start phase that first-run image building may take a while, so slow startup reads as expected build latency rather than a hang.
   - After readiness succeeds, print a concise summary table or grouped output covering:
     - compose project name
     - compose file and env file in use
     - API URL for each node
     - debug URL for each node
     - PostgreSQL endpoint for each node
     - current leader
     - each node’s effective role (primary or replica)
     - current member count and HA phase/decision summary if practical

7. Implement a durable inspect/status path for the running stack.
   - `status` should use the same project/env selection contract as `up`.
   - It should report whether the Compose services are running and then print the same endpoint/topology summary without rebuilding or recreating the stack.
   - If the stack is not up, return a clear non-zero error telling the operator which command to run next.

8. Implement a teardown path that matches the same stack contract.
   - `down` should call `docker compose ... down -v --remove-orphans` against the same project/env selection.
   - Print which stack is being torn down before executing the stop.
   - If a persistent state file is used for the operator workflow, remove or refresh it during teardown so stale metadata does not linger.

9. Wire the new script into the existing developer command surface instead of forcing operators to memorize raw Compose commands.
   - Change `make docker-up-cluster` to call the new script’s `up` subcommand.
   - Change `make docker-down-cluster` to call the new script’s `down` subcommand.
   - Add a new make target for cluster status so the inspect path is discoverable from `make` as well as directly from the script.
   - Keep existing smoke targets intact and separate from the persistent operator flow.

10. Update user-facing docs and command examples so the new flow is the obvious path.
   - Update `README.md` local cluster instructions to point to the new stable cluster command and include the printed endpoints/topology behavior.
   - Update `docs/src/tutorial/first-ha-cluster.md` to use the new command instead of raw `docker compose ... up -d --build`.
   - Add or update a how-to/reference page if needed so operators can run `up`, `status`, and `down` without reading shell internals.
   - Explain stable default ports from `.env.docker.example` and explain the first-run build delay clearly.
   - Because the repo instructions mention an `update-docs` skill but no such skill is available in this session, handle the docs update directly in the implementation turn unless a dedicated docs skill becomes available then.

11. Verification plan for the execution turn.
   - Run targeted manual checks around the new cluster lifecycle script first: `up`, `status`, and `down`, confirming the printed endpoints and topology are accurate against live `/ha/state` responses.
   - Verify both the default env-file path and any explicit `--env-file` override path so the contract is not only documented but exercised.
   - Run the required full gates exactly as requested once implementation is complete:
     - `make check`
     - `make test`
     - `make test-long`
     - `make lint`
   - Confirm docs reflect the new flow and do not leave stale raw-Compose instructions in the main operator path.
   - Only after all gates pass should the task file be marked with `<passes>true</passes>` and the Ralph task-switch / commit / push closeout happen.

NOW EXECUTE
