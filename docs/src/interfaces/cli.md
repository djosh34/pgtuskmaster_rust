# CLI Workflows

`pgtuskmasterctl` is the supported command-line client for the node API. Use it when you want the API's typed responses, auth handling, and output rendering without hand-writing HTTP requests. The CLI is intentionally small because it mirrors the same small API surface rather than adding side channels.

By default the CLI talks to `http://127.0.0.1:8080`, which matches the node's default API listen address. Override `--base-url` when the API is bound elsewhere or exposed through HTTPS.

## What the CLI can do

The current CLI surface maps directly onto the node API:

- read the current HA state with `ha state`
- request a planned switchover with `ha switchover request`
- clear a pending switchover request with `ha switchover clear`

The CLI does not bypass API authorization, TLS policy, validation, or sequencing. If the API would reject or delay something, the CLI will expose that result rather than hide it.

## Common global options

The options you are most likely to use are:

- `--base-url` to point at a non-default HTTP or HTTPS listener
- `--read-token` or `PGTUSKMASTER_READ_TOKEN` for read routes
- `--admin-token` or `PGTUSKMASTER_ADMIN_TOKEN` for write routes
- `--timeout-ms` to bound request time
- `--output json|text` to choose machine-readable or condensed human-readable output

Token fallback is intentional:

- `ha state` accepts a read token and falls back to an admin token if no read token is provided
- write commands require an admin token

That means a secured read-only automation path can use a weaker credential, while an operator still has the option to use one admin token for both read and write workflows when appropriate.

## Checking node state

For a local node with the default listener:

```console
pgtuskmasterctl ha state
```

For a secured endpoint, pass the base URL and read token explicitly:

```console
PGTUSKMASTER_READ_TOKEN="$(cat /run/secrets/pgtuskmaster-read-token)" \
pgtuskmasterctl \
  --base-url https://node-a.example.internal:8443 \
  ha state
```

`ha state` returns the same typed payload as `GET /ha/state`, including:

- cluster and scope identity
- the current leader, if one is known
- any pending switchover requester
- the current HA phase and decision
- the latest snapshot sequence number

Use this command before manual intervention. It is the fastest contract-level answer to "what does this node currently believe?" In `--output text` mode the CLI flattens the response into key-value lines, which is useful for shell inspection but deliberately less expressive than the full JSON payload.

## Requesting a switchover

The request workflow is:

```console
PGTUSKMASTER_ADMIN_TOKEN="$(cat /run/secrets/pgtuskmaster-admin-token)" \
pgtuskmasterctl \
  --base-url https://node-a.example.internal:8443 \
  ha switchover request \
  --requested-by node-b
```

The CLI sends `POST /switchover` with:

```text
{
  "requested_by": "node-b"
}
```

Interpret the result carefully:

- `accepted=true` means the API recorded the intent
- it does not mean the role change already completed
- `--requested-by` identifies who asked for the switchover; it does not choose a successor in the current implementation

The practical operator workflow is therefore:

1. read current state
2. submit switchover intent
3. keep reading state until the lifecycle result is visible

That avoids a common misread where `accepted=true` is treated as equivalent to "promotion finished."

## Clearing a pending switchover

If you need to remove queued switchover intent before it completes:

```console
PGTUSKMASTER_ADMIN_TOKEN="$(cat /run/secrets/pgtuskmaster-admin-token)" \
pgtuskmasterctl \
  --base-url https://node-a.example.internal:8443 \
  ha switchover clear
```

This maps to `DELETE /ha/switchover` and also returns `accepted=true` when the request was recorded successfully. As with the write command, the response acknowledges the API operation, not the eventual observation state on every node.

## Authentication, transport, and failure classes

The CLI follows the API's role model:

- `ha state` accepts a read token or an admin token
- `ha switchover request` and `ha switchover clear` require an admin token
- when API auth is disabled, the CLI sends no bearer token

The CLI also separates failure classes in its exit behavior:

- transport and request-build failures exit differently from API status failures
- non-expected HTTP statuses are surfaced with the status code and body
- JSON decode failures are surfaced separately from transport failures

That distinction matters in automation. A connection refusal, a `403`, and a malformed response body are not interchangeable failure modes and should not trigger the same runbook response.

If the API uses TLS, point `--base-url` at the HTTPS listener and run the CLI in an environment that trusts the server certificate chain. The CLI does not manage client certificates for you; it relies on the platform trust and reqwest client configuration.

## Practical output guidance

Use `--output json` for automation and incident capture:

```console
pgtuskmasterctl --output json ha state
```

Use `--output text` when you want a lighter terminal view:

```console
pgtuskmasterctl --output text ha state
```

Text mode is intentionally compact. It is best for quick inspection, not archival evidence. When you need to compare full HA decisions or preserve a complete response body for later analysis, keep the JSON output.
