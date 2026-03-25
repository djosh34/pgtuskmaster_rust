# How to check cluster health

This guide shows how to inspect the current cluster state with one operator command instead of hand-comparing per-node API responses.

## Prerequisites

- The `pgtm` CLI is available to run.
- At least one cluster node is running with an accessible API endpoint.
- Your shared runtime config either sets `pgtm.api.base_url` or uses an `api.listen_addr` that is already operator-reachable.

If you are using the local Docker HA cluster, first confirm the stack is up and then seed `pgtm` from one of the shipped operator configs:

```bash
docker compose ps
pgtm -c docker/pgtm.toml status
```

## Read the current cluster view

Run the default status path:

```bash
pgtm -c config.toml
```

The explicit form is the same:

```bash
pgtm -c config.toml status
```

The command starts from one node API, reads that node's `/state` payload, and renders the DCS member slots plus the seed node's local HA/process view.

## Choose the presentation

Use the default human table when you want a fast operator read:

```bash
pgtm -c config.toml status
```

Use JSON when you want automation-friendly output:

```bash
pgtm -c config.toml --json
```

Use verbose mode when you want deeper per-node detail:

```bash
pgtm -c config.toml status -v
```

Use watch mode when you want repeated observation:

```bash
pgtm -c config.toml status --watch
```

## Interpret the human output

The default renderer is centered on one short summary line plus a node table:

```text
cluster: cluster-a  health: healthy
queried via: node-a

NODE    SELF  ROLE     TRUST         PHASE    API
node-a  *     primary  full_quorum   primary  ok
node-b        replica  full_quorum   replica  ok
node-c        replica  full_quorum   replica  ok
```

The high-signal fields are:

- `health`: `healthy` when the seed node exposes quorum, visible DCS members, and an authoritative primary; `degraded` when one of those signals is missing
- `queried via`: the seed member used for cluster discovery
- `SELF`: marks that seed member in the table
- `ROLE`: the role derived from each DCS member slot's PostgreSQL observation
- `TRUST`: the seed node's published DCS authority label
- `PHASE`: the seed node's local HA/process detail when you use `-v`
- `API`: currently `-`; the present CLI does not sample peer APIs

Warnings appear only when the seed node's published view is degraded. Typical warning causes are:

- the seed node reporting `no_quorum`
- the seed node projecting no authoritative primary
- the seed node exposing no DCS member slots

## Interpret the JSON output

`pgtm --json` emits the rendered state command output built from the seed node's payload.

The top-level fields you will usually care about first are:

- `cluster_name`
- `scope`
- `queried_via`
- `discovered_member_count`
- `health`
- `warnings`
- `switchover`
- `state`

## Use the cluster view to answer operator questions

For a quick operational check, look for:

- one node with `ROLE=primary`
- replicas reporting `ROLE=replica`
- `TRUST=quorum`
- no warning lines

For a suspected incident, look for:

- `health: degraded`
- warnings about `no_quorum`, no primary, or no visible DCS members
- members showing `ROLE=unknown` or `READINESS=not_ready`
- verbose process detail on the seed node that explains whether it is promoting, starting, or idle

## Resolve connection targets without scraping status output

Use the status table to understand cluster health, then use the connection helpers when you actually need a PostgreSQL target:

```bash
pgtm -c config.toml primary
pgtm -c config.toml replicas
```

When a member publishes both routes, these commands prefer the operator-routable PostgreSQL target and fall back to the cluster route only when no operator route is available. That keeps operator scripts off the table renderer while still letting the runtime keep an internal HA route for replication and rewind. For example, to connect to the current primary:

```bash
psql "$(pgtm -c config.toml primary)"
```

If your PostgreSQL client needs path-backed TLS flags, use:

```bash
pgtm -c config.toml primary --tls
```

## Troubleshoot connectivity

If the CLI reports a `transport error`, verify:

- the seed operator config or `pgtm.api.base_url` is correct and reachable
- the chosen node API is listening on the expected port
- network access from the host running `pgtm`
- TLS and role-token material in the operator config still match the target node API
