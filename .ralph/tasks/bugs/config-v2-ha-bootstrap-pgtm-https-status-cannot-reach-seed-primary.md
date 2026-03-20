## Bug: Config v2 HA bootstrap pgtm HTTPS status cannot reach seed primary <status>not_started</status> <passes>false</passes>

<description>
`make test-long` currently fails in all 16 ultra-long HA scenarios during seed-primary bootstrap. The harness waits for the bootstrap primary, but `pgtm status` exits with a transport error while requesting `https://node-b:<port>/state`.

This regression was detected on 2026-03-20 after the in-flight config_v2 runtime/operator reduction branch passed `make check`, `make lint`, and `make test`, but `make test-long` failed uniformly across the suite. Explore the HA harness materialization, operator config_v2 loading, CLI/operator client resolution, and seed-primary API startup path first, then fix the actual boundary mismatch without reintroducing legacy config adapters.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
