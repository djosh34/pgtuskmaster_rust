# What Problem This Solves

PostgreSQL high availability fails in predictable ways when operations rely on ad-hoc scripts and implicit assumptions. The common failure mode is not only outage duration. The larger risk is unsafe role changes under partial information, which can lead to concurrent primaries and divergent histories.

This implementation reduces that risk by funneling role changes through shared, DCS-trust-aware HA logic. Promotion is gated by current evidence instead of a one-time operator guess, and planned transitions flow through the same control path as unplanned ones.

## What changes for operators

Instead of treating leader selection as a manual procedure, `pgtuskmaster` turns it into a continuous control loop:

- local PostgreSQL state is observed directly
- shared coordination state is read from etcd
- operator intent is written into the same coordination model
- the node slows down or refuses promotion when the evidence is weak

In a healthy cluster that gives you repeatable transitions. In a degraded cluster it gives you explicit conservatism instead of silent optimism.

## What that means in practice

The project intentionally trades maximum liveness for stronger safety under ambiguity. During network instability, etcd disruption, or partial-cluster failures, the correct expectation is not "always promote quickly." The correct expectation is "promote only when safety evidence is strong enough."
