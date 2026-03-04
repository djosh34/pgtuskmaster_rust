# slice-12

## Ownership

- `docs/src/start-here/problem.md` (5 claims)
- `docs/src/contributors/verification.md` (2 claims)
- `docs/src/interfaces/index.md` (1 claims)

## Instructions (skeptical, fail-closed)

- Treat docs and comments as untrusted; trust only code/tests/runtime evidence.
- For each claim below, output a row with:
  - `claim_id`
  - `status` one of: `verified`, `rewritten`, `removed`, `uncertain-with-followup`
  - `evidence_anchor` (file path + symbol and/or test name + line where possible)
  - `notes` (one sentence; if rewriting/removing, propose exact bounded replacement wording)
- Forbidden evidence: 'docs say so', 'it seems', second-hand summaries.

## Claims

### `docs/src/start-here/problem.md`

- `claim-0159` | `docs/src/start-here/problem.md:3` | `medium` | `behavioral` | expected: `code symbol,real-binary e2e` | PostgreSQL high availability fails in predictable ways when operations rely on ad-hoc scripts and implicit assumptions. The common failure mode is not only outage duration. The larger risk is unsafe role changes under partial information, where two nodes can believe they should accept writes.
- `claim-0160` | `docs/src/start-here/problem.md:5` | `low` | `descriptive` | expected: `code symbol,real-binary e2e` | This system exists to make role coordination explicit. It turns leader selection, switchover intent, and health observations into an ongoing control loop instead of a one-time manual decision. Operators get a repeatable mechanism for planned and unplanned transitions, with clear constraints when trust in shared coordination degrades.
- `claim-0161` | `docs/src/start-here/problem.md:9` | `medium` | `behavioral` | expected: `code symbol,real-binary e2e` | In a healthy cluster, operators want smooth transitions and fast recovery. In a degraded cluster, operators need the system to be conservative in exactly the right places. A design that optimizes only for speed can create data divergence that is more expensive than short unavailability.
- `claim-0162` | `docs/src/start-here/problem.md:13` | `high` | `behavioral` | expected: `code symbol,real-binary e2e` | The project intentionally trades maximum liveness for stronger safety under ambiguity. That means there are conditions where the system will decline promotion or demote aggressively. This behavior is deliberate. The alternative is optimistic progress under uncertain coordination, which increases split-brain risk.
- `claim-0163` | `docs/src/start-here/problem.md:17` | `high` | `behavioral` | expected: `code symbol,real-binary e2e` | This tradeoff is most visible during network instability, etcd disruption, and partial-cluster failures. In those moments, the correct operator expectation is not "always promote quickly." The correct expectation is "promote only when safety evidence is strong enough."

### `docs/src/contributors/verification.md`

- `claim-0049` | `docs/src/contributors/verification.md:11` | `low` | `descriptive` | expected: `code symbol` | - verify with primary evidence from implementation and tests
- `claim-0050` | `docs/src/contributors/verification.md:13` | `high` | `invariant` | expected: `code symbol` | - downgrade or remove claims that cannot be evidenced

### `docs/src/interfaces/index.md`

- `claim-0058` | `docs/src/interfaces/index.md:15` | `low` | `descriptive` | expected: `code symbol` | API --> DCS[(DCS intent records)]
