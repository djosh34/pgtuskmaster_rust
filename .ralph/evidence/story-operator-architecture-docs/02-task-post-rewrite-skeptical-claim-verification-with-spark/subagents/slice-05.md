# slice-05

## Ownership

- `docs/src/lifecycle/failsafe-fencing.md` (9 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/lifecycle/failsafe-fencing.md`

- `claim-0068` | `docs/src/lifecycle/failsafe-fencing.md:1` | `high` | `descriptive` | expected: `real-binary e2e,code symbol` | # Fail-Safe and Fencing
- `claim-0069` | `docs/src/lifecycle/failsafe-fencing.md:3` | `high` | `descriptive` | expected: `real-binary e2e,code symbol` | Fail-safe and fencing are the lifecycle's safety brakes.
- `claim-0070` | `docs/src/lifecycle/failsafe-fencing.md:5` | `high` | `descriptive` | expected: `real-binary e2e,code symbol` | - Fail-safe: coordination trust is degraded enough that normal HA actions should be constrained.
- `claim-0071` | `docs/src/lifecycle/failsafe-fencing.md:6` | `high` | `descriptive` | expected: `real-binary e2e,code symbol` | - Fencing: conflicting leader evidence or unsafe primary conditions trigger demotion-oriented behavior to reduce split-brain risk.
- `claim-0072` | `docs/src/lifecycle/failsafe-fencing.md:10` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | Observe[Observe trust and leader evidence] --> TrustOK{Full trust?}
- `claim-0073` | `docs/src/lifecycle/failsafe-fencing.md:11` | `high` | `descriptive` | expected: `real-binary e2e,code symbol` | TrustOK -->|no| FS[Enter or remain fail-safe]
- `claim-0074` | `docs/src/lifecycle/failsafe-fencing.md:12` | `low` | `descriptive` | expected: `real-binary e2e,code symbol` | TrustOK -->|yes| Conflict{Conflicting leader evidence?}
- `claim-0075` | `docs/src/lifecycle/failsafe-fencing.md:13` | `high` | `descriptive` | expected: `real-binary e2e,code symbol` | Conflict -->|yes| Fence[Fencing or demotion path]
- `claim-0076` | `docs/src/lifecycle/failsafe-fencing.md:27` | `high` | `descriptive` | expected: `real-binary e2e,code symbol` | Operators should treat fail-safe as a meaningful status, not as noise. It indicates that coordination assumptions are currently insufficient for normal promotion behavior.
