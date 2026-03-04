# slice-02

## Ownership

- `docs/src/operator/troubleshooting.md` (15 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/operator/troubleshooting.md`

- `claim-0140` | `docs/src/operator/troubleshooting.md:17` | `high` | `descriptive` | expected: `code symbol,unit test` | ## Node reports fail-safe unexpectedly
- `claim-0141` | `docs/src/operator/troubleshooting.md:20` | `low` | `descriptive` | expected: `code symbol,unit test` | - etcd endpoint instability
- `claim-0142` | `docs/src/operator/troubleshooting.md:22` | `low` | `descriptive` | expected: `code symbol,unit test` | - inconsistent membership/leader view
- `claim-0143` | `docs/src/operator/troubleshooting.md:25` | `low` | `descriptive` | expected: `code symbol,unit test` | - etcd endpoint health and latency
- `claim-0144` | `docs/src/operator/troubleshooting.md:26` | `low` | `descriptive` | expected: `code symbol,unit test` | - `[dcs].scope` consistency on all nodes
- `claim-0145` | `docs/src/operator/troubleshooting.md:27` | `low` | `descriptive` | expected: `code symbol,unit test` | - leader/member records in current scope
- `claim-0146` | `docs/src/operator/troubleshooting.md:33` | `low` | `descriptive` | expected: `code symbol,unit test` | - trust not at full quorum
- `claim-0147` | `docs/src/operator/troubleshooting.md:38` | `low` | `descriptive` | expected: `code symbol,unit test` | - DCS switchover intent visibility
- `claim-0148` | `docs/src/operator/troubleshooting.md:41` | `low` | `descriptive` | expected: `code symbol,unit test` | ## Rewind/bootstrap loops
- `claim-0149` | `docs/src/operator/troubleshooting.md:44` | `low` | `descriptive` | expected: `code symbol,unit test` | - rewind identity or privileges incorrect
- `claim-0150` | `docs/src/operator/troubleshooting.md:46` | `low` | `descriptive` | expected: `code symbol,unit test` | - source host/port for rewind is invalid
- `claim-0151` | `docs/src/operator/troubleshooting.md:54` | `low` | `descriptive` | expected: `code symbol,unit test` | ## Leader flaps or repeated role churn
- `claim-0152` | `docs/src/operator/troubleshooting.md:58` | `low` | `descriptive` | expected: `code symbol,unit test` | - unstable network to etcd
- `claim-0153` | `docs/src/operator/troubleshooting.md:63` | `low` | `descriptive` | expected: `code symbol,unit test` | - network path to etcd endpoints
- `claim-0154` | `docs/src/operator/troubleshooting.md:78` | `low` | `descriptive` | expected: `code symbol,unit test` | - [Architecture Assurance / DCS Data Model and Write Paths](../assurance/dcs-data-model.md)
