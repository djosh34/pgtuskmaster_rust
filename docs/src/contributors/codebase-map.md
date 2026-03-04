# Codebase Map

The codebase is organized around runtime responsibilities rather than broad utility layers.

## Primary runtime modules

- `src/runtime`: node bootstrap, startup planner orchestration, worker wiring
- `src/pginfo`: local PostgreSQL observation and state shaping
- `src/dcs`: coordination store integration, cache/trust handling, membership publication
- `src/ha`: lifecycle phases, decision logic, and action planning
- `src/process`: concrete PostgreSQL process and recovery action execution
- `src/api`: operator-facing control and state endpoints
- `src/debug_api`: debug snapshot and verbose state surfaces

## Supporting modules

- `src/config`: schema, parsing, validation
- `src/test_harness`: real-process fixtures and multi-node orchestration helpers
- `tests/`: integration and BDD-style external behavior checks

## Why this structure exists

The runtime is easier to reason about when each module owns one concern and publishes explicit state.

## Tradeoffs

Narrow modules reduce hidden coupling, but they require disciplined interfaces and richer shared state projections.
