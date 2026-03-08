Target docs path: docs/src/reference/runtime-configuration.md  
Diataxis type: reference  

Why this is the next doc:
- Zero documentation exists for runtime.toml configuration despite it being essential for deploying pgtuskmaster beyond the tutorial's Docker environment
- The reference section contains only CLI docs; daemon configuration is a critical gap
- Users cannot progress from tutorial to production deployment without understanding configuration options, their defaults, and validation rules

Exact additional information needed:
- file: src/config/schema.rs  
  why: Defines the Rust structs that are the source of truth for all configuration sections, fields, and their data types
- file: src/config/defaults.rs  
  why: Contains default values for every configuration parameter, essential for reference completeness  
- file: docker/configs/cluster/node-a/runtime.toml  
  why: Production-relevant example showing how configuration is organized in a realistic three-node deployment
- file: docker/configs/single/node-a/runtime.toml  
  why: Alternative example showing configuration for single-node scenarios, highlighting differences in scope and settings
