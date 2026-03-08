// todo: remove placeholder diagram text and replace it with real markdown content only; no bracketed artifact placeholders.
# Configure TLS Security

This guide shows how to enable TLS for the pgtuskmaster API and PostgreSQL connections.

## Prerequisites

- TLS certificate and private key in PEM format for the pgtuskmaster API
- TLS certificate and private key in PEM format for PostgreSQL
- Optional: CA certificate for client certificate verification
- Access to edit runtime configuration files
- Ability to restart pgtuskmaster services

## Configuration Structure

The runtime.toml file contains two independent TLS sections:

## Enable API TLS

Edit your runtime.toml and locate the `[api.security.tls]` section:

```toml
[api.security.tls]
mode = "required"  # or "optional" to allow both TLS and plaintext

[api.security.tls.identity]
cert_chain = { path = "/path/to/api-server.crt" }
private_key = { path = "/path/to/api-server.key" }

# Optional client certificate verification
[api.security.tls.client_auth]
client_ca = { path = "/path/to/client-ca.crt" }
require_client_cert = false  # Set true to enforce mutual TLS
```

Set `mode` to `required` to enforce TLS for all API clients. The identity section must provide valid PEM-encoded certificate and private key files.

## Enable PostgreSQL TLS

Edit the `[postgres.tls]` section:

```toml
[postgres.tls]
mode = "required"  # PostgreSQL will reject non-TLS connections

[postgres.tls.identity]
cert_chain = { path = "/path/to/postgres-server.crt" }
private_key = { path = "/path/to/postgres-server.key" }

# Optional client certificate verification
[postgres.tls.client_auth]
client_ca = { path = "/path/to/postgres-client-ca.crt" }
require_client_cert = false
```

After PostgreSQL TLS is enabled, update the connection identities to reflect the SSL mode.
// todo: verify the TOML example matches the actual schema shape. The existing runtime examples use `local_conn_identity = { ... }` and `rewind_conn_identity = { ... }` inside `[postgres]`, so do not invent an unsupported table layout.

```toml
[postgres.local_conn_identity]
ssl_mode = "require"  # or "prefer" for optional TLS

[postgres.rewind_conn_identity]
ssl_mode = "require"
```

## Apply Configuration

Copy the updated runtime.toml to each cluster node:

```bash
cp runtime.toml /etc/pgtuskmaster/runtime.toml
```

Restart pgtuskmaster on each node:

```bash
systemctl restart pgtuskmaster
```

Check service status to ensure clean startup:

```bash
systemctl status pgtuskmaster
```

## Verify TLS Operation

Test the API with curl:

```bash
curl -k https://localhost:8080/ha/state
```

Replace `localhost:8080` with your configured `api.listen_addr`. The `-k` flag skips certificate validation for initial testing.

For mutual TLS verification, provide a client certificate:

```bash
curl -k --cert client.crt --key client.key https://localhost:8080/ha/state
```

Test PostgreSQL TLS with psql:

```bash
PGSSLMODE=require psql "host=node-a port=5432 user=postgres dbname=postgres"
```

## Troubleshooting

**Missing identity files**: pgtuskmaster will fail to start if `tls.mode` is not `disabled` but `tls.identity` is absent. Check logs for `TlsConfigError::InvalidConfig`.

**Certificate parse errors**: Invalid PEM format triggers `TlsConfigError::PemParse`. Verify file contents contain `-----BEGIN CERTIFICATE-----` and `-----BEGIN PRIVATE KEY-----` blocks.

**Port binding failures**: Confirm no other service occupies the TLS port. The API binds to the same `listen_addr` for both TLS and plaintext modes.

**PostgreSQL connection refusal**: If `postgres.tls.mode` is `required` but clients connect without TLS, connections will be rejected. Update all client connection strings to specify `sslmode=require`.

## Client Configuration

// todo: replace the trailing "missing source support" artifact with a clean, source-supported statement. Keep bearer-token auth separate from TLS transport security.
Update applications to use HTTPS for API calls and `sslmode=require` for PostgreSQL connections.

## Next Steps

- Configure API bearer token authentication in `[api.security.auth]` for additional access control
- Set up log forwarding to capture security events
- Review PostgreSQL pg_hba.conf to restrict network access
