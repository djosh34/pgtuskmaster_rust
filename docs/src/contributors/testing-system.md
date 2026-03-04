# Testing System Deep Dive

The project verifies behavior at multiple layers so both local logic and real process behavior are covered.

## Test layers

- Unit tests: local logic and pure decision behavior
- Integration tests: cross-module behavior with controlled dependencies
- Real-binary e2e tests: PostgreSQL and etcd process orchestration
- BDD tests: external API/CLI behavior contracts

## Why this depth exists

HA systems can appear correct in unit tests while failing under real process timing and coordination conditions. Real-binary coverage closes that gap.

## Tradeoffs

Deep test coverage increases runtime and environment setup complexity. The benefit is stronger confidence in failure-path behavior that matters in production.

## Failure triage workflow

1. Reproduce in narrowest failing layer.
2. Confirm whether failure is deterministic or environment-dependent.
3. Map failure to lifecycle phase and trust state.
4. Validate with the closest realistic fixture before patching.
