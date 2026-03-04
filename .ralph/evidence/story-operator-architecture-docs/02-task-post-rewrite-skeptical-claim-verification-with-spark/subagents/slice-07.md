# slice-07

## Ownership

- `docs/src/assurance/safety-case.md` (7 claims)
- `docs/src/quick-start/prerequisites.md` (2 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/assurance/safety-case.md`

- `claim-0034` | `docs/src/assurance/safety-case.md:3` | `high` | `descriptive` | expected: `code symbol,unit test` | This chapter argues why the architecture constrains split-brain risk under expected failure modes.
- `claim-0035` | `docs/src/assurance/safety-case.md:7` | `low` | `descriptive` | expected: `code symbol,unit test` | The system reduces the chance of concurrent primaries by coupling promotion and demotion behavior to trust, leader evidence, and explicit lifecycle guards.
- `claim-0036` | `docs/src/assurance/safety-case.md:11` | `low` | `descriptive` | expected: `code symbol,unit test` | - promotion is conditional, not automatic on leader absence alone
- `claim-0037` | `docs/src/assurance/safety-case.md:12` | `high` | `descriptive` | expected: `code symbol,unit test` | - conflicting leader evidence is treated as a safety signal
- `claim-0038` | `docs/src/assurance/safety-case.md:13` | `high` | `descriptive` | expected: `code symbol,unit test` | - fail-safe mode constrains actions when coordination confidence drops
- `claim-0039` | `docs/src/assurance/safety-case.md:14` | `low` | `descriptive` | expected: `code symbol,unit test` | - recovery paths (rewind/bootstrap) are explicit before rejoin
- `claim-0040` | `docs/src/assurance/safety-case.md:19` | `low` | `descriptive` | expected: `code symbol,unit test` | - etcd endpoints are configured correctly for the intended cluster

### `docs/src/quick-start/prerequisites.md`

- `claim-0164` | `docs/src/quick-start/prerequisites.md:7` | `low` | `descriptive` | expected: `real-binary e2e,runtime log evidence` | - PostgreSQL 16 binaries (`postgres`, `pg_ctl`, `pg_rewind`, `initdb`, `pg_basebackup`, `psql`)
- `claim-0165` | `docs/src/quick-start/prerequisites.md:8` | `low` | `descriptive` | expected: `real-binary e2e,runtime log evidence` | - etcd endpoint(s) reachable by the node
