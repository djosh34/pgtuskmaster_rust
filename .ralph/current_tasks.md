# Current Tasks Summary

Generated: Wed Mar 18 08:16:01 PM CET 2026

# Task `.ralph/tasks/bugs/bug-ha-long-runs-leak-docker-networks-and-exhaust-address-pools.md`

```
## Bug: HA long runs leak Docker networks and exhaust address pools <status>not_started</status> <passes>false</passes>

<description>
`make test-long` can fail before scenario execution because Docker cannot allocate another compose network:
```

==============

# Task `.ralph/tasks/bugs/bug-process-postmaster-reload-sighup-test-times-out-with-no-signal-log.md`

```
## Bug: process postmaster reload SIGHUP test times out with no signal log <status>not_started</status> <passes>false</passes>

<description>
`make test` currently fails in `process::postmaster::tests::reload_managed_postmaster_sends_sighup`.
```

==============

# Task `.ralph/tasks/bugs/bug-test-long-takes-far-too-long-with-zero-tests-passing-for-minutes.md`

```
## Bug: test-long takes far too long with zero tests passing for minutes <status>not_started</status> <passes>false</passes>

<description>
`make test-long` is taking far too long to show any passing HA tests.
```

==============

# Task `.ralph/tasks/story-improve-code-boundaries/01-task-find-one-code-boundary-smell-and-fix-it.md`

```
## Task: Find One Code Boundary Smell And Fix It <status>not_started</status> <passes>meta-task</passes>
NEVER TICK OFF THIS TASK. ALWAYS KEEP <passes>meta-task</passes>. This is a recurring deep verification task.

<description>
This is a **RECURRING META-TASK**. Every time this task is picked up, the engineer must do a **FRESH verification** pass. **NEVER set this task's passes to anything other than meta-task**.
```

