# slice-17

## Ownership

- `docs/src/operator/observability.md` (4 claims)
- `docs/src/quick-start/initial-validation.md` (3 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/operator/observability.md`

- `claim-0109` | `docs/src/operator/observability.md:3` | `low` | `descriptive` | expected: `code symbol,unit test` | Operational confidence depends on three simultaneous views: local PostgreSQL state, DCS trust and cache state, and HA decision output.
- `claim-0110` | `docs/src/operator/observability.md:8` | `low` | `descriptive` | expected: `code symbol,unit test` | DCS[(DCS)] --> DcsView[DCS cache + trust]
- `claim-0111` | `docs/src/operator/observability.md:17` | `low` | `descriptive` | expected: `code symbol,unit test` | No single surface explains HA behavior. Logs, API state, and DCS records together provide the full context for role decisions and blocked actions.
- `claim-0112` | `docs/src/operator/observability.md:31` | `low` | `descriptive` | expected: `code symbol,unit test` | - Inspect DCS records for leader and switchover intent coherence.

### `docs/src/quick-start/initial-validation.md`

- `claim-0135` | `docs/src/quick-start/initial-validation.md:8` | `low` | `descriptive` | expected: `real-binary e2e,runtime log evidence` | - Trust visibility: reported trust level aligns with current etcd health.
- `claim-0136` | `docs/src/quick-start/initial-validation.md:10` | `low` | `descriptive` | expected: `real-binary e2e,runtime log evidence` | - DCS coherence: scope keys exist and reflect expected membership/leader intent.
- `claim-0137` | `docs/src/quick-start/initial-validation.md:22` | `low` | `descriptive` | expected: `real-binary e2e,runtime log evidence` | - etcd endpoint mismatch or scope mismatch
