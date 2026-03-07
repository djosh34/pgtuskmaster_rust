# Safety Invariants

Safety invariants are the conditions the system tries to preserve across lifecycle transitions. They are not promises that every outage will be short or that every transition will succeed quickly. They are the guardrails that explain why the implementation is willing to be conservative under ambiguity.

## Invariant 1: role changes depend on current evidence

The runtime is designed so promotion, demotion, recovery, and rejoin depend on the current combination of PostgreSQL state, DCS state, and trust posture rather than on stale assumptions. A data directory on disk, an old leader key, or a past observation that used to be true is not supposed to carry a node across a risky transition by itself.

This matters most at startup and recovery time. The runtime rebuilds managed PostgreSQL start intent, quarantines unmanaged local side effects that could silently bias startup, and reevaluates DCS evidence before it treats existing local state as safe to reuse. The consequence is that restart can feel stricter than operators expect, but that strictness is what prevents an old local picture from overriding the current cluster picture.

## Invariant 2: ambiguous leadership must not be answered with optimistic promotion

The design tries hard to separate "the old leader seems missing" from "it is safe for this node to promote." That distinction appears in failover logic, switchover sequencing, fencing behavior, and fail-safe posture. A node that merely stops seeing a leader is not supposed to treat absence alone as a promotion grant.

The practical consequence is slower progress under ambiguity. That is intentional. If a node promotes from weak evidence, the system may gain short-term availability at the cost of long-term trustworthiness. The invariant says that risk is usually not acceptable.

## Invariant 3: degraded trust constrains leadership behavior

DCS trust is part of the safety story, not a side metric. When coordination quality is degraded enough, normal promotion behavior is constrained and the lifecycle can enter `FailSafe` rather than continuing as if the shared view were still strong. This is the implementation's way of admitting that coordination evidence has lost enough value that ordinary leadership decisions would now be too speculative.

This invariant does not mean the node becomes silent when trust degrades. In fact, the API and tests are deliberately shaped so operators can still read `/ha/state` during no-quorum fail-safe situations. The invariant is about constrained action, not disappearance of observability.

## Invariant 4: conflicting leader evidence must trigger protective behavior

When trust remains strong enough to believe coordination data and a node that thinks it is primary sees conflicting leader evidence, the correct outcome is not "keep serving and hope the conflict clears." The design instead routes toward fencing or related demotion-oriented work. That is the safeguard against a node continuing primary behavior when the shared evidence says another leader may already exist.

Operationally, this can feel harsher than a human operator would choose in the moment, because the local PostgreSQL process may still look healthy. The invariant exists precisely because local health is not the same thing as cluster-safe primacy.

## Invariant 5: recovery must be explicit before rejoin

After failover, divergence, or certain startup failures, a node is not supposed to drift back into normal service simply because PostgreSQL can start. Rewind or base backup recovery must be explicit when the history situation demands it. Rejoin eligibility is therefore downstream of recovery evidence, not a default state.

This keeps a former leader or stale replica from reappearing as if it were already trustworthy. It also means a recovered node must still pass through the normal lifecycle checks again before operators should treat it as healthy.

## How to use these invariants

These invariants help separate protective strictness from an actual defect:

- if the system delays promotion because trust is degraded, that is usually policy
- if the system refuses to treat old local state as authoritative, that is usually policy
- if conflicting leader evidence pushes a node toward fencing, that is usually policy
- if the runtime acts as though those guardrails do not matter, that is where to suspect a real safety bug

The important discipline is to test observed behavior against the invariant that should explain it. When no invariant explains the behavior, or when the system appears to violate one without a stronger countervailing reason, that is the point where deeper investigation is warranted.
