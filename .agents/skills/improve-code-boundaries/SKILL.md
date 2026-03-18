---
name: improve-code-boundaries
description: Identify one boundary problem where code lives in the wrong module or type layer, then flatten it by removing useless conversions, private-ing real-world DTOs, and collapsing request/bootstrap spaghetti into one shared source of truth.
---

# Improve Code Boundaries

Use this skill when the code works but the behavior lives in the wrong folder, wrong module, wrong type layer, or wrong bootstrap layer, and that wrong placement is making the code spaghetti.

The goal is narrow and strict:

- pick one single boundary issue
- untangle it fully
- reduce code
- remove transitions and wrappers that do not add real invariants

This skill is not for taste-only refactors. Use it when the current shape forces extra conversions, extra nesting, extra helper signatures, repeated validation, or A-talks-to-B-talks-back-to-A request plumbing.

## Core rule

Do not start by inventing a better abstraction.

Start by proving the current abstraction is useless.

The main workflow is:

1. Find one candidate.
2. Map the full tree around it:
   conversion functions, helper functions, wrapper structs, config-like structs, bootstrap/request types, and all users.
3. Temporarily comment out the converted-to type, wrapper, or transition layer.
4. Run `make check`.
5. Treat every compile failure as a question:
   what exact information did this caller need?
6. For each failure, decide:
   - If the needed fact is directly visible on the original type, match on the original type and remove the boundary.
   - If the needed fact is not directly visible but obviously belongs there, alter the shared root type once so it carries that fact.
   - If the current code has two drifting type trees, replace both with one new flatter shared type that encompasses both.
   - If the type is a real-world DTO or external payload, keep that DTO private or `pub(super)` and convert exactly once at ingestion.
7. Continue following the whole transition tree until the end users directly match on the flatter shared type and the transition helpers are gone.
8. Re-run `make check` after each major collapse.

In this repo, do not use `cargo test`. If you need focused test execution while developing, use `cargo nextest ...`. For this skill's discovery loop, `make check` is the primary "what breaks if I remove this boundary?" tool.

## What to look for

All of the following are concrete smell signals:

- type in type in type in type, for no invariant gain
- a module-local input type converted into another module-local output type with nearly the same cases
- input and output enums with the same number of entries
- a field set inside a match arm but always set to the same value anyway
- bools inside enums
- a two-value enum that is really a `None` or `Some` shape and should be `Option<T>`
- `Option<Option<T>>` or nested option-like states that should become one flatter enum
- more than three forward nestings before real work happens
- helper functions that require config-like values in their signature: paths, dirs, ports, hostnames, timings, durations, TLS pieces, max counts
- multiple config-like structs that mainly carry paths, ports, hostnames, timings, durations, TLS pieces, limits, or similar environment values
- request/bootstrap/context/channel/cadence structs that just rename fields and repackage the same data
- validation repeated after config has supposedly already been loaded
- direct use of serde/TOML shapes outside the config-ingestion boundary

When you see these, ask:

- Is this type adding a real invariant, or just renaming existing data?
- Could downstream code directly match on the original type?
- If not, is there exactly one missing fact that should be added to the original shared type?
- Is this really config input world, which should stay private and disappear after validation?
- Is the runtime acting like a courier for worker-internal knowledge?

## Special rules by smell

- Smell 1: useless overabstraction and overnesting
  Read [smell-1-overabstraction.md](smell-1-overabstraction.md).
- Smell 2: wrong config-ingestion boundary
  Read [smell-2-config-boundary.md](smell-2-config-boundary.md).
- Smell 3: wrong place-ism and request/bootstrap spaghetti
  Read [smell-3-wrong-placeism.md](smell-3-wrong-placeism.md).

## Smell 1 rules

Use smell 1 when a local type converts into another local type that adds little or no information.

Strong indicators:

- `PgInfoState`-like input converted into `PostgresState`-like output with the same broad cases
- wrapper structs such as request/bootstrap/context/channel/cadence layers that only rebundle fields
- helper signatures that take config or context values they do not truly own

When you remove the converted-to type and re-run `make check`, each failure tells you what the fake boundary was pretending to provide. Most of the time the answer is "nothing a direct match on the original type could not already provide."

If there is hybrid drift, do not give up. Replace both sides with one flatter type.

## Smell 2 rules

Use smell 2 when serde or TOML-shaped config leaks beyond the module that reads it.

The only allowed long-lived real-world DTOs are true external boundary shapes, such as direct `PgPollData`-style payloads or the raw `config.toml` document shape. Those boundary DTOs must stay private or `pub(super)`.

After ingestion:

- invalid state must be unrepresentable
- validation must not be repeated elsewhere
- no "missing" fields should remain optional if the internal program requires them
- no later code should care whether a value came from inline content, a file path, or an env var

If later code still validates, resolves, inherits, or normalizes config, the boundary is wrong.

## Smell 3 rules

Use smell 3 when module A asks module B to create a request that mostly mirrors what module A already knows, or when A talks to B talks back to A with similar types.

This is common when the runtime builds many `*Request`, `*Bootstrap`, or `*Ctx` structs and passes similar subscribers and handles around manually.

Preferred direction:

- one shared observed-state source of truth
- workers receive a subscriber to that shared state and only the handles they truly need
- validated config comes from `src/config`
- runtime stops knowing worker-internal field lists

## Hard constraints while using this skill

- Solve one boundary issue per run. Do not mix three unrelated cleanups.
- Aim for code reduction, not code movement alone.
- Follow all conversion edges until the transition helpers are actually removed.
- Do not stop after deleting one type if its call graph still contains the old shape in helper functions.
- Large refactors are allowed and often required. If 10-20 files need to change to flatten the boundary, that is a good sign, not a reason to stop.
- Do not preserve obviously bad legacy shapes for compatibility. This project is greenfield.
- Do not swallow errors while untangling. If you discover existing swallowed errors or anti-patterns outside the chosen boundary, create a bug task separately.

## Repo-specific examples to start from

These are strong candidates already present in this repository:

- `src/ha/startup.rs` plus `src/ha/state.rs`
  `HaRuntimeRequest -> HaWorkerBootstrap -> HaRuntimeCtx -> HaWorkerCtx`
- `src/ha/worker.rs`
  `PgInfoState -> PostgresState` plus larger `WorldView` nesting
- `src/cli/config.rs` plus `src/config/schema.rs` and `src/config/parser.rs`
  source-world config types and late normalization
- `src/runtime/node.rs`
  runtime builds one-off request structs for each worker

## Completion standard

You are done only when all of the following are true:

- the chosen boundary layer is flatter
- dead transition helpers are removed
- end users match on the shared type directly, or on one new flatter replacement type
- real-world DTOs are private if smell 2 was involved
- config-like values are no longer sprayed through signatures without ownership
- `make check` passes
