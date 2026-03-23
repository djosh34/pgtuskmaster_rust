# Current Tasks Summary

Generated: Tue Mar 24 12:15:55 AM CET 2026

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/01-task-create-private-toml-schema-and-initial-runtimeconfigv2-root-handoff.md`

```
## Task: Create Private TOML Schema And Initial `RuntimeConfigV2` Root Handoff <status>completed</status> <passes>false</passes>

<description>
**Goal:** Create the first concrete execution task for the config-v2 direct-cfg collapse story. The higher-order goal is to start the migration by making `src/config_v2/parser/private_schema.rs` the only TOML-parsable config shape, adding parse functions in the config-v2 loaders, and switching `src/runtime/node.rs` to root itself in `RuntimeConfigV2` only. This task intentionally does not finish the downstream migration. It must stop at the first compile-failing handoff once the remaining failures are only due to the old `src/config/` corridor that later tasks in this story will delete.
```

==============

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/04-eliminate-tls-bytes.md`

```
## Task: Eliminate TLS bytes <status>not_started</status> <passes>false</passes>

<description>
Someone previously had this crazy idea to store bytes within the config struct. This causes a crazy amount of issues.
I don't want that at all. I don't want tls bytes to be inside the config struct, nor i ever want them to be written somewhere else.
```

==============

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/05-reduce-loop.md`

```
## Task: Reduce Code Loop <status>not_started</status> <passes>false</passes>

Your only goal is to reduce code and clean up the codebase.

use just-reduce-code skill
```

