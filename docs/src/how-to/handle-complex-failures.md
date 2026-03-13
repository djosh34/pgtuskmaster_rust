# Handle Complex Failures

This guide covers failures that combine more than one problem at once: quorum loss, partial partitions, overlapping DCS and PostgreSQL faults, or a rejoin that does not converge cleanly.

## Goal

Determine:

- whether the cluster still has a trustworthy DCS view
- whether the seed node still publishes authoritative primary state
- whether the runtime is converging or stopped at a safety boundary

## Step 1: Compare multiple seeds

Run the same command from each reachable seed:

```bash
pgtm -c /etc/pgtuskmaster/node-a.toml status -v
pgtm -c /etc/pgtuskmaster/node-b.toml status -v
pgtm -c /etc/pgtuskmaster/node-c.toml status -v
```

Focus on:

- warnings
- `TRUST`
- `LEADER`
- `DECISION`

The CLI is intentionally seed-based now, so comparing seeds tells you whether the cluster view is converging.

## Step 2: Restore trust before authority

If a seed reports `degraded` or `not_trusted`, restore DCS health or member freshness first.

Do not try to bypass the system with manual promotions or ad hoc DCS edits while trust is degraded.

## Step 3: Inspect the raw state of one suspicious node

```bash
curl --fail --silent http://node-a:8080/state | jq .
```

Look at:

- `dcs.trust`
- `dcs.cache.leader_lease`
- `dcs.cache.member_slots`
- `ha.publication`
- `ha.role`
- `ha.world.global`
- `ha.planned_commands`
- `process`

## Step 4: Decide whether to wait or intervene

Keep watching when:

- trust is recovering
- warnings are decreasing
- one primary projection is emerging consistently

Investigate more deeply when:

- trust stays degraded
- seeds keep disagreeing on authority
- no seed publishes a primary after the expected lease window
- local process work repeats without convergence

## What Not to Do

During complex faults, do not:

- manually promote PostgreSQL
- delete DCS keys to "unstick" the system
- restart every node at once
- trust one seed if the others still disagree

The conservative answer is usually the correct answer.
