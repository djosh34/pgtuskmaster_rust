# Deployment and Topology

The default deployment story is container-first:

- build the production node image from `docker/Dockerfile.prod`
- use the checked-in Compose stacks under `docker/compose/`
- mount runtime TOML and managed PostgreSQL config sources through Compose `configs`
- mount credentials through Compose `secrets`

That container-first stance is not marketing convenience. It gives the project one canonical deployment shape that the docs, tests, smoke flows, and quick-start path can all refer to without ambiguity.

## What the checked-in topologies actually prove

### Single-node lab

`docker/compose/docker-compose.single.yml` starts:

- one `etcd` service
- one node service named `node-a`
- one published API/debug port
- one published PostgreSQL port

This topology proves the basics:

- the production image builds
- the runtime config parses
- startup planning can initialize and expose a coherent state
- the host can reach the API and PostgreSQL listeners the docs describe

It does not prove multi-node lease behavior, switchover sequencing, or failover timing under stress.

### Three-node lab cluster

`docker/compose/docker-compose.cluster.yml` starts:

- one `etcd` service
- three node services: `node-a`, `node-b`, `node-c`
- one published API/debug port per node
- one published PostgreSQL port per node

This topology proves more:

- inter-node reachability for replication-oriented flows
- DCS coordination across multiple members
- richer observability and smoke coverage across several nodes

It still does not prove everything about a hardened production deployment. The lab topology is intentionally useful, not complete.

## Environment assumptions you should keep explicit

The checked-in deployment assumes:

- Docker is the process supervisor for the first-run path
- etcd is internal to the Compose bridge network
- API and debug routes share one listener per node
- PostgreSQL gets one published client port per node
- runtime configuration and PostgreSQL auth files are mounted, not generated ad hoc inside the container
- secrets arrive as files, not as plaintext environment variables

Those assumptions keep the lab honest. If you later translate the deployment into Kubernetes, Nomad, systemd, or another host-native path, you still need to preserve the same semantic contract or update the docs and tests accordingly.

## Network exposure decisions

The Compose files intentionally keep exposure narrow:

- etcd is not published to the host
- the API and debug routes share one published port on each node
- PostgreSQL gets one published client port per node
- there is no second published debug socket

That matches the runtime behavior. When `debug.enabled = true`, debug routes are extra HTTP paths on the API listener, not a distinct network service. This matters during review and hardening. If you see a deployment diagram with a separate debug port, it is describing a topology the current runtime does not implement.

## Manual deployment is a translation exercise, not a new contract

Manual or host-native deployment is still possible, but it is secondary. If you do not use containers, you still must provide:

- PostgreSQL 16 binaries at the absolute paths the runtime expects
- a reachable etcd deployment
- readable secret files for PostgreSQL roles and any API TLS material
- deliberate API bind, TLS, and token settings
- stable storage and log paths that match the config

The safest way to do that translation is to treat the checked-in container assets as the contract and ask how your target platform satisfies each part of it. If you skip that comparison, it is easy to produce a deployment that "starts" but no longer matches the semantics the docs and tests assume.

## Operational caveats about the starter stacks

The starter stacks are intentionally permissive in ways that should not be copied blindly:

- API TLS is disabled
- API token auth is disabled
- debug routes are enabled
- PostgreSQL host and replication access inside the private bridge use trust-based `pg_hba`

These choices exist so the first-run path is inspectable and reproducible. They are not a claim that production should look the same. The correct way to read the lab topology is "this proves the runtime wiring," not "this is the finished security posture."

## How to decide when the lab is no longer enough

Move beyond the checked-in lab when you need to answer questions like:

- what cert distribution path will back API TLS
- where role tokens or rendered runtime TOML will be stored securely
- how PostgreSQL client access and replication policy should be segmented
- which hostnames or service names the runtime should advertise across real networks

At that point the lab has done its job. It gave you a truthful baseline. Your next job is to preserve the same behavior contract while replacing its intentionally local-only assumptions.
