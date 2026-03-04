# slice-10

## Ownership

- `docs/src/lifecycle/recovery.md` (6 claims)
- `docs/src/lifecycle/index.md` (2 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/lifecycle/recovery.md`

- `claim-0062` | `docs/src/lifecycle/recovery.md:3` | `high` | `descriptive` | expected: `real-binary e2e,code symbol` | After failover or fencing events, nodes may need recovery work before they can safely follow or become eligible again.
- `claim-0063` | `docs/src/lifecycle/recovery.md:6` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | - rewind when divergence is recoverable
- `claim-0064` | `docs/src/lifecycle/recovery.md:7` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | - bootstrap when rewind is unsafe or unavailable
- `claim-0065` | `docs/src/lifecycle/recovery.md:8` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | - rejoin as replica after data and coordination state are coherent
- `claim-0066` | `docs/src/lifecycle/recovery.md:12` | `high` | `invariant` | expected: `real-binary e2e,code symbol` | A node that was previously primary can carry divergent history. Recovery ensures that rejoin behavior does not reintroduce stale or conflicting timelines.
- `claim-0067` | `docs/src/lifecycle/recovery.md:20` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | If a node repeatedly fails to rejoin, treat identity, replication auth, and rewind connectivity as first-class diagnostics. Do not force eligibility until recovery preconditions are satisfied.

### `docs/src/lifecycle/index.md`

- `claim-0107` | `docs/src/lifecycle/index.md:11` | `high` | `descriptive` | expected: `real-binary e2e,code symbol` | 5. Fail-safe and fencing
- `claim-0108` | `docs/src/lifecycle/index.md:14` | `medium` | `behavioral` | expected: `real-binary e2e,code symbol` | Use this section when behavior changes over time and you need to understand transition logic, not only static configuration.
