---
## Task: Replace bespoke app logging with tracing-based structured logging <status>not_started</status> <passes>false</passes>

<description>
Replace the custom app-side logging stack with a more standard Rust structured logging architecture based on the `tracing` ecosystem, while preserving structured output and operator usefulness.

The agent must explore the current logging implementation, runtime integration points, and tests first, then implement the following fixed product decisions:
- use the standard Rust `tracing` ecosystem for app-side structured logging
- remove the bespoke logging framework pieces that reimplement standard `tracing` / `tracing-subscriber` behavior, including the custom sink abstraction and custom sink fanout/file/stderr framework
- keep only a thin initialization/bootstrap layer whose job is to configure `tracing` output and exporters
- default local logging output is structured JSONL to stderr
- structured file output is supported through the same tracing-based pipeline
- app logs, subprocess logs, and Postgres-ingested logs all flow through one unified structured logging pipeline
- do not require trace IDs, span IDs, or distributed tracing concepts as part of the product contract
- preserve the useful Postgres log collection behavior and move it onto the same tracing-based pipeline
- the Postgres ingester must ingest both plain `.log` files and PostgreSQL JSON logs
- logs must carry structured attribution for producer and origin
- logs emitted by node-owned runtime code or node-owned ingest paths must include `member_id` as structured metadata when that identity is available

The resulting logging behavior should support what operators actually need:
- a lot of debug logging where useful
- normal info logging where it matters
- warn/error/fatal style events where appropriate
- structured attribution of who produced the log and where it came from

That attribution should include, as appropriate:
- producer identity such as `pgtuskmaster`, `postgres`, `pg_ctl`, `pg_rewind`, or other managed subprocesses
- transport/source shape such as internal app event, child stdout/stderr, tailed postgres file, or parsed postgres JSON
- origin within the Rust codebase such as target/module/file/line or explicit subsystem fields
- node identity such as `member_id` where the event belongs to a specific node context

The Postgres log ingester is part of the desired end state. It should ingest both plain `.log` files and PostgreSQL JSON logs, and it should emit them into the same structured tracing-based pipeline with clear source attribution.

The agent should use parallel subagents after exploration for architecture review, code migration, and verification/test updates.
</description>

<acceptance_criteria>
- [ ] App-side structured logging is based on standard Rust `tracing` patterns rather than a bespoke sink stack where that is not justified
- [ ] Bespoke logging framework components are removed and replaced by a thin tracing initialization layer
- [ ] Default structured stderr JSONL output remains supported
- [ ] Structured file output is supported through the same tracing-based logging pipeline
- [ ] The resulting logging architecture is simpler and easier to justify than the current design
- [ ] Existing useful Postgres logging/collection behavior is preserved or cleanly integrated
- [ ] Postgres `.log` files and JSON logs are ingested into the same structured logging pipeline with producer/source attribution
- [ ] Logs carry enough structured metadata to identify the producing component, node `member_id` where applicable, and code/project origin where applicable
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
</acceptance_criteria>
