---
name: improve-code-boundaries
description: Identify one boundary problem where code lives in the wrong module, type layer, DTO layer, rendering layer, or bootstrap layer, then flatten it by removing useless conversions, duplicate shapes, stringly rendering, and request/bootstrap spaghetti.
---

# Improve Code Boundaries

Use this skill when the code works but the behavior lives in the wrong folder, wrong module, wrong type layer, or wrong bootstrap layer, and that wrong placement is making the code spaghetti.

The goal is narrow and strict:

- pick one single boundary issue
- untangle it fully
- reduce code
- remove transitions and wrappers that do not add real invariants

This skill is not for taste-only refactors. Use it when the current shape forces extra conversions, extra nesting, extra helper signatures, repeated validation, duplicate parse/render logic, premature string building, or A-talks-to-B-talks-back-to-A request plumbing.

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
- raw data kept as `String` and parsed later in another layer
- multiple manual renderers or parsers for the same concept
- CLI code building strings before it reaches one command-output `Display` boundary
- one concept stored twice in the same type, such as both a top-level field and a nested sub-struct field

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
- Smell 4: string-heavy rendering instead of one `Display` boundary
  Read [smell-4-display-not-strings.md](smell-4-display-not-strings.md).
- Smell 5: duplicate shapes and repeated parse/render functions
  Read [smell-5-shared-connection-shape.md](smell-5-shared-connection-shape.md).
- Smell 6: raw DTO data not converted once into one shared flat type
  Read [smell-6-raw-dto-boundary.md](smell-6-raw-dto-boundary.md).
- Smell 7: stop overengineering
  Read [smell-7-stop-overengineering.md](smell-7-stop-overengineering.md).

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
- defaulting must not be spread across `const` values and serde `default = ...` helper functions
- post-validation timing values must not remain raw counts like `*_ms` or `*_seconds`; they should be `Duration`

If later code still validates, resolves, inherits, or normalizes config, the boundary is wrong.

Hard rules for this repo's config boundary:

- `const` default values are a hard no for validated config modeling
- serde `#[serde(default = \"...\")]` helper-function defaults are also a hard no for validated config modeling
- instead, implement `Default` for the owning config type and convert from the raw DTO into the validated type there
- once config is validated, durations should be `std::time::Duration`, not raw integer counts

## Smell 3 rules

Use smell 3 when module A asks module B to create a request that mostly mirrors what module A already knows, or when A talks to B talks back to A with similar types.

This is common when the runtime builds many `*Request`, `*Bootstrap`, or `*Ctx` structs and passes similar subscribers and handles around manually.

Preferred direction:

- one shared observed-state source of truth
- workers receive a subscriber to that shared state and only the handles they truly need
- validated config comes from `src/config`
- runtime stops knowing worker-internal field lists

## Smell 4 rules

Use smell 4 when code starts manufacturing `String` values too early instead of carrying typed output until one final rendering boundary.

Preferred CLI shape:

- send command
- `pgtm` does the query
- receive the answer and parse JSON once with serde
- convert once into one command output enum or struct
- render that enum or struct directly with `Display`

If several helpers create strings before that final output boundary, the code is stringly and the boundary is wrong.

## Smell 5 rules

Use smell 5 when the same concept has more than one parser, more than one renderer, or more than one partially overlapping struct.

Strong indicators:

- many places format the same DSN or conninfo string manually
- one shared concept exists, but callers still build key-value strings by hand
- one type stores the same information twice
- one type allows impossible hybrid states instead of using a flatter enum

Preferred direction:

- one canonical shared type
- one parse function
- one render function or one `Display` impl
- all callers reuse that

## Smell 6 rules

Use smell 6 when raw non-config input data leaks past its ingestion boundary.

This is smell 2 for non-config DTOs:

- raw data type is private or `pub(super)`
- raw data is converted once into one shared enum or struct
- conversion is not split into several half-normalization steps
- downstream modules only see the shared validated flat type

If raw strings are still parsed later, or if several modules each "finish" part of the conversion, the boundary is wrong.

## Smell 7 rules

Use smell 7 when the code feels feature-heavy, state-heavy, timing-heavy, or generally more clever than the tests actually require.

Core rule:

- simpler solution is better
- simpler state machine is better
- fewer remembered facts are better
- fewer timing concepts are better
- the real question is "what is the minimum needed to keep `test-long` happy?"

This smell is often related to smell 1 and smell 3, but it is broader. Sometimes the code is in the right module and still overbuilt.

Preferred workflow:

1. remove a suspicious field, helper, branch, timing fact, or remembered feature
2. run `make check`
3. if behavior is at stake, run the narrowest relevant `cargo nextest ...`, and if this area is mainly proven by longer integration coverage, run `make test-long`
4. if everything still passes, you overengineered it, so keep it removed
5. if tests fail for a behavior that should matter but was not covered before, add the missing test and then re-simplify

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
- `src/command/mod.rs` plus `src/cli/output.rs`
  command output already has a `Display` boundary, but lower layers still build strings manually
- `src/pginfo/conninfo.rs` plus `src/command/mod.rs` and `src/postgres_managed.rs`
  one canonical conninfo type exists, but ad-hoc DSN rendering still appears elsewhere
- `src/pginfo/query.rs` plus `src/pginfo/state.rs`
  raw poll data crosses a boundary before being fully normalized
- `src/ha/types.rs` plus `src/ha/worker.rs` and `src/ha/reconcile.rs`
  timing and remembered-state bookkeeping may be carrying more behavior than the tests actually need

## Completion standard

You are done only when all of the following are true:

- the chosen boundary layer is flatter
- dead transition helpers are removed
- end users match on the shared type directly, or on one new flatter replacement type
- real-world DTOs are private if smell 2 was involved
- config-like values are no longer sprayed through signatures without ownership
- `make check` passes
