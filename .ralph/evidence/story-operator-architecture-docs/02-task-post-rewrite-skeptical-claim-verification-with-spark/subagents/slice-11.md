# slice-11

## Ownership

- `docs/src/assurance/runtime-topology.md` (5 claims)
- `docs/src/contributors/codebase-map.md` (2 claims)
- `docs/src/contributors/testing-system.md` (1 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/assurance/runtime-topology.md`

- `claim-0003` | `docs/src/assurance/runtime-topology.md:3` | `low` | `descriptive` | expected: `code symbol,unit test` | A node contains multiple specialized workers with bounded responsibilities. The system boundary is local PostgreSQL management plus DCS coordination.
- `claim-0004` | `docs/src/assurance/runtime-topology.md:9` | `low` | `descriptive` | expected: `code symbol,unit test` | Dcs[DCS worker]
- `claim-0005` | `docs/src/assurance/runtime-topology.md:17` | `low` | `descriptive` | expected: `code symbol,unit test` | DCS[(etcd)] --> Dcs
- `claim-0006` | `docs/src/assurance/runtime-topology.md:18` | `low` | `descriptive` | expected: `code symbol,unit test` | Dcs --> Ha
- `claim-0007` | `docs/src/assurance/runtime-topology.md:21` | `low` | `descriptive` | expected: `code symbol,unit test` | Api --> DCS

### `docs/src/contributors/codebase-map.md`

- `claim-0046` | `docs/src/contributors/codebase-map.md:5` | `low` | `descriptive` | expected: `code symbol` | ## Primary runtime modules
- `claim-0047` | `docs/src/contributors/codebase-map.md:9` | `low` | `descriptive` | expected: `code symbol` | - `src/dcs`: coordination store integration, cache/trust handling, membership publication

### `docs/src/contributors/testing-system.md`

- `claim-0048` | `docs/src/contributors/testing-system.md:9` | `low` | `descriptive` | expected: `code symbol` | - Real-binary e2e tests: PostgreSQL and etcd process orchestration
