# Safety Case

This chapter argues why the architecture constrains split-brain risk under expected failure modes.

## Claim

The system reduces the chance of concurrent primaries by coupling promotion and demotion behavior to trust, leader evidence, and explicit lifecycle guards.

## Supporting reasoning

- promotion is conditional, not automatic on leader absence alone
- conflicting leader evidence is treated as a safety signal
- fail-safe mode constrains actions when coordination confidence drops
- recovery paths (rewind/bootstrap) are explicit before rejoin

## Assumptions

- node clocks and network behavior remain within operationally realistic bounds
- etcd endpoints are configured correctly for the intended cluster
- PostgreSQL binaries and auth identities are correctly provisioned
- operators use documented control surfaces for planned transitions

## Residual risk

No distributed control system eliminates all risk. Extreme multi-fault conditions and operator misconfiguration remain risk factors. The architecture is designed to make these risks visible and to bias behavior toward conservative outcomes.

## Why this matters

A safety case turns architecture claims into explicit, reviewable arguments. It improves operator trust and contributor discipline.
