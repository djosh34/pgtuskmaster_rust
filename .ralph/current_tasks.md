# Current Tasks Summary

Generated: Thu Mar 19 10:16:22 AM CET 2026

# Task `.ralph/tasks/bugs/write-convergence-health-check-race.md`

```
## Bug: Write convergence health check can observe one extra committed write after stopping workers <status>not_started</status> <passes>false</passes>

<description>
`make test` exposed a failure in `tests/ha/support/invariants/write_convergence.rs::one_primary_and_two_replicas_are_determined_healthy` where `ensure_healthy()` expected all members to converge to count `3` but observed `4` on every member instead.
```

