# slice-08

## Ownership

- `docs/src/lifecycle/bootstrap.md` (7 claims)
- `docs/src/assurance/safety-invariants.md` (1 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/lifecycle/bootstrap.md`

- `claim-0051` | `docs/src/lifecycle/bootstrap.md:6` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | - initializing a new primary
- `claim-0052` | `docs/src/lifecycle/bootstrap.md:7` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | - cloning as a replica from a healthy source
- `claim-0053` | `docs/src/lifecycle/bootstrap.md:14` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | HasData -->|no| HasLeader{Healthy leader evidence in DCS?}
- `claim-0054` | `docs/src/lifecycle/bootstrap.md:15` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | HasLeader -->|yes| Clone[Clone as replica]
- `claim-0055` | `docs/src/lifecycle/bootstrap.md:16` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | HasLeader -->|no| Init[Initialize primary]
- `claim-0056` | `docs/src/lifecycle/bootstrap.md:25` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | Startup may pause to gather enough evidence before action. That can feel slower than immediate initialization, but it avoids unsafe assumptions about leader availability and data lineage.
- `claim-0057` | `docs/src/lifecycle/bootstrap.md:29` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | Startup symptoms often determine later failover quality. If bootstrap repeatedly fails, verify binary paths, directory permissions, replication auth, and DCS scope consistency before forcing manual role assumptions.

### `docs/src/assurance/safety-invariants.md`

- `claim-0025` | `docs/src/assurance/safety-invariants.md:7` | `low` | `descriptive` | expected: `code symbol,unit test` | - conflicting leader evidence triggers conservative behavior
