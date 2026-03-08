## Task: Add Primary Resolution And Shell-Friendly Connection Helpers To `pgtm` <status>not_started</status> <passes>false</passes>

<priority>medium</priority>

<description>
**Goal:** Add carefully designed shell-oriented commands to `pgtm` that let operators resolve the current primary or replica endpoints and feed that information into `psql`, scripts, or automation without scraping human output. The higher-order goal is to support fast operator workflows like “connect me to the current primary” while still keeping the CLI output principled and parseable.

**Scope:**
- Design commands or subcommands that expose the current primary member, replica members, and connection details in a script-friendly way.
- Prefer minimal verbs and nouns such as `pgtm primary`, `pgtm replicas`, or `pgtm connect-info` over deep command nesting.
- Make DSN output the operator-first contract for now.
- Ensure any shell-friendly output has a dedicated stable mode instead of encouraging users to scrape the human table/status view.
- Reuse DCS member record data for PostgreSQL host and port instead of requiring duplicated node connection info in config.
- Keep the default DSN concise, then add an explicit `--tls` flag for TLS-expanded DSNs rather than always printing TLS connection parameters.
- Document intended use with `psql` and shell pipelines, but avoid inventing ambiguous magic that hides errors or silently picks unsafe targets.

**Context from research:**
- The user explicitly raised the idea of piping the current primary into `psql`; this is directionally useful, but only if the output contract is deliberate.
- Patroni exposes DSN/query-oriented operator helpers, which is one reason `patronictl` is more useful than plain state inspection.
- DCS member records already carry PostgreSQL host and port, which makes DSN rendering possible without extra node inventory in config.
- `pgtm` must not become a PostgreSQL client in this task. It only resolves and prints DSNs.
- Today the CLI exposes only HA API state and does not help operators bridge from member identity to a live PostgreSQL connection target.
- This should be designed after or together with config-backed contexts and cluster-wide status, because connection helpers become much more useful once the CLI knows the cluster inventory.
- Current local design direction is to keep the command tree shallow and avoid `ha`, `cluster`, or other redundant prefixes for common operator commands.
- The current design direction is to keep output modes simple here too: default human output and `--json`.
- The current design direction is to support an explicit `--tls` flag that expands DSNs with TLS settings using config-backed paths where that is safe and well-defined.
- The TLS-expanded DSN path should be driven by semantic PostgreSQL client TLS config, not by print-only field names.
- `--tls` should default to `sslmode=verify-full` for v1 instead of introducing a separate sslmode knob unless a real use case proves it necessary.

**Expected outcome:**
- Operators have a supported, scriptable way to discover the current primary or replicas without fragile text scraping.
- The CLI can support practical workflows such as targeted SQL verification, connection-string export, and incident triage.
- The product gains a concrete differentiator beyond “curl wrapper with prettier text.”

Expected command/output direction:

```bash
pgtm -c config.toml primary
pgtm -c config.toml primary --json
pgtm -c config.toml primary --tls

pgtm -c config.toml replicas
pgtm -c config.toml replicas --json
pgtm -c config.toml replicas --tls
```

Expected default output:

```text
host=node-a.db.example.com port=5432 user=postgres dbname=postgres
```

Expected `replicas` default output:

```text
host=node-b.db.example.com port=5432 user=postgres dbname=postgres
host=node-c.db.example.com port=5432 user=postgres dbname=postgres
```

Expected `--tls` output direction:

```text
host=node-a.db.example.com port=5432 user=postgres dbname=postgres sslmode=verify-full sslrootcert=/etc/pgtm/postgres-ca.pem sslcert=/etc/pgtm/postgres.crt sslkey=/run/secrets/postgres.key
```

TLS field source rules:
- Use `[pgtm.postgres_client]` when present.
- Otherwise fall back to `[pgtm.api_client]`.
- `pgtm` prints DSNs only; it does not open PostgreSQL connections.

</description>

<acceptance_criteria>
- [ ] Define and implement a stable DSN-oriented contract for `pgtm primary` and `pgtm replicas`.
- [ ] The chosen command names stay minimal and flat under `pgtm`, without requiring users to traverse redundant namespaces.
- [ ] The implementation distinguishes clearly between human output and machine/script output so automation does not depend on table formatting.
- [ ] The command output contract works with only default human output and `--json`; it does not introduce an open-ended output-format matrix yet.
- [ ] `pgtm primary` renders a DSN for the current primary in human output.
- [ ] `pgtm replicas` renders one replica DSN per line in human output.
- [ ] The DSN rendering uses cluster state rather than duplicated node definitions in config.
- [ ] The default DSN output stays concise and does not always inject optional TLS fields.
- [ ] `pgtm primary --tls` and `pgtm replicas --tls` are supported.
- [ ] `--tls` uses `sslmode=verify-full` for v1 unless implementation evidence forces a more configurable design.
- [ ] `pgtm primary --tls` is explicit about which PostgreSQL client TLS fields can be emitted, such as `sslmode`, `sslrootcert`, `sslcert`, and `sslkey`, and it does not reuse unrelated API TLS server fields.
- [ ] If `[pgtm.postgres_client]` is absent, TLS-expanded DSN printing falls back to `[pgtm.api_client]`.
- [ ] The task does not add internal PostgreSQL connection support to `pgtm`.
- [ ] Error handling is explicit for no leader, disagreement between nodes, incomplete inventory, or unavailable connection metadata.
- [ ] Docs include at least one supported `psql` or shell-pipeline example that uses the new contract without text scraping.
- [ ] Tests cover healthy primary resolution, no-leader cases, disagreement cases, and output-shape stability.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
