# Delete ExternalToolLowerer

- Smell: `ExternalToolLowerer` mixes destructive filesystem prep, spec validation, config lookup, and argv rendering in one boundary-crossing type.
- Files: `src/process/tools.rs:17`, `src/process/tools.rs:21`, `src/process/tools.rs:62`, `src/process/tools.rs:237`
- Collapse: move disk prep out of command rendering and build `ProcessCommandSpec` from the single prepared action directly.
- Win: makes side effects explicit and removes a catch-all adapter object.
