---
## Task: Replace bespoke app logging with tracing-based structured logging <status>not_started</status> <passes>false</passes>

<description>
Replace the custom app-side logging stack with a more standard Rust structured logging architecture based on the `tracing` ecosystem, while preserving structured output and operator usefulness.

The agent must explore the current logging implementation, runtime integration points, and tests first, then design and implement a cleaner logging architecture. The intended direction is:
- stop hand-rolling logging patterns where the standard Rust ecosystem already solves them well
- remove the bespoke logging framework pieces such as the custom sink abstraction and custom fanout/file/stderr sink plumbing if they are only reimplementing standard `tracing` / `tracing-subscriber` capabilities
- keep only a thin initialization/bootstrap layer whose job is to configure the standard logging library, not to act as a homegrown logging framework
- keep structured logs as first-class output
- keep default stderr JSONL output
- support structured file output through the tracing ecosystem as part of the same logging pipeline
- avoid introducing trace/span requirements that the product does not currently need
- preserve or improve the existing usefulness of Postgres log collection where it is already working well
- route app logs, subprocess logs, and Postgres-ingested logs through one unified structured logging pipeline

The resulting logging behavior should support what operators actually need:
- a lot of debug logging where useful
- normal info logging where it matters
- warn/error/fatal style events where appropriate
- structured attribution of who produced the log and where it came from

That attribution should include, as appropriate:
- producer identity such as `pgtuskmaster`, `postgres`, `pg_ctl`, `pg_rewind`, or other managed subprocesses
- transport/source shape such as internal app event, child stdout/stderr, tailed postgres file, or parsed postgres JSON
- origin within the Rust codebase such as target/module/file/line or explicit subsystem fields

The Postgres log ingester is part of the desired end state. It should ingest both plain `.log` files and PostgreSQL JSON logs, and it should emit them into the same structured tracing-based pipeline with clear source attribution.

The agent should use parallel subagents after exploration for architecture review, code migration, and verification/test updates.
</description>

<acceptance_criteria>
- [ ] App-side structured logging is based on standard Rust `tracing` patterns rather than a bespoke sink stack where that is not justified
- [ ] Bespoke logging framework components are removed or reduced to a thin tracing initialization layer
- [ ] Default structured stderr JSONL output remains supported
- [ ] Structured file output is supported through the same tracing-based logging pipeline
- [ ] The resulting logging architecture is simpler and easier to justify than the current design
- [ ] Existing useful Postgres logging/collection behavior is preserved or cleanly integrated
- [ ] Postgres `.log` files and JSON logs are ingested into the same structured logging pipeline with producer/source attribution
- [ ] Logs carry enough structured metadata to identify the producing component and code/project origin where applicable
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
