# First Run

Start with the single-node Compose stack. It gives you one `etcd` service plus one `pgtuskmaster` node named `node-a`, wired with tracked configs and file-backed Docker secrets.

## 1. Validate your local Compose render

```console
docker compose --env-file .env.docker -f docker/compose/docker-compose.single.yml config
docker compose --env-file .env.docker -f docker/compose/docker-compose.cluster.yml config
```

This fails closed if:

- `.env.docker` is missing or incomplete
- a required secret file path does not exist
- the Compose config, config mounts, or secret mounts are malformed

If you are changing the repo-owned Docker assets themselves, `make docker-compose-config` still exists as the contributor-facing gate that validates the checked-in example env file.

What this step proves is narrower than "the cluster works". It proves that your local environment can resolve every file reference, environment variable, secret path, and config mount that the supported lab topology depends on. That matters because a later runtime failure is much easier to interpret once you know the shape of the deployment is valid.

If this render step fails, read the error literally. It is usually pointing at one of four concrete problems: the environment file is missing, a secret path is wrong, a tracked config path was renamed, or your host Docker installation is not reading the file tree you think it is reading. None of those are HA bugs, and trying to diagnose them as HA behavior only adds confusion.

## 2. Bring up the single-node stack

```console
make docker-up
```

That target builds `docker/Dockerfile.prod`, starts `etcd`, and starts `node-a` with:

- `docker/configs/single/node-a/runtime.toml`
- `docker/configs/common/pg_hba.conf`
- `docker/configs/common/pg_ident.conf`
- Docker secrets mounted from the three secret files referenced in `.env.docker`

The stack publishes only two host ports:

- `PGTM_SINGLE_API_PORT` for both `/ha/state` and `/debug/*`
- `PGTM_SINGLE_PG_PORT` for PostgreSQL client traffic

The helper target builds the production image and then starts the single-node Compose project in detached mode. Under the default lab wiring, `etcd` must become healthy before `node-a` is considered ready to start. That dependency is important because the first runtime loop needs shared coordination available before the node can settle into a stable HA interpretation.

Expect intermediate states after `make docker-up`. Container creation finishing is not the same as the node being operational. The runtime still needs time to observe PostgreSQL reachability, establish DCS trust, choose an initial path, and publish coherent state through the API. During this window, a container can be alive while `/ha/state` is still converging toward something meaningful.

If the stack fails here, separate the symptom carefully:

- Build failure usually points at image or toolchain prerequisites rather than cluster logic.
- `etcd` healthcheck failure points at the coordination service path, so the HA node cannot establish trusted coordination yet.
- `node-a` container restarts often point at config, secret, or process startup problems before any real HA decision has settled.
- Missing published ports mean the host-facing proof path is broken even if the internal network exists.

## 3. Confirm the API is live

```console
curl --silent --show-error http://127.0.0.1:${PGTM_SINGLE_API_PORT}/ha/state
curl --silent --show-error http://127.0.0.1:${PGTM_SINGLE_API_PORT}/debug/verbose
```

The starter stack intentionally enables debug routes and disables API auth so the lab flow is easy to inspect. That is a quick-start convenience, not a production hardening posture.

Treat these checks as an observation step, not just a connectivity step. A reachable `/ha/state` proves that the API listener is exposed on the published host port. A reachable `/debug/verbose` proves that the debug routes are riding that same listener in the checked-in lab path rather than hiding behind a second unpublished socket. Reading the payloads also gives you the first real evidence of how the node currently sees itself: member identity, HA phase, trust posture, leader information, and decision tick.

If `/ha/state` responds but looks odd, do not immediately assume failure. On a fresh single-node startup, transient waiting phases can appear before the node settles. What matters is whether the state evolves toward a coherent, self-consistent picture. If the response is structurally valid but never converges, that is the point to move into the validation and troubleshooting guidance rather than repeating `curl` without interpretation.

If `/debug/verbose` fails while `/ha/state` succeeds, the problem is not a total API outage. It is a mismatch between the expected lab debug exposure and what is currently configured. That distinction matters later when you harden the API, because the quick-start default is intentionally more permissive than a production posture.

## 4. Confirm the published PostgreSQL path

The same Compose file also publishes `PGTM_SINGLE_PG_PORT` from the container's PostgreSQL port. This check matters because a healthy HA control loop is not enough if clients still cannot reach the database endpoint that the lab claims to expose. Use whichever host-side probe is natural in your environment, or rely on the smoke target in the next chapter if you want the repository-owned proof path.

If the API is healthy but the PostgreSQL port is not reachable, interpret that as a deployment-surface mismatch first. The node may still be running internally while the host publication, PostgreSQL startup, or container-network assumptions are wrong. That is a different problem from a bad HA decision, and the distinction becomes important when you scale from single-node proof to cluster experiments.

## 5. Tear the lab stack down when you are done

```console
make docker-down
```

`make docker-down` removes the containers plus their named volumes. If you want to keep the single-node data directory around between restarts, use raw `docker compose stop/start` instead of the destructive helper.

That destructive behavior is intentional in the quick-start path. The default expectation is that you can reproduce a clean first-run proof repeatedly, not that the helper preserves a carefully curated lab state forever. If you want to inspect a stateful run more slowly, switch to raw Compose lifecycle commands after you understand the consequences.

## What this stage should leave you knowing

By the end of first run, you should be able to answer four concrete questions:

- Did the tracked deployment render without missing config or secret references?
- Did the repository-owned image and Compose project build and start successfully?
- Did the host-published API surface expose both HA and debug routes as expected?
- Did the host-published PostgreSQL surface become reachable as part of the same startup?

If the answer to any of those is "not yet", stay in the quick-start path until the failure is clearly classified. The Operator Guide assumes you already have a trustworthy first-run baseline.
