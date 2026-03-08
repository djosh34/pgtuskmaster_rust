## Task: Add Cluster-Wide `pgtm status` UX With Topology And Table Output <status>not_started</status> <passes>false</passes>

<priority>medium</priority>

<description>
**Goal:** Give operators one high-signal command, `pgtm status`, that answers “what is my cluster doing right now?” without forcing them to loop over nodes, compare JSON by hand, or infer cluster topology from several separate requests. The higher-order goal is to make PGTuskMaster’s operator CLI materially better than a collection of shell snippets while keeping the command surface minimal.

**Scope:**
- Design and implement a cluster-oriented `status` command that polls one or more configured node APIs and synthesizes a single cluster view.
- Add a human-first default output that is intentionally compact: a health indication plus a cluster table as the main presentation.
- Support only two output modes for now: default human output and `--json`.
- Use `status -v` as the main path for deeper operator detail instead of multiplying command nouns too early.
- Keep the command tree flat and minimal: no `ha` namespace and no extra grouping keywords for simple operator actions.
- Add `status --watch` because repeated observation is a first-class operator workflow.
- Consider a table or topology view that resembles the usefulness of `patronictl list` and CloudNativePG’s `kubectl cnpg status`, but adapt it to PGTuskMaster’s HA vocabulary and available signals.
- Ensure the command handles partial failures and disagreement explicitly instead of silently hiding unreachable nodes.

**Context from research:**
- Current CLI text output only reports a single node snapshot as `key=value` lines.
- Many docs still use shell loops over `curl` plus `jq` to compare leader, trust, and phase across nodes.
- Patroni provides cluster-level list/topology affordances.
- CloudNativePG provides a human-friendly status summary and a richer `--verbose` expansion path.
- The repo already has clear source-backed cluster signals to summarize: `leader`, `self_member_id`, `member_count`, `dcs_trust`, `ha_phase`, `ha_decision`, and `snapshot_sequence`.
- Current local design direction is to shorten the operator binary name to `pgtm` and make `status` the default mental entry point for operators.
- The current design direction is also that `pgtm` with no subcommand should behave like `pgtm status`.
- The current stable public API does not yet expose a full cluster member list in a stable machine-oriented payload, so this task may need to add or extend a stable API payload that `pgtm status` can consume.

**Expected outcome:**
- Operators can run one command and immediately see which node is primary, which are replicas, whether trust is degraded, and whether nodes disagree.
- The docs can replace most multi-node `curl` loops with a single ctl command.
- The product gets a clearer operator identity instead of stopping at a low-level transport wrapper.

Expected output direction:

```text
cluster: prod-eu1  health: healthy

NODE    SELF  ROLE     TRUST         PHASE    API
node-a  *     primary  full_quorum   primary  ok
node-b        replica  full_quorum   replica  ok
node-c        replica  full_quorum   replica  ok
```

With pending switchover:

```text
cluster: prod-eu1  health: healthy
switchover: pending -> node-b

NODE    SELF  ROLE     TRUST         PHASE    API
node-a  *     primary  full_quorum   primary  ok
node-b        replica  full_quorum   replica  ok
node-c        replica  full_quorum   replica  ok
```

Verbose direction:

```text
cluster: prod-eu1  health: degraded
warning: node-c unreachable

NODE    SELF  ROLE     TRUST         PHASE     LEADER  DECISION          PGINFO              READINESS  PROCESS  API
node-a  *     primary  full_quorum   primary   node-a  no_change         Primary tl=7        ready      idle     ok
node-b        replica  fail_safe     replica   node-a  enter_fail_safe   Replica tl=7        ready      idle     ok
node-c        unknown  unknown       unknown   ?       ?                 unavailable         ?          ?        down
```

JSON direction:

```json
{
  "cluster_name": "prod-eu1",
  "health": "healthy",
  "self_member_id": "node-a",
  "nodes": [
    {
      "member_id": "node-a",
      "is_self": true,
      "role": "primary",
      "trust": "full_quorum",
      "phase": "primary",
      "api": "ok"
    }
  ]
}
```

</description>

<acceptance_criteria>
- [ ] Add a cluster-oriented `status` command under `src/cli/` and wire it through the operator binary entry point using the short CLI name `pgtm`.
- [ ] `pgtm` with no subcommand behaves as `pgtm status`.
- [ ] The default human output is compact, with the table as the main presentation and no cluttered front matter.
- [ ] Switchover state is surfaced in status only when it is present and operationally relevant.
- [ ] JSON status output includes the queried node identity explicitly, so it is always clear which node’s snapshot was used.
- [ ] `pgtm status -v` provides deeper detail without requiring a separate debug-style noun for the common inspection path.
- [ ] `pgtm status -v` uses debug data when available to surface pginfo and related local-node detail, and handles debug-disabled cases cleanly.
- [ ] The command surfaces partial failures and disagreement explicitly, including unreachable nodes, leader mismatch, or multiple sampled primaries.
- [ ] Only two output modes exist for now: default human output and `--json`, with stable JSON for automation and scripting.
- [ ] `pgtm status --watch` is supported.
- [ ] The command surface does not require an `ha` prefix or similarly redundant namespace for routine operator usage.
- [ ] The implementation exposes or reuses a stable machine-readable cluster member payload so `pgtm` does not depend on unstable debug text dumps for the main status table.
- [ ] The output design is documented in the CLI reference and used in the relevant how-to guides that currently loop over nodes with `curl`.
- [ ] Tests cover healthy clusters, degraded trust, node disagreement, unreachable nodes, and multi-primary evidence.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
