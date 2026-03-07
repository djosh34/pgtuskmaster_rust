# Prerequisites

The default first-run path is the checked-in Docker Compose stack. You do not need a host-level PostgreSQL or etcd install for that path, but you do need Docker plus three readable local secret files.

Required on the host:

- Docker Engine with the `docker compose` v2 plugin
- enough disk space for one or three PostgreSQL data volumes, depending on which stack you plan to start
- free local ports for the API and PostgreSQL mappings listed in `.env.docker`
- a writable checkout so the helper targets can build images and write temporary smoke artifacts

Before the first Compose launch:

1. Create `.env.docker` from the tracked `.env.docker.example` file at the repository root.
2. Replace the example secret paths with paths to local secret files.
3. Write a non-empty value into each referenced file:
   - `PGTM_SECRET_SUPERUSER_FILE`
   - `PGTM_SECRET_REPLICATOR_FILE`
   - `PGTM_SECRET_REWINDER_FILE`
4. Keep those files out of git. The repository only tracks `docker/secrets/*.example` placeholders.

For the checked-in local lab, those files primarily satisfy the runtime's file-backed secret contract while PostgreSQL network auth stays trust-based inside the private Compose bridge. For a hardened deployment, replace the placeholder values with strong real passwords and lock down `pg_hba`.

The checked-in Compose assets use:

- `PGTUSKMASTER_IMAGE` for the node image tag
- `ETCD_IMAGE` for the etcd image tag
- `PGTM_SINGLE_*` ports for the single-node stack
- `PGTM_CLUSTER_NODE_*_*` ports for the three-node cluster stack

If you want the default quick-start validation path to work before you do a full `up`, run:

```console
docker compose --env-file .env.docker -f docker/compose/docker-compose.single.yml config
docker compose --env-file .env.docker -f docker/compose/docker-compose.cluster.yml config
```

Manual or non-container deployment is still supported as an advanced path, but it is no longer the default first-run workflow. Use the Compose flow first unless you are deliberately building a host-native installation.
