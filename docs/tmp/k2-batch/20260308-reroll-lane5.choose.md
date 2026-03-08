Target docs path: `docs/src/reference/runtime-configuration.md`
Diátaxis type: `reference`
Why this is the next doc:
- Critical gap: No reference exists for runtime configuration, which is essential for operating the system beyond the tutorial
- Complements existing CLI reference: Users need both CLI and config documentation
- Natural progression: After learning basic cluster setup, users must understand configuration options
- Evidence of planned need: Draft file exists at `docs/draft/docs/src/reference/runtime-configuration.md`
- Enables operational work: Required for customizing deployments, security, networking, and HA behavior

Exact additional information needed:
- file: `src/config/schema.rs` - why: contains the complete configuration structure, field types, and validation rules
- file: `src/config/defaults.rs` - why: provides all default values for optional configuration fields
- file: `docker/configs/cluster/node-a/runtime.toml` - why: shows production-like multi-node configuration example
- file: `docker/configs/cluster/node-b/runtime.toml` - why: shows variations and common patterns across nodes
- file: `docker/configs/cluster/node-c/runtime.toml` - why: demonstrates complete cluster configuration set
- file: `docker/configs/single/node-a/runtime.toml` - why: provides minimal/single-node configuration baseline
- file: `src/config/parser.rs` - why: reveals validation logic, required vs optional fields, and error handling

Optional runtime evidence to generate:
- command: `cargo run --bin pgtuskmaster -- --help` - why: shows runtime CLI flags that override or interact with config files
- command: `find docker/configs -name "*.toml" -exec echo "=== {} ===" \; -exec cat {} \;` - why: captures all configuration examples in the repository for pattern analysis
