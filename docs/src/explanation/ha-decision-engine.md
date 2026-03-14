# HA Decision Engine

The HA loop is organized around four steps:

1. observe the local and global world
2. decide the desired local role and published authority
3. reconcile that desired state into a structured action plan
4. execute the planned work

You can see the current results in the `ha` section of `GET /state`.

## Two Different Questions

The HA engine produces two core outputs because operators and the local node need different answers:

- `ha.role`: what should this node do?
- `ha.publication.authority`: who should operators currently trust as primary?

That difference lets the runtime keep local work and public authority separate during lease-open, degraded-trust, or recovery windows.

## Trust Gates Normal HA

The first major branch is DCS trust.

When trust is not `full_quorum`, the node avoids ordinary leadership behavior:

- a local primary moves toward fail-safe behavior
- a replica keeps following or waits for quorum
- published authority moves away from `primary`

Safety wins over availability when the cluster view is not trustworthy enough.

## Structured Planned Actions

The reconcile step turns the desired state into `ha.planned_actions`, a structured read model with one optional field per action family:

- `publication` for authority updates
- `coordination` for lease and switchover work
- `local` for local maintenance work
- `process` for PostgreSQL/process intent

That same structured plan is exposed in `/state`, so operators can see not only the current answer but also the next intended work without depending on the old mixed execution enum.
