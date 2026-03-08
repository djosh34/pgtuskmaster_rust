Target docs path: docs/src/how-to/remove-cluster-node.md
Diataxis type: how-to
Why this is the next doc:
- The How-To section lacks guidance for the inverse of "Add a Cluster Node"
- Node lifecycle management requires both addition and removal procedures
- Addresses a critical operational task for cluster maintenance and scaling down
- Builds directly on existing DCS trust and member record concepts already documented
- Prevents operational ambiguity about graceful vs non-graceful node removal

Exact additional information needed:
- file: src/dcs/store.rs
  why: To find DCS member deletion API methods, key removal patterns, and lease release calls
- file: src/dcs/keys.rs
  why: To document exact member record key patterns under /{scope}/member/{member_id} for cleanup
- file: src/ha/decide.rs
  why: To verify HA phase transitions when a member disappears from DCS cache
- file: src/process/jobs.rs
  why: To identify if a "fencing" or "shutdown" job exists for graceful node decommission
- file: tests/ha/support/multi_node.rs
  why: To see how test harness performs node teardown and DCS cleanup in scenarios

Optional runtime evidence to generate:
- command: docker compose -f docker/compose/docker-compose.cluster.yml stop node-c && sleep 5 && docker compose -f docker/compose/docker-compose.cluster.yml exec etcd etcdctl get / --prefix | grep docker-cluster/member/
  why: To capture DCS member record structure and observe automatic cleanup behavior
- command: for port in 18081 18082 18083; do curl -s http://127.0.0.1:$port/ha/state | jq '.member_count, .dcs_trust'; done
  why: To verify member count reduction and trust state after node removal
- command: grep -r "delete\|remove\|unregister" src/dcs/
  why: To find any existing DCS removal operations that could be exposed for operator use
