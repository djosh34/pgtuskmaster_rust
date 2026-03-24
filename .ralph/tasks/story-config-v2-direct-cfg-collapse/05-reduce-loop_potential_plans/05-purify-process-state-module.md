# Purify process state module

- Smell: `state.rs` holds pure state types, runtime bootstrap wiring, channel setup, and real filesystem startup prep in the same module.
- Files: `src/process/state.rs:37`, `src/process/state.rs:117`, `src/process/state.rs:159`, `src/process/state.rs:220`
- Collapse: keep only state/data types in `state.rs`; move worker bootstrap and `ensure_start_paths` into preparation/bootstrap code.
- Win: `state.rs` stops doing IO and stops acting as a misc bag of unrelated responsibilities.
