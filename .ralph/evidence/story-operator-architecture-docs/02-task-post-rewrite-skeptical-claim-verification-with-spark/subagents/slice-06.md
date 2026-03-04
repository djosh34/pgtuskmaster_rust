# slice-06

## Ownership

- `docs/src/concepts/glossary.md` (8 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/concepts/glossary.md`

- `claim-0026` | `docs/src/concepts/glossary.md:3` | `low` | `descriptive` | expected: `code symbol` | - DCS: distributed configuration store used for coordination (etcd in this implementation).
- `claim-0027` | `docs/src/concepts/glossary.md:4` | `low` | `descriptive` | expected: `code symbol` | - Scope: namespace prefix in DCS keys, usually `/<scope>/...`.
- `claim-0028` | `docs/src/concepts/glossary.md:6` | `low` | `descriptive` | expected: `code symbol` | - Leader record: DCS record identifying current primary leadership ownership.
- `claim-0029` | `docs/src/concepts/glossary.md:7` | `low` | `descriptive` | expected: `code symbol` | - Switchover intent: operator request record for planned primary transition.
- `claim-0030` | `docs/src/concepts/glossary.md:9` | `high` | `descriptive` | expected: `code symbol` | - Fail-safe: conservative operating posture under degraded coordination trust.
- `claim-0031` | `docs/src/concepts/glossary.md:10` | `high` | `descriptive` | expected: `code symbol` | - Fencing: safety behavior that reduces split-brain risk when conflicting evidence appears.
- `claim-0032` | `docs/src/concepts/glossary.md:12` | `low` | `descriptive` | expected: `code symbol` | - Rewind: divergence-recovery path using `pg_rewind`.
- `claim-0033` | `docs/src/concepts/glossary.md:13` | `low` | `descriptive` | expected: `code symbol` | - Bootstrap recovery: re-clone path when rewind is unsafe or not possible.
