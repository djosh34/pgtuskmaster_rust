## Task: Delete `src/config/` And Prove Zero Config Dependencies Remain <status>not_started</status> <passes>false</passes>

<description>
Migrate the final code to src/config_v2, while not making ANYTHING new public inside src/config_v2/parser
All validation, in ALL code (must verify this), but only be done once, and that must be done only inside src/config_v2/parser.
All other validation functions must go, as you can encode that with non-optional rust types that cannot represent invalid states/configs.

When done you delete src/config, to validate that the full migration is complete.
</description>

<acceptance_criteria>
- [ ] `src/config/` is deleted
- [ ] `src/config_v2/parser` does NOT export any types
- [ ] `src/lib.rs` no longer declares or re-exports old config modules/types
- [ ] Repo-wide search finds zero code dependencies on `crate::config` or `pgtuskmaster_rust::config`
- [ ] Docs/examples/fixtures no longer describe old config paths or old config type names
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly 
</acceptance_criteria>
