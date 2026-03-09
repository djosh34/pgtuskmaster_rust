## Task: Unify Runtime And Operator Config Into One File Per Node With Public Endpoint Overrides <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Make one config file per node sufficient for both the daemon and `pgtm`, including correct host-reachable API and PostgreSQL connection output. The higher-order goal is to eliminate the current two-file mental model where the runtime config is "truthful" for in-container binds but operators need a second config or extra flags just to get usable `status`, `primary`, and `replicas` results.

**Scope:**
- Define one per-node config contract that works for:
- the daemon process inside Docker
- `pgtm` on the host for local Docker
- `pgtm` on real VM deployments
- Add or refine explicit operator-facing/public endpoint overrides for both:
- the API base URL used by `pgtm`
- the PostgreSQL host/port published into DCS and emitted in DSN output
- Ensure secret paths are file-based and can reference files mounted inside Docker or present directly on a VM without requiring a separate env indirection layer.
- Ensure the same config shape supports full TLS for both API and PostgreSQL.
- Remove redundant docs-owned operator overlay configs for deployments where the shipped runtime config already has the necessary public endpoint information.

**Context from research:**
- `src/cli/config.rs` currently refuses to derive a base URL from `0.0.0.0:8080` and tells operators to set `pgtm.api_url`.
- The docs explicitly teach a split between daemon configs and separate operator-facing configs under `docs/examples/`.
- The current docker runtime configs already contain `[pgtm].api_url`, which means the docs-owned docker operator configs are effectively duplicating shipped config.
- The user reported a real ergonomics bug: `pgtm primary` DSN output does not work outside the Docker container because the published PostgreSQL host does not model the operator-facing/public endpoint cleanly enough.
- The user wants one config file per VM to be feasible for a 3-real-VM Docker deployment.

**Expected outcome:**
- On local Docker, this works directly from the shipped node config:

```bash
pgtm -c docker/configs/local/node-a/config.toml status
pgtm -c docker/configs/local/node-a/config.toml primary
pgtm -c docker/configs/local/node-a/config.toml replicas
```

- On a real VM, this works directly from the node's installed config:

```bash
pgtm -c /etc/pgtuskmaster/config.toml status
pgtm -c /etc/pgtuskmaster/config.toml primary
psql "$(pgtm -c /etc/pgtuskmaster/config.toml primary)"
```

- The DCS member record publishes operator-reachable API and PostgreSQL endpoints, so cluster discovery and DSN emission are truthful outside the container boundary.
- The product no longer needs a separate "operator overlay" config for the common Docker and VM paths.

</description>

<acceptance_criteria>
- [ ] One per-node config file is sufficient for both the daemon and `pgtm` in the canonical local Docker path.
- [ ] One per-node config file is sufficient for both the daemon and `pgtm` in the documented 3-real-VM Docker path.
- [ ] The config model includes explicit operator/public endpoint support for both API and PostgreSQL, and those fields are documented as the supported fix for in-container bind addresses and host-unreachable DSN output.
- [ ] `pgtm status`, `pgtm primary`, and `pgtm replicas` all work from the same config file without `--base-url` overrides in the canonical Docker and VM paths.
- [ ] The DCS member publication path writes operator-reachable API and PostgreSQL endpoint data rather than container-only endpoint data.
- [ ] The config model stays file-based for secrets and TLS material and does not reintroduce env-file indirection as the primary UX.
- [ ] Redundant docs-owned operator overlay configs for local Docker are deleted or reduced to clearly justified non-canonical examples.
- [ ] The runtime configuration reference and `pgtm` CLI docs explain the new one-file-per-node contract with exact examples.
- [ ] At least one real-binary or integration-style test proves that `pgtm primary` returns a host-usable DSN from outside the container boundary.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
