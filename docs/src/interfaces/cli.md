# CLI Workflows

`pgtuskmasterctl` is the supported command-line client for the node API. Use it when you want the API's typed JSON responses without hand-writing HTTP requests.

By default the CLI talks to `http://127.0.0.1:8080`, which matches the node's default API listen address. Override `--base-url` when the API is bound elsewhere or when you expose it through HTTPS.

## What the CLI can do

The current CLI surface is intentionally small:

- read the current HA state with `ha state`
- request a planned switchover with `ha switchover request`
- clear a pending switchover request with `ha switchover clear`

That surface maps directly onto the node API. The CLI does not bypass API authorization, TLS policy, or request validation.

## Checking node state

For a local node with the default API listener:

```console
pgtuskmasterctl ha state
```

For a secured endpoint, pass the read token and HTTPS base URL explicitly:

```console
PGTUSKMASTER_READ_TOKEN="$(cat /run/secrets/pgtuskmaster-read-token)" \
pgtuskmasterctl \
  --base-url https://node-a.example.internal:8443 \
  ha state
```

`ha state` returns the same typed payload as `GET /ha/state`, including:

- the node's cluster and scope identity
- the current leader, if one is known
- any pending switchover requester
- the current HA phase and decision
- the latest snapshot sequence number exported by the runtime

Use that command first before attempting any manual intervention.

## Requesting a switchover

The switchover workflow is namespaced under `ha switchover`. The request command requires `--requested-by`, which is written into the DCS record and shows up in subsequent state responses.

```console
PGTUSKMASTER_ADMIN_TOKEN="$(cat /run/secrets/pgtuskmaster-admin-token)" \
pgtuskmasterctl \
  --base-url https://node-a.example.internal:8443 \
  ha switchover request \
  --requested-by node-b
```

`--requested-by` records who asked for the switchover. The current API does not let you nominate a specific successor member. The CLI sends a `POST /switchover` request with a body shaped like:

```text
{
  "requested_by": "node-b"
}
```

The server returns `202 Accepted` when it records the request. That only means the intent was accepted; the actual role change still depends on cluster state and the HA loop.

## Clearing a pending switchover

If you need to remove a queued switchover request before it completes:

```console
PGTUSKMASTER_ADMIN_TOKEN="$(cat /run/secrets/pgtuskmaster-admin-token)" \
pgtuskmasterctl \
  --base-url https://node-a.example.internal:8443 \
  ha switchover clear
```

This maps to `DELETE /ha/switchover` and also returns `202 Accepted` on success.

## Authentication and transport

The CLI follows the API's role model:

- `ha state` accepts a read token or an admin token
- `ha switchover request` and `ha switchover clear` require an admin token
- when API auth is disabled, the CLI sends no bearer token and the server treats the routes as open

The CLI does not manage client certificates for you. If the API is configured with TLS, point `--base-url` at the HTTPS listener and run the CLI in an environment that trusts the server certificate chain.

## Output and failure handling

The default output format is JSON. Use `--output text` when you want a lighter human-readable rendering:

```console
pgtuskmasterctl --output text ha state
```

Transport failures, unexpected HTTP statuses, and decode failures are surfaced as CLI errors with distinct exit codes. Treat those as API-level failures, not as evidence that the HA state itself is healthy or unhealthy.
