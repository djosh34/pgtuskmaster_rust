# slice-15

## Ownership

- `docs/src/lifecycle/failover.md` (4 claims)
- `docs/src/introduction.md` (3 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/lifecycle/failover.md`

- `claim-0077` | `docs/src/lifecycle/failover.md:3` | `high` | `descriptive` | expected: `real-binary e2e,code symbol` | Failover addresses unplanned primary loss or primary unsafety. Candidate nodes evaluate whether promotion is safe under current trust and readiness evidence.
- `claim-0078` | `docs/src/lifecycle/failover.md:5` | `medium` | `behavioral` | expected: `real-binary e2e,code symbol` | Promotion is not granted solely because the old leader is unreachable. The node also requires sufficient coordination confidence and local readiness.
- `claim-0079` | `docs/src/lifecycle/failover.md:9` | `high` | `descriptive` | expected: `real-binary e2e,code symbol` | Failover is where optimistic assumptions are most dangerous. The lifecycle intentionally makes promotion conditional to prevent split-brain during partial failures.
- `claim-0080` | `docs/src/lifecycle/failover.md:17` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | During an outage, the key question is not "why is promotion delayed" in isolation. The key question is whether evidence quality supports safe promotion. Use trust, lease, and readiness signals together.

### `docs/src/introduction.md`

- `claim-0081` | `docs/src/introduction.md:3` | `high` | `descriptive` | expected: `code symbol,real-binary e2e` | This system is a local high-availability control plane for PostgreSQL. One `pgtuskmaster` node supervises one PostgreSQL instance, participates in shared coordination through etcd, and continuously decides whether that PostgreSQL should run as primary, replica, or in a conservative safety state.
- `claim-0082` | `docs/src/introduction.md:5` | `high` | `descriptive` | expected: `code symbol,real-binary e2e` | The practical goal is to keep role transitions safe and predictable. When a cluster is healthy, the system supports stable leadership, planned switchovers, and unplanned failover handling. When signals are inconsistent, the system prefers actions that reduce split-brain risk, even when those actions temporarily reduce write availability.
- `claim-0083` | `docs/src/introduction.md:11` | `low` | `descriptive` | expected: `code symbol,real-binary e2e` | Node <-->|coordination| ETCD[(etcd / DCS)]
