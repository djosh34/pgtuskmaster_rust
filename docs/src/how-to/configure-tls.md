# Configure TLS

This guide shows how to enable TLS for the two server surfaces that `pgtuskmaster` exposes:

- the HTTP API
- the managed PostgreSQL server

## Goal

Enable TLS on one or both surfaces while keeping `pgtm` truthful for operators.

## Current schema

The runtime now models TLS directly on the real typed config blocks:

```toml
[postgres]
tls = { mode = "disabled" }
tls = { mode = "enabled", identity = { cert_chain = { path = "/path/to/postgres-chain.pem" }, private_key = { path = "/path/to/postgres-key.pem" } } }

[api]
transport = { transport = "http" }
transport = { transport = "https", tls = { identity = { cert_chain = { path = "/path/to/api-chain.pem" }, private_key = { path = "/path/to/api-key.pem" } } } }
```

## Enable API TLS

Update the API block:

```toml
[api]
listen_addr = "0.0.0.0:8443"
transport = { transport = "https", tls = { identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } } } }
auth = { type = "disabled" }
```

If you also want API client certificate verification:

```toml
[api]
listen_addr = "0.0.0.0:8443"
transport = { transport = "https", tls = { identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/api-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/api-key.pem" } }, client_auth = { client_certificate = "required", client_ca = { path = "/etc/pgtuskmaster/tls/client-ca.pem" }, allowed_common_names = ["operator-a"] } } }
auth = { type = "disabled" }
```

Use `client_certificate = "optional"` if you want to validate presented client certs without requiring them for every caller.

## Add operator client settings for `pgtm`

If the daemon binds an unspecified address such as `0.0.0.0:8443`, add an operator-facing `pgtm` block so the CLI resolves the real URL and TLS material correctly:

```toml
[pgtm.api]
base_url = "https://db-a.example.com:8443"
auth = { type = "role_tokens", read_token = { path = "/run/secrets/api-read-token" }, admin_token = { path = "/run/secrets/api-admin-token" } }
tls = { ca_cert = { path = "/etc/pgtm/api-ca.pem" } }
```

If the API requires client certificates, also set:

```toml
[pgtm.api]
tls = { ca_cert = { path = "/etc/pgtm/api-ca.pem" }, identity = { cert = { path = "/etc/pgtm/operator.crt" }, key = { path = "/etc/pgtm/operator.key" } } }
```

## Enable PostgreSQL TLS

Enable server-side PostgreSQL TLS and, when needed, rewind verification:

```toml
[postgres]
tls = { mode = "enabled", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/postgres-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/postgres-key.pem" } } }

[postgres.rewind.transport]
ssl_mode = "verify_full"
ca_cert = { path = "/etc/pgtuskmaster/tls/postgres-ca.pem" }
```

If PostgreSQL itself should verify client certificates, add client auth:

```toml
[postgres]
tls = { mode = "enabled", identity = { cert_chain = { path = "/etc/pgtuskmaster/tls/postgres-chain.pem" }, private_key = { path = "/etc/pgtuskmaster/tls/postgres-key.pem" } }, client_auth = { client_certificate = "required", client_ca = { path = "/etc/pgtuskmaster/tls/postgres-client-ca.pem" } } }
```

The old `local_conn_identity` and `rewind_conn_identity` blocks no longer exist. Local SQL uses the configured superuser role and `postgres.local_database`; rewind uses `postgres.roles.rewinder` and `postgres.rewind`.

If you want `pgtm primary --tls` and `pgtm replicas --tls` to print PostgreSQL client certificate paths, configure:

```toml
[pgtm.postgres.tls]
ca_cert = { path = "/etc/pgtm/postgres-ca.pem" }
identity = { cert = { path = "/etc/pgtm/postgres.crt" }, key = { path = "/etc/pgtm/postgres.key" } }
```

## Put the certificate material in place

Before restarting nodes, make sure every configured path exists and is readable by the runtime:

- API cert chain and private key
- PostgreSQL cert chain and private key
- optional client CA bundle for API mTLS
- optional client CA bundle for PostgreSQL client auth
- any `pgtm` CA and client identity material

## Roll out the change

For a staged rollout:

1. Deploy cert and key files to every node.
2. Update client trust material.
3. Change the target surface to HTTPS or enabled PostgreSQL TLS.
4. Restart the node.
5. Verify clients reconnect with TLS.

## Verify API TLS

```bash
pgtm -c config.toml status
```

This verifies that the CLI can reach the HTTPS API with the configured CA bundle, optional client identity, and any configured auth tokens.

## Verify PostgreSQL TLS

```bash
pgtm -c config.toml primary --tls
psql "$(pgtm -c config.toml primary --tls)"
```

## Troubleshooting

### Startup fails immediately after enabling TLS

Check the configured cert/key/CA paths first. HTTPS and enabled PostgreSQL TLS both require readable PEM material.

### `pgtm` cannot reach the API after the change

Check:

- `pgtm.api.base_url`
- `pgtm.api.tls`
- `pgtm.api.auth`

### PostgreSQL clients fail after the change

Check:

- `postgres.tls`
- `postgres.rewind.transport`
- `pgtm.postgres.tls` if you rely on `--tls` DSN output
