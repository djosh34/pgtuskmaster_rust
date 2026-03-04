# slice-18

## Ownership

- `docs/src/start-here/solution.md` (4 claims)
- `docs/src/assurance/index.md` (2 claims)
- `docs/src/start-here/docs-map.md` (2 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/start-here/solution.md`

- `claim-0155` | `docs/src/start-here/solution.md:7` | `low` | `descriptive` | expected: `code symbol,real-binary e2e` | Observe[Observe PG + DCS state] --> Decide[Decide safest next phase]
- `claim-0156` | `docs/src/start-here/solution.md:14` | `low` | `descriptive` | expected: `code symbol,real-binary e2e` | - It observes local PostgreSQL state and shared DCS state.
- `claim-0157` | `docs/src/start-here/solution.md:26` | `high` | `invariant` | expected: `code symbol,real-binary e2e` | A loop-based controller can look cautious, because it revalidates instead of rushing actions. That caution adds decision latency in some cases, but it prevents many unsafe transitions that appear fast only because they skip verification.
- `claim-0158` | `docs/src/start-here/solution.md:30` | `low` | `descriptive` | expected: `code symbol,real-binary e2e` | During incidents, operators can reason about current behavior by asking three questions: what is the node observing, what decision did it make, and what action is blocked or running. That mental model maps directly to logs, API state, and DCS records.

### `docs/src/assurance/index.md`

- `claim-0001` | `docs/src/assurance/index.md:3` | `medium` | `behavioral` | expected: `code symbol,unit test` | This section is for readers who want confidence that behavior remains safe under stress, not only that normal workflows exist.
- `claim-0002` | `docs/src/assurance/index.md:8` | `low` | `descriptive` | expected: `code symbol,unit test` | - DCS data semantics and write ownership

### `docs/src/start-here/docs-map.md`

- `claim-0138` | `docs/src/start-here/docs-map.md:3` | `medium` | `behavioral` | expected: `code symbol,real-binary e2e` | This book is intentionally layered. Read the smallest layer that solves your immediate need, then move deeper only when you need more certainty.
- `claim-0139` | `docs/src/start-here/docs-map.md:14` | `low` | `descriptive` | expected: `code symbol,real-binary e2e` | | Section | Primary question it answers |
