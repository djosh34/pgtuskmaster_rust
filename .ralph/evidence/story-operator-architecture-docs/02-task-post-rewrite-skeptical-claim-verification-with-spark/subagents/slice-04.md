# slice-04

## Ownership

- `docs/src/assurance/dcs-data-model.md` (10 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/assurance/dcs-data-model.md`

- `claim-0012` | `docs/src/assurance/dcs-data-model.md:1` | `low` | `descriptive` | expected: `code symbol,unit test` | # DCS Data Model and Write Paths
- `claim-0013` | `docs/src/assurance/dcs-data-model.md:3` | `low` | `descriptive` | expected: `code symbol,unit test` | Yes, the docs explicitly include what is written into DCS and how ownership works.
- `claim-0014` | `docs/src/assurance/dcs-data-model.md:5` | `low` | `descriptive` | expected: `code symbol,unit test` | The DCS namespace is scoped under `/<scope>/...` and stores coordination records with clear writers.
- `claim-0015` | `docs/src/assurance/dcs-data-model.md:11` | `low` | `descriptive` | expected: `code symbol,unit test` | | `/<scope>/member/<member_id>` | DCS worker | publish local member state |
- `claim-0016` | `docs/src/assurance/dcs-data-model.md:12` | `low` | `descriptive` | expected: `code symbol,unit test` | | `/<scope>/leader` | HA worker | leader lease / primary coordination |
- `claim-0017` | `docs/src/assurance/dcs-data-model.md:19` | `low` | `descriptive` | expected: `code symbol,unit test` | - DCS worker writes local membership updates as PostgreSQL state changes.
- `claim-0018` | `docs/src/assurance/dcs-data-model.md:20` | `low` | `descriptive` | expected: `code symbol,unit test` | - HA worker updates leader coordination as lifecycle decisions change.
- `claim-0019` | `docs/src/assurance/dcs-data-model.md:27` | `high` | `invariant` | expected: `code symbol,unit test` | Explicit ownership prevents hidden coordination side effects. If a record is wrong, operators and contributors can identify which component is responsible.
- `claim-0020` | `docs/src/assurance/dcs-data-model.md:31` | `medium` | `behavioral` | expected: `code symbol,unit test` | Central coordination records simplify reasoning but require trust evaluation. If DCS trust degrades, role logic must become more conservative even when local PostgreSQL appears healthy.
- `claim-0021` | `docs/src/assurance/dcs-data-model.md:35` | `low` | `descriptive` | expected: `code symbol,unit test` | When behavior is unexpected, inspect key ownership before assuming broad system failure. For example, stale switchover intent and conflicting leader records have different writers and different corrective paths.
