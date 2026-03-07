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

## 3. Confirm the API is live

```console
curl --silent --show-error http://127.0.0.1:${PGTM_SINGLE_API_PORT}/ha/state
curl --silent --show-error http://127.0.0.1:${PGTM_SINGLE_API_PORT}/debug/verbose
```

The starter stack intentionally enables debug routes and disables API auth so the lab flow is easy to inspect. That is a quick-start convenience, not a production hardening posture.

## 4. Tear the lab stack down when you are done

```console
make docker-down
```

`make docker-down` removes the containers plus their named volumes. If you want to keep the single-node data directory around between restarts, use raw `docker compose stop/start` instead of the destructive helper.
