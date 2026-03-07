# Container Deployment

This is the primary operator path for the repository. The checked-in Docker assets are meant to be the canonical first deployment shape, not an afterthought.

## What ships

The repo now includes:

- `docker/Dockerfile.prod`: minimal runtime image for `pgtuskmaster` nodes on top of `postgres:16-bookworm`
- `docker/Dockerfile.dev`: development image with Rust, cargo, Node, and other local iteration tooling
- `docker/compose/docker-compose.single.yml`: `etcd` + `node-a`
- `docker/compose/docker-compose.cluster.yml`: `etcd` + `node-a` + `node-b` + `node-c`
- `docker/configs/**`: tracked runtime TOML and PostgreSQL config inputs
- `docker/secrets/*.example`: placeholder secret files only

## Prepare the local environment

1. Create `.env.docker` from `.env.docker.example`.
2. Replace the example secret file paths with local non-example files.
3. Write local values into those files. For the checked-in lab stacks, readable non-empty secrets are sufficient; for anything beyond the lab, generate strong real passwords.

The env file controls:

- `PGTUSKMASTER_IMAGE`
- `ETCD_IMAGE`
- `PGTM_SECRET_SUPERUSER_FILE`
- `PGTM_SECRET_REPLICATOR_FILE`
- `PGTM_SECRET_REWINDER_FILE`
- the single-node published ports
- the per-node published ports for the three-node cluster

## Bring up the single-node stack

```console
make docker-up
```

Useful checks after startup:

```console
curl --silent --show-error http://127.0.0.1:${PGTM_SINGLE_API_PORT}/ha/state
curl --silent --show-error http://127.0.0.1:${PGTM_SINGLE_API_PORT}/debug/verbose
```

Teardown:

```console
make docker-down
```

## Bring up the three-node cluster

```console
make docker-up-cluster
```

Validation:

```console
make docker-smoke-cluster
```

Teardown:

```console
make docker-down-cluster
```

## Security posture of the starter stacks

The checked-in lab stacks prioritize operability over hardening:

- API TLS is disabled
- API token auth is disabled
- debug routes are enabled
- PostgreSQL host and replication access inside the Compose bridge network use trust-based `pg_hba` rules

That choice is deliberate because `api.security.auth.role_tokens` are plain strings in the runtime schema today. For the quick-start path, it is safer to keep the lab clearly scoped as local-only than to pretend the checked-in config files are a secret management system.

The password files are still mounted and validated because the runtime contract is file-backed. The starter stack keeps that contract visible now so hardening later does not require a deployment shape change.

For a hardened deployment, keep the same file-backed password pattern but generate a protected runtime TOML that enables:

- API TLS
- API role tokens
- any private key material you need through the same secret-file discipline

## Compose and Makefile helpers

The repo-owned commands are:

- `make docker-compose-config`
- `make docker-up`
- `make docker-down`
- `make docker-up-cluster`
- `make docker-down-cluster`
- `make docker-smoke-single`
- `make docker-smoke-cluster`

`make test-long` runs the Compose config validation and both smoke flows after the ultra-long HA nextest profile, so container deployment drift is part of the mandatory long gate now.
