# Verbose context for docs/src/tutorial/single-node-setup.md

Target audience and intent:
- This tutorial should be a gentler first-run path than the existing three-node HA tutorial.
- The repository already carries a dedicated single-node Docker Compose file and a dedicated single-node runtime config.
- There is no code path that introduces a distinct "single-node mode" flag. The node still runs the normal daemon, still talks to etcd, and still uses the normal runtime schema.

What the single-node Docker stack actually contains:
- `docker/compose/docker-compose.single.yml` defines exactly two services: `etcd` and `node-a`.
- `etcd` listens on client `2379` and peer `2380` inside the compose network.
- `node-a` depends on healthy etcd, mounts a runtime config, `pg_hba.conf`, and `pg_ident.conf`, and mounts secrets for `postgres-superuser-password`, `replicator-password`, and `rewinder-password`.
- `node-a` publishes two host ports via environment variables:
  - `${PGTM_SINGLE_API_PORT}:8080`
  - `${PGTM_SINGLE_PG_PORT}:5432`
- The node stores PostgreSQL data under `/var/lib/postgresql` and logs under `/var/log/pgtuskmaster`.

What the single-node runtime config says:
- `docker/configs/single/node-a/runtime.toml` sets:
  - `cluster.name = "docker-single"`
  - `cluster.member_id = "node-a"`
  - `dcs.scope = "docker-single"`
  - `dcs.endpoints = ["http://etcd:2379"]`
  - `api.listen_addr = "0.0.0.0:8080"`
  - `debug.enabled = true`
- PostgreSQL listens on `node-a:5432`.
- TLS is disabled for both PostgreSQL and the API in this sample config.
- API auth is disabled in this sample config.
- The binary paths point explicitly at PostgreSQL 16 binaries under `/usr/lib/postgresql/16/bin/...`.

Minimum PostgreSQL version evidence:
- The strongest repo evidence is that the sample single-node config hardcodes PostgreSQL 16 binary paths.
- The test harness also contains explicit PostgreSQL 16 helpers and writes `PG_VERSION` marker files containing `16`.
- I did not find any evidence in the requested files of support for multiple PostgreSQL major versions or dynamic binary discovery in this tutorial path.
- Safe phrasing for docs: this repository's provided single-node example is wired for PostgreSQL 16.

Whether behavior changes in single-node versus multi-node:
- There is no dedicated feature gate that disables HA subsystems for single-node operation.
- The same daemon entrypoint is used: `src/bin/pgtuskmaster.rs` only parses `--config <PATH>` and then calls `runtime::run_node_from_config_path(...)`.
- The same runtime schema is used for single-node and multi-node configs.
- DCS is still required even in the single-node compose example because the config still points at etcd and the process still relies on DCS-backed state.
- Trust evaluation in `src/dcs/state.rs` only demands at least two fresh members when `cache.members.len() > 1`. That means a one-member cluster can still reach `FullQuorum` if etcd is healthy, the self record exists, and the self record is fresh.
- The role credentials for `replicator` and `rewinder` are still present in the sample config even though a one-node tutorial may not exercise replication or rewind immediately.
- Because there is no special single-node branch, operator-visible behavior is mostly "same system, fewer members" rather than "reduced feature edition."

Minimal runtime config sections that appear essential in practice:
- `cluster`: required for cluster name and member identity.
- `postgres`: required. The schema requires data dir, listen host/port, socket dir, log file, local and rewind conn identities, TLS config, roles, `pg_hba`, and `pg_ident`.
- `dcs`: required. Endpoints and scope are necessary; the sample uses etcd even for one node.
- `ha`: required. Loop interval and lease TTL are part of trust and election behavior.
- `process`: required. Timeout settings and binary paths are required by the runtime schema.
- `logging`: required by the full runtime schema.
- `api`: required by the full runtime schema.
- `debug`: required by the full runtime schema, though it can be enabled or disabled.

Which config fields are optional or have optional sub-fields:
- The schema contains optional pieces inside sections, for example:
  - `dcs.init` is optional.
  - API security sub-configs can be omitted or set to disabled modes.
  - Some logging sink file paths are optional when the sink is disabled.
  - Some v2 input fields are optional because defaults or expansion logic can fill them in.
- But for a user-facing tutorial based on the shipped sample, it is safest to present the provided sample file as the authoritative starting point instead of trying to derive a smaller hand-written config.

Useful tutorial caveats drawn from the sources:
- Because `debug.enabled = true`, the tutorial can use debug endpoints from the start.
- Because API auth is disabled in the sample, curl examples do not need auth headers unless the tutorial explicitly hardens the config.
- The repo has an e2e policy test that treats post-start control as mostly hands-off: observe through `/ha/state`, request switchovers through the API, and use external process or network fault injection rather than internal worker steering.
- For a single-node tutorial, that policy suggests the learning flow should emphasize normal startup, state observation, and a small number of read-only checks instead of low-level internals manipulation.
