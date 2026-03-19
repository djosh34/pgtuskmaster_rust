# Smell 10: Remove The Damn Helpers

This smell is about behavior living in a helper function even though that helper has only one real caller.

The bad version is not "there are helper functions." The bad version is:

- the helper is only called once
- the helper exists only to hide a small local transformation from the place that actually owns it
- the helper name pretends a boundary exists, but the call graph proves it does not
- a file grows a pile of tiny private functions that fragment one workflow into artificial steps
- reading the caller requires jumping around the file even though the logic is not reused

Functions that are called once are `wrong_helpers`.

## Detection checklist

Look for these signals:

- one file has many small private functions
- most of those functions are only called from one place
- the helper takes only data that already exists in the caller
- the helper returns a shape that is immediately matched or forwarded by the caller
- deleting the helper would make one workflow easier to read top-to-bottom
- the helper does not create a reusable invariant shared across several callers

Do not guess based on names alone. Prove the call count.

## Required workflow

This smell has a strict proof method:

1. pick a file that clearly has too many helpers
2. choose one suspicious helper
3. rename it to `wrong_helper_test_<original_name>`
4. run `make check`
5. inspect the failures and count the callers
6. revert the rename
7. if there is only one real caller, inline the helper implementation into that caller
8. remove the helper
9. run `make check`
10. run `make test`

The rename step matters because it forces the compiler to show every call site instead of relying on a text search that might miss indirection or tests.

## Decision rule

If a helper has one caller, it is guilty until proven otherwise.

To keep the helper, you must be able to point to a real boundary such as:

- shared invariant enforcement across several callers
- one real abstraction that hides unavoidable complexity
- one reused operation that genuinely improves several sites at once

If the helper only exists to move a dozen lines out of the caller, remove it.

## How to inline correctly

When you inline a `wrong_helper`:

- move the full implementation into the only caller
- simplify the surrounding match or local variables while you are there
- remove any now-dead structs, enums, or helper calls that only existed for that helper
- keep pushing until the workflow reads in one place without bounce-around indirection

Do not stop after pasting the body. Usually the caller can be simplified further once the fake boundary is gone.

## Example A: one conversion helper used once

Bad shape:

```rust
fn build_status(raw: &RawThing) -> Status {
    match raw {
        RawThing::Ready(value) => Status::Ready(value.clone()),
        RawThing::Missing => Status::Missing,
    }
}

fn reconcile(raw: &RawThing) -> Plan {
    match build_status(raw) {
        Status::Ready(value) => Plan::Run(value),
        Status::Missing => Plan::Wait,
    }
}
```

If `build_status` is only called from `reconcile`, the helper is wrong. The match belongs in `reconcile` or both shapes should be replaced with one flatter shape.

Better:

```rust
fn reconcile(raw: &RawThing) -> Plan {
    match raw {
        RawThing::Ready(value) => Plan::Run(value.clone()),
        RawThing::Missing => Plan::Wait,
    }
}
```

The code is smaller, the workflow is local, and one fake boundary disappeared.

## Example B: a file full of tiny "step" helpers

Bad shape:

- `collect_x`
- `build_y`
- `plan_z`
- `finalize_q`

If each helper is called exactly once by the next helper, the file is not modular. It is just artificially fragmented.

The fix is usually:

1. start at the leaf helper with one caller
2. inline it
3. rerun `make check`
4. continue collapsing the chain until the real boundary appears

Many files get dramatically smaller once this chain starts collapsing.

## Bias

Prefer one readable match in the owning function over three tiny helpers that only exist to sound organized.

This project is greenfield. Do not preserve helper clutter out of habit.
