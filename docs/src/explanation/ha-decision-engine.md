# HA Decision Engine

The HA loop is organized around four steps:

1. observe the local and global world
2. decide the desired local role and published authority
3. reconcile that desired state into ordered commands
4. execute those commands

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

## Ordered Commands Instead of Layered Public Views

The reconcile step turns the desired state into `ha.planned_commands`, an ordered list such as:

- publish authority
- acquire or release a lease
- start as primary or replica
- promote or demote
- rewind or base backup
- clear switchover intent

That same list is exposed in `/state`, so operators can see not only the current answer but also the next intended work.
