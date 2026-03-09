# How to check cluster health

This guide shows how to inspect the current cluster state with one operator command instead of hand-comparing per-node API responses.

## Prerequisites

- The `pgtm` CLI is available to run.
- At least one cluster node is running with an accessible API endpoint.
- Your shared runtime config either sets `[pgtm].api_url` or uses an `api.listen_addr` that is already operator-reachable.

If you are using the local Docker HA cluster, you can first print the current endpoints and topology with:

```bash
tools/docker/cluster.sh status --env-file .env.docker.example
```

Or, when your local stack is configured through `.env.docker`:

```bash
make docker-status-cluster
```

## Read the current cluster view

Run the default status path:

```bash
pgtm -c /etc/pgtuskmaster/config.toml
```

The explicit form is the same:

```bash
pgtm -c /etc/pgtuskmaster/config.toml status
```

The command starts from one node API, reads the stable HA state payload, discovers peer API URLs from the stable member list, then samples those peers to build a cluster view.

## Choose the presentation

Use the default human table when you want a fast operator read:

```bash
pgtm -c /etc/pgtuskmaster/config.toml status
```

Use JSON when you want automation-friendly output:

```bash
pgtm -c /etc/pgtuskmaster/config.toml --json
```

Use verbose mode when you want deeper per-node detail:

```bash
pgtm -c /etc/pgtuskmaster/config.toml status -v
```

Use watch mode when you want repeated observation:

```bash
pgtm -c /etc/pgtuskmaster/config.toml status --watch
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

- `health`: `healthy` when the CLI sampled a consistent cluster view; `degraded` when it saw unreachable members, disagreement, degraded trust, or incomplete sampling
- `queried via`: the seed member used for cluster discovery
- `SELF`: marks that seed member in the table
- `ROLE`: the sampled node role as seen from that node's own HA state
- `TRUST`: the sampled DCS trust value
- `PHASE`: the sampled HA phase
- `API`: whether the CLI could sample that member's API (`ok`, `down`, or `missing`)

Warnings appear only when the cluster view is degraded. Typical warning causes are:

- one or more unreachable nodes
- missing advertised peer API URLs
- leader mismatch across sampled nodes
- more than one sampled primary
- incomplete sampling

## Interpret the JSON output

`pgtm --json` emits the aggregated cluster view. It is not just the seed node's raw HA state payload.

The top-level fields you will usually care about first are:

- `cluster_name`
- `scope`
- `queried_via`
- `sampled_member_count`
- `discovered_member_count`
- `health`
- `warnings`
- `switchover`
- `nodes`

Each `nodes[]` entry records whether that member was actually sampled and, if not, why:

- `sampled`
- `api_status`
- `observation_error`

That means automation can distinguish "member exists in discovery data" from "member responded successfully."

## Use the cluster view to answer operator questions

For a quick operational check, look for:

- one node with `ROLE=primary`
- replicas reporting `ROLE=replica`
- `TRUST=full_quorum` across sampled nodes
- `API=ok` for the members you expect to be reachable
- no warning lines

For a suspected incident, look for:

- `health: degraded`
- warnings about leader mismatch or multiple sampled primaries
- `TRUST=fail_safe` or `TRUST=not_trusted`
- members stuck in transition phases such as `candidate_leader`, `rewinding`, `bootstrapping`, `fencing`, or `waiting_switchover_successor`

## Resolve connection targets without scraping status output

Use the status table to understand cluster health, then use the connection helpers when you actually need a PostgreSQL target:

```bash
pgtm -c /etc/pgtuskmaster/config.toml primary
pgtm -c /etc/pgtuskmaster/config.toml replicas
```

That keeps operator scripts off the table renderer. For example, to connect to the current primary:

```bash
psql "$(pgtm -c /etc/pgtuskmaster/config.toml primary)"
```

If your PostgreSQL client needs path-backed TLS flags, use:

```bash
pgtm -c /etc/pgtuskmaster/config.toml primary --tls
```

## Troubleshoot connectivity

If the CLI reports a `transport error`, verify:

- the seed operator config or `[pgtm].api_url` is correct and reachable
- the node APIs are listening on the expected ports
- network access from the host running `pgtm`
- peer nodes are publishing usable `api_url` values in cluster discovery data

If a node shows `API=missing`, the cluster discovered that member in stable DCS state but did not have an operator-reachable peer API URL to sample. Fix the node's published API target before treating the cluster view as fully healthy.
