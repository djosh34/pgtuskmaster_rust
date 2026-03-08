## Task: Rewrite Operator Docs To Use `pgtm` Instead Of Raw `curl` <status>not_started</status> <passes>false</passes>

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
- [ ] Review and update operator-facing docs across `docs/src/` so routine workflows use `pgtm -c config.toml`.
- [ ] `docs/src/how-to/` guides no longer teach raw `curl` as the normal path for cluster status, switchover, debugging, monitoring, or node inspection.
- [ ] The docs are shortened where `pgtm` output already carries the needed operator meaning.
- [ ] API reference material may still document raw endpoints, but operator-facing how-to pages do not rely on them.
- [ ] The CLI reference and all examples use the new `pgtm` surface consistently.
- [ ] No rewritten operator page leaves stale bearer-header, raw JSON body, endpoint-path, or `jq` pipeline instructions for flows that `pgtm` covers.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
