## Learnings
- `make test-bdd` depends on a concrete integration target named `tests/bdd_state_watch.rs`; without that exact file, the command fails even if unit tests pass.
- For typed `tokio::sync::watch` state channels, explicit version-overflow handling in `publish()` keeps version semantics strict and testable.
