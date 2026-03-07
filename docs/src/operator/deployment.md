# Deployment and Topology

The default deployment story is now container-first:

- build the production node image from `docker/Dockerfile.prod`
- use the checked-in Compose stacks under `docker/compose/`
- mount runtime TOML and managed PostgreSQL config sources through Compose `configs`
- mount credentials through Compose `secrets`

If you are learning the system or validating a change, start with [Container Deployment](./container-deployment.md). Manual host-native deployment is still possible, but it is the advanced path now.

## Starter topologies

### Single-node lab

`docker/compose/docker-compose.single.yml` starts:

- one `etcd` service
- one node service named `node-a`
- one published API/debug port
- one published PostgreSQL port

This is the shortest supported path to a coherent `/ha/state`.

### Three-node lab cluster

`docker/compose/docker-compose.cluster.yml` starts:

- one `etcd` service
- three node services: `node-a`, `node-b`, `node-c`
- one published API/debug port per node
- one published PostgreSQL port per node

The cluster stack is the default day-1 topology for proving inter-node reachability, replica bootstrap, and DCS coordination behavior inside containers.

## Network exposure rule

The checked-in Compose files intentionally keep exposure narrow:

- etcd stays internal to the Compose bridge network
- the API and debug routes share the same published port on each node
- PostgreSQL gets one published client port per node
- there is no separate published debug port

That matches the runtime contract: `debug.enabled = true` adds `/debug/*` routes on the API listener instead of creating a second socket.

## Secrets and config ownership

The container deployment surface is split deliberately:

- `docker/configs/**`: tracked runtime TOMLs and managed `pg_hba` / `pg_ident` sources
- `docker/secrets/*.example`: tracked placeholders only
- `.env.docker`: local path mapping for the real secret files that Compose mounts

Do not push real password files into the repo. The runtime already supports file-backed secrets; use that instead of environment-variable secrets for the PostgreSQL roles.

## Manual deployment is secondary

If you are not using containers, you still need to satisfy the same runtime contract:

- real PostgreSQL 16 binaries at absolute paths
- a reachable etcd deployment
- readable secret files
- deliberate API exposure and auth/TLS choices

Use the host-native path only when you are intentionally translating the checked-in container contract into another deployment system.
