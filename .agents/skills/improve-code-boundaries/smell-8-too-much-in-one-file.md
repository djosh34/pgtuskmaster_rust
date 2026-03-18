# Smell 8: Too Much In One File

This smell is about a file that has become a coordination center for several unrelated responsibilities.

The bad version is not "the file is long." The bad version is:

- the file mixes workflows that do not need to know about each other
- helper types are being shared across unrelated concerns just because they happen to live nearby
- private structs and enums keep getting promoted to public or shared types without a real invariant
- splitting the file would reduce cross communication, but the current design keeps everything tangled together

The right response is usually to split the file into smaller files or modules, then keep the post-split interfaces narrow and mostly private.

## Detection checklist

Look for these signals:

- parsing, validation, orchestration, and rendering all live in one file
- one file has several local enums or structs that are only relevant to one narrow path
- functions keep handing the same intermediate data back and forth
- unrelated code blocks are only nearby because the file got large over time
- a split would let most helper types stay private instead of forcing them into a shared boundary
- the file has been "organized" into sections, but the sections still talk to each other too much

Ask these questions:

- Which pieces of this file actually need to talk?
- Which pieces only appear together because the file is too large?
- Which helper types can stay local after the split?
- Is the current shared type real, or just a bundle created to avoid creating another file?

## Required workflow

Use evidence, not instinct:

1. pick one file that is clearly doing too much
2. split off one cohesive responsibility into a separate file
3. keep the new file's private structs and enums private unless there is a real boundary to expose
4. follow other improvement steps to create shared type/boundary post new file creation
5. run `make check`
6. follow the compile failures to see which shared facts are actually needed
7. if the split caused unnecessary communication, adjust the boundary until the modules mostly stop talking to each other

The target shape is usually:

- one small public API per file
- one or more private helper types per file
- minimal cross-file calls
- no extra glue module that simply recombines what the split just separated

## Example A: one file with parser, runtime, and output logic

A common bad shape is a single file that does all of this:

- parses raw input
- normalizes it
- drives runtime behavior
- formats user-facing output

That file is not just long. It is acting as a mini application layer, a domain layer, and a presentation layer at once.

The fix is usually not "add more helper functions."

The fix is:

- move parsing into a parser module
- keep parser-only DTOs private to that module
- move runtime logic into a runtime module
- let output live behind one final display boundary

After the split, the modules should exchange the smallest possible validated type, not a giant shared bucket.

## Example B: one file with several private helper types that never needed to be shared

Suppose one file contains:

- a local state enum for one workflow
- a small helper struct for one sub-step
- a second helper enum for error handling
- a bunch of functions that only work on that local cluster

If another file needs only one final result, do not publish all of that internal machinery.

Prefer:

- keep the helper types private in the source file that owns them
- expose only the final result type or a narrow function
- let the other file depend on the result, not the internal stages

The point of the split is to reduce the number of things that can talk to each other, not to preserve every intermediate shape.

## Example C: a split that created too much glue

Sometimes the first split makes things worse:

- one module owns the data
- another module immediately rewraps the data
- a third module turns the wrapper back into the original shape

That is a sign the split boundary was wrong.

Do not add a manager type or a shared context type to paper over it.

Instead, ask whether:

- the shared fact belongs on the original type
- the helper types can stay private to one side
- the modules should be merged again because they are not actually separate concerns

## Decision rule

If splitting a file increases cross communication, the split was too artificial.

If splitting a file reveals that most helper types can stay private and the public surface gets smaller, the split was correct.

Keep the split only when it makes the code:

- smaller
- more local
- less coupled
- easier to reason about without a new abstraction layer
