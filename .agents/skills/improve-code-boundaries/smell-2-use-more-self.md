# Smell 2: use more self

- many places have random ctx or functions that should really be owned by the struct
- refactor them fully to use self, instead of those functions.
  - This leads to less function args
  - This leads to better modules


## Example 1


