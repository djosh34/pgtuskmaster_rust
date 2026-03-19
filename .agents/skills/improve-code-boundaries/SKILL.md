---
name: improve-code-boundaries
description: Identify one boundary problem where code lives in the wrong module, type layer, DTO layer, rendering layer, or bootstrap layer, then flatten it by removing useless conversions, duplicate shapes, stringly rendering, and request/bootstrap spaghetti.
---

# Improve Code Boundaries

Use this skill when the code works but the behavior lives in the wrong folder, wrong module, wrong type layer, or wrong bootstrap layer, and that wrong placement is making the code spaghetti.

The goal is narrow and strict:

- first check `found_smells/`; if it contains a smell file, do that before picking anything else
- when that smell is fully done, remove the processed file from `found_smells/`
- pick one single smell
- find one single boundary issue for that smell (if you can't find any, switch smell)
- untangle it fully according to the smell instructions
- make check & make test

BE BOLD in your refactors: Large-scale code cleanups are encouraged:
- If you can remove entire type -> Great!
- If you can remove an entire file -> Greater!
- If you can remove/merge an entire mod/dir -> GREATEST!

## Found smells first

Before choosing from the standard smell list, inspect `found_smells/`.

- If there is a smell file in `found_smells/`, read it first and use that as the task for this run.
- Finish that smell completely before touching any other smell.
- After the work is complete, delete the processed file from `found_smells/`.

## Else, find new smells

- Read [how-to-create-found-smell.md](how-to-create-found-smell.md) on how to add smell
- Explore codebase (using subagents) to find new smells
- QUIT IMMEDIATELY

## Chose a smell

Chose one of the following are concrete smell signals:

- Smell 10: remove the damn helpers
  Read [smell-10-remove-the-damn-helpers.md](smell-10-remove-the-damn-helpers.md).
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
- Smell 8: too much in one file
  Read [smell-8-too-much-in-one-file.md](smell-8-too-much-in-one-file.md).
- Smell 9: typed error boundary, not string buckets
  Read [smell-9-typed-error-boundary.md](smell-9-typed-error-boundary.md).
- Smell 1: useless overabstraction and overnesting
  Read [smell-1-overabstraction.md](smell-1-overabstraction.md).

## Hard constraints while using this skill

- Solve one boundary issue per run
- End goal is code reduction and simplification. Less code is better, less structs/enums/mods etc is better
- Follow all conversion edges until the transition helpers are actually removed.
- Do not stop after deleting one type if its call graph still contains the old shape in helper functions.
- Large refactors are allowed and often required. If 10-20 files need to change to flatten the boundary, that is a good sign, not a reason to stop.
- Do not preserve obviously bad legacy shapes for compatibility. This project is greenfield.
