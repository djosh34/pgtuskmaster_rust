# slice-03

## Ownership

- `docs/src/operator/deployment.md` (12 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/operator/deployment.md`

- `claim-0091` | `docs/src/operator/deployment.md:3` | `low` | `descriptive` | expected: `code symbol,unit test` | A standard deployment runs one `pgtuskmaster` process per PostgreSQL instance, with all nodes connected to a shared etcd cluster for coordination.
- `claim-0092` | `docs/src/operator/deployment.md:7` | `low` | `descriptive` | expected: `code symbol,unit test` | subgraph ETCD[etcd cluster]
- `claim-0093` | `docs/src/operator/deployment.md:8` | `low` | `descriptive` | expected: `code symbol,unit test` | E1[(etcd)]
- `claim-0094` | `docs/src/operator/deployment.md:9` | `low` | `descriptive` | expected: `code symbol,unit test` | E2[(etcd)]
- `claim-0095` | `docs/src/operator/deployment.md:10` | `low` | `descriptive` | expected: `code symbol,unit test` | E3[(etcd)]
- `claim-0096` | `docs/src/operator/deployment.md:31` | `low` | `descriptive` | expected: `code symbol,unit test` | A <-->|coordination| ETCD
- `claim-0097` | `docs/src/operator/deployment.md:32` | `low` | `descriptive` | expected: `code symbol,unit test` | B <-->|coordination| ETCD
- `claim-0098` | `docs/src/operator/deployment.md:33` | `low` | `descriptive` | expected: `code symbol,unit test` | C <-->|coordination| ETCD
- `claim-0099` | `docs/src/operator/deployment.md:38` | `low` | `descriptive` | expected: `code symbol,unit test` | This topology keeps data-plane and control-plane responsibilities clear. PostgreSQL stays local to each node. DCS is shared coordination memory, not a substitute for local database health.
- `claim-0100` | `docs/src/operator/deployment.md:42` | `low` | `descriptive` | expected: `code symbol,unit test` | A distributed control plane introduces dependence on etcd availability for full coordination trust. The benefit is explicit cluster-wide intent and leader visibility.
- `claim-0101` | `docs/src/operator/deployment.md:46` | `low` | `descriptive` | expected: `code symbol,unit test` | During network partitions or etcd instability, nodes may enter conservative states even if local PostgreSQL looks healthy. That behavior is expected and should be interpreted through trust state, not process count alone.
- `claim-0102` | `docs/src/operator/deployment.md:52` | `low` | `descriptive` | expected: `code symbol,unit test` | - Validate that each node can reach every configured etcd endpoint.
