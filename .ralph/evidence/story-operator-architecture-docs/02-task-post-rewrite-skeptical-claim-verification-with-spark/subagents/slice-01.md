# slice-01

## Ownership

- `docs/src/operator/configuration.md` (18 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/operator/configuration.md`

- `claim-0116` | `docs/src/operator/configuration.md:3` | `medium` | `behavioral` | expected: `code symbol,unit test` | This chapter starts with one recommended production profile and then explains each section in detail. The goal is to make field choices meaningful, not only syntactically valid.
- `claim-0117` | `docs/src/operator/configuration.md:33` | `low` | `descriptive` | expected: `code symbol,unit test` | [dcs]
- `claim-0118` | `docs/src/operator/configuration.md:46` | `low` | `descriptive` | expected: `code symbol,unit test` | initdb = "/usr/pgsql-16/bin/initdb",
- `claim-0119` | `docs/src/operator/configuration.md:62` | `medium` | `behavioral` | expected: `code symbol,unit test` | Explicit configuration is more verbose than permissive auto-discovery. The benefit is deterministic behavior during failover, rewind, and startup planning. The cost is that operators must supply complete, correct field values.
- `claim-0120` | `docs/src/operator/configuration.md:74` | `high` | `invariant` | expected: `code symbol,unit test` | - PostgreSQL implication: prevents launching with incomplete auth or process wiring that would fail later.
- `claim-0121` | `docs/src/operator/configuration.md:79` | `low` | `descriptive` | expected: `code symbol,unit test` | - `member_id`: stable node identity in DCS membership records.
- `claim-0122` | `docs/src/operator/configuration.md:91` | `low` | `descriptive` | expected: `code symbol,unit test` | - `rewind_conn_identity` is used for rewind-related connectivity.
- `claim-0123` | `docs/src/operator/configuration.md:92` | `medium` | `behavioral` | expected: `code symbol,unit test` | - Operational effect: mismatched user identities will fail validation or later job execution.
- `claim-0124` | `docs/src/operator/configuration.md:93` | `medium` | `behavioral` | expected: `code symbol,unit test` | - PostgreSQL implication: rewind user privileges and auth paths must support `pg_rewind` safely.
- `claim-0125` | `docs/src/operator/configuration.md:98` | `low` | `descriptive` | expected: `code symbol,unit test` | - Operational effect: missing replication-compatible auth in `pg_hba` causes basebackup and replication connection failures.
- `claim-0126` | `docs/src/operator/configuration.md:101` | `low` | `descriptive` | expected: `code symbol,unit test` | ### `[dcs]`
- `claim-0127` | `docs/src/operator/configuration.md:103` | `low` | `descriptive` | expected: `code symbol,unit test` | - `endpoints`: etcd cluster URLs.
- `claim-0128` | `docs/src/operator/configuration.md:110` | `low` | `descriptive` | expected: `code symbol,unit test` | - `lease_ttl_ms`: leader lease freshness budget.
- `claim-0129` | `docs/src/operator/configuration.md:111` | `low` | `descriptive` | expected: `code symbol,unit test` | - Operational effect: shorter loops detect change faster but increase control-plane activity; TTL influences sensitivity to lease expiration and failover timing.
- `claim-0130` | `docs/src/operator/configuration.md:117` | `low` | `descriptive` | expected: `code symbol,unit test` | - PostgreSQL implication: rewind/bootstrap capability depends directly on correct binary wiring.
- `claim-0131` | `docs/src/operator/configuration.md:131` | `low` | `descriptive` | expected: `code symbol,unit test` | | Rewind jobs fail on permissions/auth | rewinder identity mismatch or privileges | `rewind_conn_identity` and role grants |
- `claim-0132` | `docs/src/operator/configuration.md:132` | `high` | `invariant` | expected: `code symbol,unit test` | | Node cannot find binaries | invalid `process.binaries` paths | absolute binary paths |
- `claim-0133` | `docs/src/operator/configuration.md:133` | `low` | `descriptive` | expected: `code symbol,unit test` | | Trust drops repeatedly despite healthy PostgreSQL | etcd endpoint or scope mismatch | `[dcs]` endpoints and `scope` |
