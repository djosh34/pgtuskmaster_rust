## Task: Unify Runtime And Operator Config Into One File Per Node With Public Endpoint Overrides <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Make the config model honest and small: the daemon keeps a full runtime config because it genuinely needs one, while `pgtm` gains support for a much smaller operator config because it does not. The higher-order goal is to eliminate the current confusion where operators are forced to carry around full daemon settings when they only need API target, auth, and TLS/client connection context.

**Scope:**
- Define and document two supported config contracts:
- a full per-node daemon runtime config
- a minimal `pgtm` operator config
- Keep the "one config per VM" path feasible: on a VM that runs the daemon locally, `pgtm` must still be able to read the node runtime config directly.
- Keep the "small operator config" path feasible: on a laptop, bastion, or local host, `pgtm` must be able to use a smaller config file that does not contain daemon-only sections.
- Add or refine explicit operator-facing/public endpoint overrides for both:
- the API base URL used by `pgtm`
- the PostgreSQL host/port published into DCS and emitted in DSN output
- Ensure secret paths are file-based and can reference files mounted inside Docker or present directly on a VM without requiring a separate env indirection layer.
- Ensure the same config shape supports full TLS for both API and PostgreSQL.
- Remove redundant docs-owned operator overlay configs for deployments where the shipped runtime config or the new minimal `pgtm` config already provides the necessary operator data.

**Context from research:**
- `src/cli/config.rs` currently refuses to derive a base URL from `0.0.0.0:8080` and tells operators to set `pgtm.api_url`.
- The docs explicitly teach a split between daemon configs and separate operator-facing configs under `docs/examples/`.
- The current docker runtime configs already contain `[pgtm].api_url`, which means the docs-owned docker operator configs are effectively duplicating shipped config.
- The user reported a real ergonomics bug: `pgtm primary` DSN output does not work outside the Docker container because the published PostgreSQL host does not model the operator-facing/public endpoint cleanly enough.
- The user wants one config file per VM to be feasible for a 3-real-VM Docker deployment.

**Expected outcome:**
- The daemon runtime config keeps only daemon-relevant sections plus public endpoint advertisement. It should still contain these sections because the runtime genuinely needs them:
- `[cluster]`
- `[postgres]`
- `[dcs]`
- `[ha]`
- `[process]`
- `[logging]`
- `[api]`
- one explicit public/advertised endpoint block for operator-reachable API and PostgreSQL addresses
- `[debug]`
- The daemon runtime config also owns the strict security defaults for the shipped stack:
- PostgreSQL server TLS settings
- PostgreSQL role auth configuration
- strict `pg_hba` and `pg_ident` sources
- API TLS server settings
- API auth token settings
- The minimal `pgtm` config becomes a genuinely small file. It should contain only the fields `pgtm` actually needs:
- seed API URL or operator-reachable node API URL
- read/admin auth token references when API auth is enabled
- API client TLS material when HTTPS is enabled
- PostgreSQL client TLS material when DSN TLS expansion is desired
- optionally a default/fallback PostgreSQL endpoint override only if the DCS-discovery path cannot provide a truthful answer by itself
- On local Docker, the canonical host-side CLI config is the minimal file:

```bash
pgtm -c docker/pgtm.toml status
pgtm -c docker/pgtm.toml primary
pgtm -c docker/pgtm.toml replicas
```

- On a real VM that runs the daemon locally, this still works directly from the node's installed runtime config:

```bash
pgtm -c /etc/pgtuskmaster/config.toml status
pgtm -c /etc/pgtuskmaster/config.toml primary
psql "$(pgtm -c /etc/pgtuskmaster/config.toml primary)"
```

- On a remote operator host that does not run the daemon, the smaller config is enough:

```bash
pgtm -c ./pgtm.toml status
pgtm -c ./pgtm.toml primary
```

- The DCS member record publishes operator-reachable API and PostgreSQL endpoints, so cluster discovery and DSN emission are truthful outside the container boundary.
- The repo and docs explain clearly that `/etc/pgtuskmaster/config.toml` is an installation path convention for real Linux VMs, not the required repo layout for local examples.

</description>

<acceptance_criteria>
- [ ] `pgtm` supports both a full daemon runtime config and a separate minimal operator config.
- [ ] One per-node daemon config file is sufficient for both the daemon and `pgtm` in the documented 3-real-VM Docker path.
- [ ] The canonical local Docker host-side CLI path uses a smaller `docker/pgtm.toml` file instead of requiring a full runtime config.
- [ ] The config model includes explicit operator/public endpoint support for both API and PostgreSQL, and those fields are documented as the supported fix for in-container bind addresses and host-unreachable DSN output.
- [ ] `pgtm status`, `pgtm primary`, and `pgtm replicas` all work from either supported config shape without `--base-url` overrides in the canonical Docker and VM paths.
- [ ] The DCS member publication path writes operator-reachable API and PostgreSQL endpoint data rather than container-only endpoint data.
- [ ] The config model stays file-based for secrets and TLS material and does not reintroduce env-file indirection as the primary UX.
- [ ] The task docs and implementation specify exactly which fields belong in the daemon runtime config versus the minimal `pgtm` config, with examples precise enough that a new operator can copy them without guesswork.
- [ ] The task docs and implementation specify which security settings remain daemon-only and why `pgtm` does not need to repeat them in its smaller config.
- [ ] Redundant docs-owned operator overlay configs for local Docker are deleted or reduced to clearly justified non-canonical examples.
- [ ] The runtime configuration reference and `pgtm` CLI docs explain the new one-file-per-node contract with exact examples.
- [ ] The docs also explain when `/etc/pgtuskmaster/config.toml` is appropriate and when shallow repo-local paths such as `docker/node-a.toml` and `docker/pgtm.toml` are preferred.
- [ ] At least one real-binary or integration-style test proves that `pgtm primary` returns a host-usable DSN from outside the container boundary.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
