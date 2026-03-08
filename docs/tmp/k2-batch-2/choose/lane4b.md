Target docs path: docs/src/how-to/add-cluster-node.md

Diataxis type: how-to

Why this is the next doc:
- Addresses cluster scaling, a common operational task not covered by existing documentation
- The bootstrap guide covers initial deployment but not dynamic node addition to a running cluster
- The HA test harness demonstrates dynamic node management, proving runtime support
- Provides practical steps for real-world goal (application of skill) distinct from initial setup
- Fills a clear gap in the How-to section between bootstrap and failure handling

Exact additional information needed:
- file: src/config/schema.rs
  why: Document all required configuration fields and validation rules specific to joining nodes
- file: src/dcs/state.rs
  why: Understand how member records are published and validated during join
- file: src/ha/decide.rs
  why: Explain how a new node transitions from init through waiting phases to replica
- file: docker/configs/cluster/node-a/runtime.toml
  why: Use as a minimal configuration template for new nodes
- file: tests/ha/support/multi_node.rs
  why: Observe the programmatic sequence for adding nodes in test scenarios
- extra info: What network ports and protocols must be open between nodes beyond DCS connectivity?

Optional runtime evidence to generate:
- command: etcdctl get --prefix /docker-cluster/member/ --print-value-only | jq .
  why: Capture exact DCS member record structure and leader lease format
- command: pgtuskmasterctl --base-url http://127.0.0.1:18081 ha state --output json
  why: Document the expected HA state before, during, and after node addition
