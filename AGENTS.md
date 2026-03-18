Please quit immediately if you feel you are filling up your own context too much.

Never ignore the linter, the linters are there with good reason.
Skipping tests is one of the worst things you can do, giving extremely false confidence. Never skip a test, if something is missing in order to test -> fail.
Create bug immediately when spotted with add-bug skill.

We STRONGLY advice against using 'mut', and MOST of the time it can be replaced by pure and functional patterns.

When creating new enums/structs, always first verifiably search the codebase for overlaps and aim to alter that existing state in order to reuse it instead of creating a new one.
You should always aim to 'reduce code': 
 - remove unnecessary 'combine functions', replace with larger match against rust enums
 - remove unnecessary extra calls, or function calling one other function: that is a big code smell
 - when designing, always make sure to NOT do smells listed inside skill improve-code-boundaries, you must for each struct creation use that skill

Use explore agent often to not bloat context for when wanting to change something.
For large scale simple changes, spin up do_fast agent with ultra simple instructions

Also never swallow/ignore any errors. That is a huge anti-pattern, and must be reported as add-bug task.

This is greenfield project with 0 users. 
We don't have legacy at all. If you find any legacy code/docs, remove it.
No backwards compatibility allowed!
You are encouraged to make large refactors and schema changes
There are no 'versions', no v2/v1 configs, only the current version
Always aim for code reduction refactors when possible. Can I reuse the same types? Can I merge types with multiple uses?

Never run `cargo test` in this repo.
If you need a focused local test while developing, use `cargo nextest ...`, not `cargo test`.

## Cross application applicable learnings
- 
