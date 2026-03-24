# Smell 1: Useless Overabstraction And Overnesting

This smell is about local program state being projected into other local program state for no real gain.

The bad version is not "there are several types." The bad version is:

- the types mostly restate each other
- the wrapper chain adds nesting but not invariants
- helper functions exist mostly to move the same facts around
- downstream users could have matched on the original type directly
- helper methods or free functions only forward to the next helper and do not change behavior

Do not search for `build_` by name and stop there. Study what the functions actually do.
