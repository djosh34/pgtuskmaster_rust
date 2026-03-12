# Verbose Context For `validating-cluster-behavior`

This tutorial target sits between the existing "observing failover" tutorial and the operator-facing how-to pages for handling failures.

The key teaching goal is not cluster setup and not incident response procedure. The teaching goal is validation:

- bring up a development HA cluster,
- trigger a concrete failure,
- watch the operator-visible surfaces change,
- confirm that PostgreSQL connectivity and replica behavior match the expected HA contract,
- and then repeat the observation loop for another fault pattern.

The most relevant current source material for this is the HA cucumber suite under `tests/ha/features/`.

Important current product behavior that should stay factually aligned with the tutorial:

- The operator-visible `pgtm primary` view now fails closed when sampling is too weak to trust a primary, but it still resolves a primary when there is a unique sampled primary with enough corroborating observation.
- Generic planned switchovers are now resolved to a specific target at request time, so a requested switchover should converge to one chosen replica instead of being re-resolved repeatedly by different nodes.
- A storage-stalled primary is kept fenced even after it drops the leader lease, so a stalled old primary should not drift back into ordinary no-lease behavior during failover.
- The HA validation harness now explicitly tracks proof-convergence blockers for `pg_basebackup` blockage and PostgreSQL-path isolation, which means failure scenarios intentionally testing lag or recovery do not incorrectly wait for full proof-row convergence too early.

For a tutorial, the most useful operator-visible observations are:

- `pgtm status --json` to inspect sampled members, roles, health, and warnings.
- `pgtm primary --tls --json` to confirm which node is the current writable primary from the operator perspective.
- `pgtm replicas --tls --json` to confirm which nodes are currently treated as replicas.
- proof-row checks through SQL to confirm that writes remain visible on the expected nodes before and after failover or recovery.

The HA cucumber scenarios show a few especially teachable validation patterns:

- Primary killed then rejoins as replica:
  This is the cleanest failover-and-rejoin exercise. It demonstrates a stable failover, a new primary becoming authoritative, and the old primary rejoining safely as a replica.
- Replica stopped while primary stays primary:
  This is a simpler "negative" validation. The important learning point is that not every fault should produce failover. The cluster should keep the same primary and keep serving writes while the stopped replica later rejoins.
- Planned switchover:
  This is the cleanest planned control-plane handoff exercise. It teaches users to distinguish a controlled role change from an unplanned failure response.

If K2 wants to make the tutorial narrower, the safest tutorial arc is:

1. Confirm a healthy cluster and identify the primary.
2. Record an initial proof row.
3. Stop a replica and validate that the primary does not change.
4. Restart the replica and confirm it rejoins and catches up.
5. Optionally repeat with a primary-failure scenario if K2 wants a second exercise section.

If K2 wants a richer tutorial, it can sequence two exercises:

1. A no-failover validation where a replica outage keeps the current primary stable.
2. A failover validation where the primary is killed and the old primary later rejoins as a replica.

The runtime evidence file `docs/tmp/ha-replica-stopped-primary-stays-primary-output.txt` comes from a passing ultra-long scenario run on the current tree and can be cited for exact step names and successful outcome text if K2 wants concrete phrasing grounded in the current harness behavior.
