# slice-13

## Ownership

- `docs/src/assurance/decision-model.md` (4 claims)
- `docs/src/assurance/tradeoffs-limits.md` (3 claims)
- `docs/src/quick-start/index.md` (1 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/assurance/decision-model.md`

- `claim-0008` | `docs/src/assurance/decision-model.md:6` | `low` | `descriptive` | expected: `code symbol,unit test` | - DCS trust and coordination records
- `claim-0009` | `docs/src/assurance/decision-model.md:14` | `low` | `descriptive` | expected: `code symbol,unit test` | DCS[DCS trust + cache] --> Decide
- `claim-0010` | `docs/src/assurance/decision-model.md:17` | `low` | `descriptive` | expected: `code symbol,unit test` | Decide --> Writes[DCS writes]
- `claim-0011` | `docs/src/assurance/decision-model.md:22` | `high` | `invariant` | expected: `code symbol,unit test` | A single-source decision model prevents hidden decision channels. Every major transition can be traced back to explicit observed inputs.

### `docs/src/assurance/tradeoffs-limits.md`

- `claim-0022` | `docs/src/assurance/tradeoffs-limits.md:5` | `low` | `descriptive` | expected: `code symbol,unit test` | ## Primary tradeoffs
- `claim-0023` | `docs/src/assurance/tradeoffs-limits.md:13` | `low` | `descriptive` | expected: `code symbol,unit test` | - Coordination quality depends on etcd health and consistent scope usage.
- `claim-0024` | `docs/src/assurance/tradeoffs-limits.md:14` | `low` | `descriptive` | expected: `code symbol,unit test` | - Recovery speed depends on rewind/bootstrap prerequisites and network access.

### `docs/src/quick-start/index.md`

- `claim-0134` | `docs/src/quick-start/index.md:5` | `medium` | `behavioral` | expected: `real-binary e2e,runtime log evidence` | You will move through three steps:
