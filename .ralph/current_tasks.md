# Current Tasks Summary

Generated: Fri Mar 20 03:30:15 PM CET 2026

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/01-task-create-private-toml-schema-and-initial-runtimeconfigv2-root-handoff.md`

```
## Task: Create Private TOML Schema And Initial `RuntimeConfigV2` Root Handoff <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Create the first concrete execution task for the config-v2 direct-cfg collapse story. The higher-order goal is to start the migration by making `src/config_v2/parser/private_schema.rs` the only TOML-parsable config shape, adding parse functions in the config-v2 loaders, and switching `src/runtime/node.rs` to root itself in `RuntimeConfigV2` only. This task intentionally does not finish the downstream migration. It must stop at the first compile-failing handoff once the remaining failures are only due to the old `src/config/` corridor that later tasks in this story will delete.
```

==============

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/02-task-follow-the-direct-cfg-reduction-loop-one-root-at-a-time.md`

```
## Task: Follow The Direct-`cfg` Reduction Loop One Root At A Time <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Define the exact reduction loop that every config-v2 migration task in this story must follow. The higher-order goal is to stop vague “migrate to RuntimeConfigV2” work and replace it with one strict invasive loop:
```

==============

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/10-task-rebuild-dev-support-and-tests-around-v2-config-only.md`

```
## Task: Rebuild `dev_support/` And Tests Around V2 Config Only <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Remove test/dev preservation of the old config tree by rebuilding helpers, builders, harnesses, and fixtures around `RuntimeConfigV2` and `OperatorConfigV2` only. The higher-order goal is to prevent tests from keeping `src/config/` alive after production code migrates.
```

==============

# Task `.ralph/tasks/story-config-v2-direct-cfg-collapse/11-task-delete-src-config-and-prove-zero-config-dependencies-remain.md`

```
## Task: Delete `src/config/` And Prove Zero Config Dependencies Remain <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Delete `src/config/` entirely and prove that no code, tests, docs, or fixtures depend on it anymore. The higher-order goal is to close the story with a hard architectural proof instead of stopping at a partial migration.
```

