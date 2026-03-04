# slice-09

## Ownership

- `docs/src/lifecycle/switchover.md` (7 claims)
- `docs/src/contributors/harness-internals.md` (1 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/lifecycle/switchover.md`

- `claim-0084` | `docs/src/lifecycle/switchover.md:9` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | participant DCS as etcd / DCS
- `claim-0085` | `docs/src/lifecycle/switchover.md:11` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | participant Old as Current primary
- `claim-0086` | `docs/src/lifecycle/switchover.md:15` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | API->>DCS: write switchover intent
- `claim-0087` | `docs/src/lifecycle/switchover.md:16` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | HA->>DCS: observe intent and trust
- `claim-0088` | `docs/src/lifecycle/switchover.md:18` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | HA->>New: promote when lease and readiness allow
- `claim-0089` | `docs/src/lifecycle/switchover.md:19` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | HA->>DCS: clear switchover intent
- `claim-0090` | `docs/src/lifecycle/switchover.md:32` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | If a switchover stalls, treat it as a precondition failure, not immediately as a control-plane bug. Check trust posture, node readiness, and leader lease state first.

### `docs/src/contributors/harness-internals.md`

- `claim-0045` | `docs/src/contributors/harness-internals.md:8` | `low` | `descriptive` | expected: `code symbol` | - start and stop etcd clusters
