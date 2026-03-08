Target docs path: docs/src/reference/dcs-state-model.md
Diataxis type: reference
Why this is the next doc:
- DCS state is central to the trust model and HA decisions but lacks formal reference documentation
- Architecture.md explains concepts but operators need precise field definitions for debugging
- Debug API exposes DCS state but doesn't document the complete schema for member records, leader state, and switchover state
- Critical for understanding why nodes report degraded trust and for building tooling that interacts with etcd directly
- Fills a gap between conceptual explanation and API reference

Exact additional information needed:
- file: src/dcs/state.rs
  why: Contains DcsCache, DcsTrust, MemberRecord, LeaderState, SwitchoverState structures that define the state model
- file: src/dcs/keys.rs
  why: Shows how cluster scope, member IDs, and keys are organized in etcd paths
- file: src/dcs/worker.rs
  why: Reveals how state is published, freshness checks work, and trust evaluation uses member records
- file: docker/configs/cluster/node-a/runtime.toml
  why: Shows DCS endpoints and scope configuration that affect the state model
- extra info: What are all fields in MemberRecord including postgres_host, postgres_port, role, readiness, sql, timeline, wal positions? Which are required vs optional?

Optional runtime evidence to generate:
- command: docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml up -d --build && sleep 10 && curl -s http://127.0.0.1:18081/debug/verbose | jq '.dcs'
  why: To capture actual DCS state payload from running cluster to verify schema and field presence
