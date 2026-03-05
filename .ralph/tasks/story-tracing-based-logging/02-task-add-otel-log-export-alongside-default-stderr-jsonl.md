---
## Task: Add OTEL log export alongside default stderr JSONL output <status>not_started</status> <passes>false</passes>

<description>
Add OpenTelemetry log export support alongside the default stderr JSONL output, without forcing trace/span semantics onto the product.

The agent must explore the current logging pipeline, available Rust ecosystem support, and the runtime configuration surface first, then implement an operator-usable export path that keeps local structured logs working well.

Intended direction:
- default local behavior remains structured JSONL to stderr
- optional structured file output remains compatible with the same tracing-based pipeline
- the product can also export logs to an OTEL endpoint
- the log export design should fit the standard Rust tracing/OpenTelemetry ecosystem cleanly
- trace IDs and span IDs should not become mandatory product concepts if the current product does not need them

This task is about logs, not full tracing. If the implementation uses the tracing ecosystem internally, that is fine, but the operator-facing contract should stay focused on structured logs and log export rather than inventing distributed tracing concepts the product does not use.

The agent should use parallel subagents after exploration for exporter integration, config/runtime wiring, and verification.
</description>

<acceptance_criteria>
- [ ] Default stderr JSONL output remains supported and documented
- [ ] Optional structured file output remains supported and documented
- [ ] Logs can be exported to an OTEL endpoint through a supported runtime configuration path
- [ ] OTEL log export does not require the product to invent or depend on trace/span semantics it does not otherwise use
- [ ] Documentation explains the default local logging path and the OTEL export path clearly
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
