# slice-14

## Ownership

- `docs/src/contributors/ha-pipeline.md` (4 claims)
- `docs/src/contributors/worker-wiring.md` (3 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/contributors/ha-pipeline.md`

- `claim-0041` | `docs/src/contributors/ha-pipeline.md:7` | `low` | `descriptive` | expected: `code symbol` | 1. Read latest PostgreSQL and DCS state snapshots.
- `claim-0042` | `docs/src/contributors/ha-pipeline.md:8` | `low` | `descriptive` | expected: `code symbol` | 2. Evaluate trust posture and leader evidence.
- `claim-0043` | `docs/src/contributors/ha-pipeline.md:24` | `low` | `descriptive` | expected: `code symbol` | - local process actions: start, stop, promote, demote, rewind, bootstrap
- `claim-0044` | `docs/src/contributors/ha-pipeline.md:25` | `low` | `descriptive` | expected: `code symbol` | - coordination actions: leader lease acquire/release, switchover intent clear

### `docs/src/contributors/worker-wiring.md`

- `claim-0059` | `docs/src/contributors/worker-wiring.md:12` | `low` | `descriptive` | expected: `code symbol` | dcs: DcsWorker,
- `claim-0060` | `docs/src/contributors/worker-wiring.md:19` | `low` | `descriptive` | expected: `code symbol` | The runtime starts workers with shared state receivers/senders and coordination store handles. Each worker owns one primary output state and consumes specific upstream inputs.
- `claim-0061` | `docs/src/contributors/worker-wiring.md:24` | `low` | `descriptive` | expected: `code symbol` | - DCS worker publishes trust plus cache state and writes local membership.
