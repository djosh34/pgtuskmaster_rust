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

<plan>
- [ ] Collapse the operator API endpoint into one validated config_v2 shape instead of carrying loose `api_base_url`, `expected_transport`, and `api_resolve_to` fields.
  - Reuse the existing operator config types rather than adding another parallel endpoint DTO.
  - Parse `pgtm.api.base_url` once during `config_v2::load_operator_config`, validate the scheme against `expected_transport` there, and keep `resolve_to` attached to that same validated endpoint shape so later code cannot drop or recombine it incorrectly.
- [ ] Make CLI context resolution consume the validated endpoint directly.
  - Remove the downstream endpoint revalidation/recomposition in `src/cli/config.rs`.
  - Preserve `--base-url` override behavior, but validate the override against the same transport expectation in one place instead of spreading that logic across multiple helpers.
- [ ] Prove the host-observer HTTPS contract with focused tests before running the full gates.
  - Add loader coverage for operator configs that carry `base_url`, `expected_transport`, and `resolve_to`.
  - Add a CLI/client-path test that uses a non-resolvable HTTPS hostname plus `resolve_to = 127.0.0.1:<port>` and verifies `GET /state` succeeds with the configured TLS material.
  - Do not change API startup or reintroduce legacy operator adapters unless those focused tests show the endpoint boundary diagnosis is wrong.
- [ ] Implement the fix, then run `make check`, `make lint`, `make test`, and `make test-long`, ticking the acceptance boxes only after each passes.
NOW EXECUTE
</plan>
