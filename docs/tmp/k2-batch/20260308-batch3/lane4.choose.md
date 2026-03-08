Target docs path: docs/src/explanation/failure-modes.md  
Diataxis type: explanation  
Why this is the next doc:  
- The architecture doc explains the happy-path design but does not detail failure scenarios  
- The trust model and HA phase machine imply complex failure modes that deserve explicit coverage  
- Operators need to understand "why" the system enters FailSafe, Fencing, or split-brain states  
- No existing doc covers etcd outages, network partitions, PostgreSQL crashes, or leader lease expiration behavior  
- The HA observer tests hint at split-brain detection but are not explained at the conceptual level  

Exact additional information needed:  
- file: src/ha/decide.rs  
  why: Need the exact phase transitions that trigger Fencing, FailSafe, and Rewinding  
- file: src/dcs/state.rs  
  why: Need the exact conditions that downgrade trust to NotTrusted or FailSafe  
- file: src/dcs/etcd_store.rs  
  why: Need behavior under etcd unavailability, leader election issues, and write failures  
- file: src/ha/decision.rs  
  why: Need the full set of decision variants related to fault handling  
- file: tests/ha/support/observer.rs  
  why: Need the logic used to detect split-brain in tests to inform the explanation  
- file: src/ha/actions.rs  
  why: Need the actions taken during fencing, step-down, lease release, and recovery  
- extra info: How does the system behave when the DCS is healthy but the local PostgreSQL instance is unreachable for extended periods? Does it self-fence, enter FailSafe, or wait indefinitely?  

Optional runtime evidence to generate:  
- command: cargo test --test ha_partition_isolation -- --nocapture  
  why: Would show actual partition behavior, phase transitions, and trust downgrades in isolation scenarios
