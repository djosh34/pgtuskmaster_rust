# Smell 4: Display Boundary, Not String Soup

This smell is about creating `String` values too early.

The preferred shape is:

1. execute the command
2. parse raw response data once with serde
3. convert once into one command output enum or struct
4. render that type directly via `Display`

Any time you use format!, use must be very skeptical and question yourself: can this stay in the same type until later?


