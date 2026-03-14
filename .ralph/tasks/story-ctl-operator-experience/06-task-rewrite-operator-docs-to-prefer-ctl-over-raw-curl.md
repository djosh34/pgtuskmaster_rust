## Task: Rewrite Operator Docs To Use `pgtm` Instead Of Raw `curl` <status>done</status> <passes>true</passes>

<priority>medium</priority>

<description>
**Goal:** Reduce operator friction in the docs by making `pgtm` the default and primary operator interface across the documentation set. The higher-order goal is to stop teaching users to reconstruct auth headers, base URLs, endpoint paths, and ad-hoc `jq` pipelines when the product should provide one coherent operator CLI.

**Scope:**
- Audit all operator-facing docs under `docs/src/` that currently use `curl`, `jq`, or the old long CLI name.
- Rewrite those guides around `pgtm -c config.toml` so operators do not need to assemble raw HTTP commands.
- Shorten verbose how-to prose where the new `pgtm` output makes explanation redundant.
- Leave raw HTTP only in low-level API reference material where the protocol itself is the subject.
- Update command examples so the docs consistently rely on the shared runtime config plus client section instead of ad-hoc CLI auth flags.

**Context from research:**
- The old docs grew around `curl` because the current CLI is too thin, not because raw HTTP is the right user experience.
- The desired end state from design discussion is stronger than “prefer the CLI when possible”: operator docs should be rewritten around `pgtm` as the normal path.
- The docs should become shorter as the CLI becomes cluster-aware, config-driven, and able to surface status, switchover state, primary DSNs, replica DSNs, watch mode, and verbose pginfo/debug-backed detail directly.

**Expected outcome:**
- Operator docs use `pgtm` consistently and no longer teach raw `curl` for ordinary workflows.
- The operator docs become shorter and clearer because they lean on stable `pgtm` output instead of repeating endpoint mechanics.
- Raw HTTP remains only in API reference material, not as the normal operator workflow.

Expected operator examples in docs:

```bash
pgtm -c config.toml
pgtm -c config.toml status -v
pgtm -c config.toml status --watch
pgtm -c config.toml switchover
pgtm -c config.toml switchover node-b
pgtm -c config.toml primary
pgtm -c config.toml primary --tls
pgtm -c config.toml replicas
```

</description>

<acceptance_criteria>
- [x] Review and update operator-facing docs across `docs/src/` so routine workflows use `pgtm -c config.toml`.
- [x] `docs/src/how-to/` guides no longer teach raw `curl` as the normal path for cluster status, switchover, debugging, monitoring, or node inspection.
- [x] The docs are shortened where `pgtm` output already carries the needed operator meaning.
- [x] API reference material may still document raw endpoints, but operator-facing how-to pages do not rely on them.
- [x] The CLI reference and all examples use the new `pgtm` surface consistently.
- [x] No rewritten operator page leaves stale bearer-header, raw JSON body, endpoint-path, or `jq` pipeline instructions for flows that `pgtm` covers.
- [x] `make check` — passes cleanly
- [x] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [x] `make lint` — passes cleanly
- [x] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

## Execution Plan

### Scope and editorial rules
- Treat `pgtm -c <config>` as the default operator contract everywhere outside low-level API reference pages.
- Keep raw HTTP examples only where the protocol itself is the subject, specifically the API reference pages and any narrowly justified protocol-debugging note.
- Remove stale operator guidance that teaches bearer headers, endpoint paths, raw JSON request bodies, or `jq` pipelines for workflows already covered by `pgtm`.
- Shorten pages rather than translating every old `curl` example one-for-one. When the human CLI output already conveys the operator meaning, prefer one canonical command plus a short explanation of how to read it.
- Standardize the naming split across docs:
  - `pgtm` is the operator CLI.
  - `pgtuskmaster` is the daemon binary.

### Unblock truthful config-backed examples first
- The shipped Docker runtime configs in `docker/configs/single/node-a/runtime.toml` and `docker/configs/cluster/node-{a,b,c}/runtime.toml` currently expose only `api.listen_addr = "0.0.0.0:8080"` and do not define `[pgtm].api_url`.
- Host-side examples like `pgtm -c docker/configs/.../runtime.toml status` will therefore fail unless the config used by the docs also tells `pgtm` which operator-reachable API URL to use.
- The skeptical review changes the default execution choice here: do **not** mutate the shipped container runtime configs unless execution proves there is a strong reason to keep one file serving both daemon-in-container and operator-on-host use.
- Prefer a docs-owned client config path instead:
  - add a small tutorial/example config or overlay that sets `[pgtm].api_url` to the host-mapped port and, when relevant, `[pgtm.api_client]` or `[pgtm.postgres_client]`,
  - keep the daemon runtime examples container-truthful,
  - reuse that docs-side config path consistently across the rewritten tutorials and how-to pages instead of sprinkling `--base-url` overrides back in.
- Verify that the chosen config approach matches `docs/src/reference/runtime-configuration.md` and document the split clearly enough that readers understand why the docs config is operator-facing while the runtime config is daemon-facing.

### Explicit page inventory from the current grep
- Treat these files as the minimum rewrite set during execution because they still contain raw HTTP, `jq`-heavy operator flows, or old `cargo run --bin pgtm` usage:
  - `docs/src/tutorial/single-node-setup.md`
  - `docs/src/tutorial/debug-api-usage.md`
  - `docs/src/tutorial/observing-failover.md`
  - `docs/src/tutorial/first-ha-cluster.md`
  - `docs/src/how-to/add-cluster-node.md`
  - `docs/src/how-to/debug-cluster-issues.md`
  - `docs/src/how-to/handle-network-partition.md`
  - `docs/src/how-to/monitor-via-metrics.md`
  - `docs/src/how-to/configure-tls.md`
  - `docs/src/how-to/configure-tls-security.md`
  - `docs/src/how-to/remove-cluster-node.md`
  - `docs/src/how-to/overview.md`
  - `docs/src/SUMMARY.md`
  - `docs/src/overview.md`
  - `docs/src/reference/overview.md`
- Keep `docs/src/reference/debug-api.md`, `docs/src/reference/http-api.md`, and `docs/src/reference/pgtuskmaster-cli.md` out of the broad rewrite set unless they need framing adjustments to point operator readers back to `pgtm`.

### Rewrite the operator how-to guides around `pgtm`
- Sweep all operator-facing pages under `docs/src/how-to/` and rewrite them so routine workflows begin with config-backed `pgtm`.
- `docs/src/how-to/check-cluster-health.md` and `docs/src/how-to/perform-switchover.md` are already close; tighten wording and examples so they remain the canonical patterns that other pages can link to instead of re-explaining flags.
- `docs/src/how-to/add-cluster-node.md` should replace direct `curl /ha/state` polling and multi-node `curl | jq` loops with `pgtm -c ... status`, `pgtm -c ... status -v`, and only minimal `--json` usage when machine-readable output is actually the point.
- `docs/src/how-to/debug-cluster-issues.md` should stay CLI-first, but replace remaining `--base-url` examples with config-backed invocations and demote raw HTTP to a brief note that points readers to the API reference for protocol-level debugging.
- `docs/src/how-to/handle-network-partition.md` should stop teaching node-by-node `pgtm --base-url ... --json status | jq ...` loops as the default. Reframe the page around repeated `status -v`, `status --watch`, and targeted `debug verbose` checks from config-backed contexts.
- `docs/src/how-to/monitor-via-metrics.md` should become explicitly CLI-first instead of “API and CLI” first. Remove `jq`-heavy polling and raw debug endpoint examples where `status`, `status -v`, `status --watch`, and `debug verbose` already express the operator workflow.
- `docs/src/how-to/configure-tls.md` and `docs/src/how-to/configure-tls-security.md` should validate TLS and auth with `pgtm -c ...`, relying on `[pgtm.api_client]` and `[pgtm.postgres_client]` instead of raw `curl` plus `Authorization: Bearer ...` headers.
- Do a final sweep across the remaining how-to pages, including `handle-primary-failure.md`, `bootstrap-cluster.md`, `remove-cluster-node.md`, and `run-tests.md`, to catch stale old CLI naming or stray operator-facing endpoint instructions even if they were not obvious in the first grep.

### Rewrite the tutorials that still teach raw HTTP or ad-hoc CLI context
- `docs/src/tutorial/single-node-setup.md` currently teaches host-side `curl` polling and `cargo run --bin pgtm -- --base-url ...`; rewrite it around one truthful config-backed `pgtm -c ...` path once the config issue above is resolved.
- `docs/src/tutorial/debug-api-usage.md` should keep teaching the debug payload, but the steps should start with `pgtm -c ... debug verbose` and `pgtm -c ... --json debug verbose`, not raw `/debug/verbose` calls.
- Sweep `docs/src/tutorial/first-ha-cluster.md` and `docs/src/tutorial/observing-failover.md` for any remaining old long CLI name, `cargo run --bin pgtm`, or `--base-url`-driven operator examples and normalize them if present.

### Clean up navigation and reference framing
- Update `docs/src/SUMMARY.md`, `docs/src/overview.md`, and `docs/src/reference/overview.md` so navigation consistently presents `pgtm` as the operator CLI and `pgtuskmaster` as the daemon binary.
- Keep `docs/src/reference/pgtm-cli.md` as the authoritative command reference and make the operator guides lean on it instead of re-documenting every option.
- Leave `docs/src/reference/http-api.md` and `docs/src/reference/pgtuskmaster-cli.md` low-level and protocol/daemon focused.
- Keep raw endpoint examples in `docs/src/reference/debug-api.md`, but add or preserve framing that operators normally use `pgtm debug verbose` and only drop to raw HTTP when they are validating the protocol itself.

### Shortening pass after the rewrites
- After the first rewrite pass, trim repeated explanations of:
  - bearer token headers,
  - endpoint path construction,
  - raw payload field names that are only visible because old examples used `curl`,
  - long `jq` filters that duplicate `pgtm` output.
- Prefer one command and one interpretation paragraph per workflow instead of repeating the same mechanics on every page.
- If a page still needs machine-readable automation examples, use `pgtm --json ...` and keep any follow-up shell filtering minimal and explicitly secondary to the default operator path.

### Verification and completion
- Before the full repo checks, run targeted greps to prove the doc set moved in the intended direction:
  - `rg -n "curl|jq|Authorization: Bearer|cargo run --bin pgtm -- --base-url|pgtuskmaster CLI" docs/src/how-to docs/src/tutorial docs/src/overview.md docs/src/SUMMARY.md docs/src/reference/overview.md`
- Confirm that any remaining `curl` examples in operator-facing pages are either gone or explicitly justified as protocol-level exceptions.
- During execution, use the docs update workflow/skill required by the repo before final validation so the doc refresh step is not skipped.
- Run the mandatory gates and fix any fallout from doc or config edits:
  - `make check`
  - `make test`
  - `make test-long`
  - `make lint`
- Only after every required command passes should the task file be updated with completed acceptance checkboxes, `<passes>true</passes>`, task switching, commit, and push.

NOW EXECUTE
