Target docs path: docs/src/how-to/perform-switchover.md
Diataxis type: how-to
Why this is the next doc:
- Builds directly on tutorial/first-ha-cluster.md (users have a running cluster)
- Logical next operational goal after checking cluster health
- Addresses a critical HA workflow that operators need to perform
- CLI and API support exists but only in reference form; lacks procedural guidance
- Can be validated against existing test scenarios

Exact additional information needed:
- file: src/cli/args.rs
  why: To capture exact CLI syntax for ha switchover request/clear commands and required parameters
- file: tests/ha_multi_node_failover.rs
  why: To understand switchover test patterns, expected sequence, and timing considerations
- file: src/api/controller.rs
  why: To confirm switchover endpoint behavior, request/response structure, and validation rules
- extra info: Are there any existing switchover test utilities or helper functions in tests/ha/support/ that demonstrate the expected operational sequence?

Optional runtime evidence to generate:
- command: cargo test --test ha_multi_node_failover -- --nocapture
  why: To capture actual switchover success/failure output and timing for realistic examples
