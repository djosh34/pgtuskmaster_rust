# slice-16

## Ownership

- `docs/src/lifecycle/steady-state.md` (4 claims)
- `docs/src/quick-start/first-run.md` (3 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/lifecycle/steady-state.md`

- `claim-0103` | `docs/src/lifecycle/steady-state.md:3` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | After startup planning, the runtime enters continuous reconciliation. Each loop reevaluates local PostgreSQL state, DCS trust, and coordination records.
- `claim-0104` | `docs/src/lifecycle/steady-state.md:6` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | - one member acts as primary
- `claim-0105` | `docs/src/lifecycle/steady-state.md:7` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | - replicas follow the current leader
- `claim-0106` | `docs/src/lifecycle/steady-state.md:8` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | - leader lease remains current

### `docs/src/quick-start/first-run.md`

- `claim-0113` | `docs/src/quick-start/first-run.md:3` | `low` | `descriptive` | expected: `real-binary e2e,runtime log evidence` | This flow is a practical first launch. It validates the startup planner path, API availability, and basic DCS coordination behavior.
- `claim-0114` | `docs/src/quick-start/first-run.md:11` | `low` | `descriptive` | expected: `real-binary e2e,runtime log evidence` | Bring up etcd and verify reachability from the node host.
- `claim-0115` | `docs/src/quick-start/first-run.md:33` | `medium` | `behavioral` | expected: `real-binary e2e,runtime log evidence` | A first run is successful when the control loop is alive, not only when a process exists. API state confirms that workers are observing and publishing state instead of failing silently.
