# Smell 10: Remove The Damn Helpers

This smell is about behavior living in a helper function even though that helper has only one real caller.

The bad version is not "there are helper functions." The bad version is:

- the helper is only called once
- the helper exists only to hide a small local transformation from the place that actually owns it
- the helper name pretends a boundary exists, but the call graph proves it does not
- a file grows a pile of tiny private functions that fragment one workflow into artificial steps
- reading the caller requires jumping around the file even though the logic is not reused

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

