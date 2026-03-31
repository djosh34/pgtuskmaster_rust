## Task: Smooth The Local Docker Cluster Startup Experience <status>not_started</status> <passes>false</passes>

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
- [ ] The local Docker cluster startup path is reviewed across `docker/compose/docker-compose.cluster.yml`, `.env.docker.example`, `tools/docker/common.sh`, and `tools/docker/smoke-cluster.sh`
- [ ] A stable operator-facing command or script exists for bringing up a persistent local cluster without immediate teardown
- [ ] Successful startup output includes the API URL, debug URL, and PostgreSQL endpoint for each node
- [ ] Successful startup output includes current cluster topology, including which node is primary and which nodes are replicas
- [ ] There is a documented or scripted way to inspect/status the same running stack later without rediscovering hidden temp paths or Compose arguments
- [ ] There is a documented or scripted teardown path for the same running stack
- [ ] The first-run build path is either improved directly or explained clearly enough that the delay is expected and not confusing
- [ ] Relevant docs or command help text are updated so a new operator can start and connect to the cluster without reading helper-script internals
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
