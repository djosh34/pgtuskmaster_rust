# Glossary

- DCS: distributed configuration store used for coordination (etcd in this implementation).
- Scope: namespace prefix in DCS keys, usually `/<scope>/...`.
- Member: one node identity participating in cluster coordination.
- Leader record: DCS record identifying current primary leadership ownership.
- Switchover intent: operator request record for planned primary transition.
- Trust: current confidence level in coordination data quality.
- Fail-safe: conservative operating posture under degraded coordination trust.
- Fencing: safety behavior that reduces split-brain risk when conflicting evidence appears.
- Bootstrap: initial data and role setup path at startup.
- Rewind: divergence-recovery path using `pg_rewind`.
- Bootstrap recovery: local reinitialization (`RunBootstrap` / `initdb`) when rewind is unsafe or fails; basebackup cloning is a separate startup path for joining an existing primary.
