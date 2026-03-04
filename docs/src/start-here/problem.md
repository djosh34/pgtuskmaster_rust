# What Problem This Solves

PostgreSQL high availability fails in predictable ways when operations rely on ad-hoc scripts and implicit assumptions. The common failure mode is not only outage duration. The larger risk is unsafe role changes under partial information, which can lead to concurrent primaries.

This implementation reduces that risk by funneling role changes through shared DCS-trust-aware HA logic that gates promotion and by validating no-dual-primary windows in failure tests.

This system exists to make role coordination explicit. It turns leader selection, switchover intent, and health observations into an ongoing control loop instead of a one-time manual decision. Operators get a repeatable mechanism for planned and unplanned transitions, with clear constraints when trust in shared coordination degrades.

## Why this matters

In a healthy cluster, operators want smooth transitions and fast recovery. In a degraded cluster, operators need the system to be conservative in exactly the right places. A design that optimizes only for speed can create data divergence that is more expensive than short unavailability.

## Tradeoffs

The project intentionally trades maximum liveness for stronger safety under ambiguity. That means there are conditions where the system will decline promotion or demote aggressively. This behavior is deliberate. The alternative is optimistic progress under uncertain coordination, which increases split-brain risk.

## When this matters in operations

This tradeoff is most visible during network instability, etcd disruption, and partial-cluster failures. In those moments, the correct operator expectation is not "always promote quickly." The correct expectation is "promote only when safety evidence is strong enough."
