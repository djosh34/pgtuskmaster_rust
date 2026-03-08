# Extra context for docs/src/how-to/configure-tls.md and docs/src/how-to/configure-tls-security.md

The requested TLS docs must stay honest about the current repository state.

What is clearly present in source:

- Runtime config supports `api.security.tls` and `postgres.tls`.
- `src/config/schema.rs` defines TLS server config with `mode`, `identity`, and optional `client_auth`.
- Valid TLS modes are `disabled`, `optional`, and `required`.
- When TLS mode is not `disabled`, `src/tls.rs` requires `identity.cert_chain` and `identity.private_key`.
- Optional client certificate verification is configured through `client_auth.client_ca` and `require_client_cert`.
- The API security block also supports token auth, and published docs already state that read and admin bearer tokens exist.

What is clearly *not* present in the current docker cluster example configs:

- `docker/configs/cluster/node-a/runtime.toml` and `node-b/runtime.toml` both set `postgres.tls.mode = "disabled"` and `api.security.tls.mode = "disabled"`.
- The example cluster configs do not ship a working enabled-TLS example.
- A draft must not claim the cluster example already demonstrates enabled TLS.

Certificate-generation support present in the repo:

- A file search under `docker/` did not find checked-in PEM, CRT, or KEY assets.
- The current repository snapshot also does not expose an obvious dedicated certificate generation script in `docker/`.
- `docker/Dockerfile.dev` installs general tooling but does not, by itself, define a certificate creation workflow.
- Therefore, any TLS how-to should explain configuration shape and verification steps without claiming the repo ships a turnkey PKI bootstrap helper unless a source file explicitly shows one.

Verification commands that are supportable from current docs and source:

- It is safe to use API observation commands such as `curl` against `/ha/state` and other HTTP endpoints, because those endpoints are published.
- It is safe to mention `pgtuskmasterctl` with `--read-token` or `--admin-token` when documenting auth alongside TLS, because those flags are already in published reference docs.
- It is not safe to claim a tested `psql` TLS example command with concrete cert file paths unless those paths are sourced from a real config or generated asset in the repo.

Practical framing constraints:

- Because current sample configs leave TLS disabled, a how-to should read as an operator procedure for adapting a deployment, not as a "copy the shipped example and you're done" tutorial.
- The doc can show what fields must be populated and what failure modes the runtime enforces if required identity material is absent.
- The doc should separate API bearer-token auth from TLS transport security, since they are configured independently.
