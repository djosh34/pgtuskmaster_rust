# Why process execution is isolated behind the process worker

The [process worker reference](../reference/process-worker.md) lists the job types, command runner surfaces, and lifecycle details. The architectural reason for that worker is straightforward: HA should express intent, but one dedicated place should own the risks of spawning and supervising local processes.

## The process worker is an execution membrane

The worker validates command specs, rejects non-absolute program paths, tracks active jobs, captures output when configured, drains stdout and stderr with bounded buffers, supports cancellation, and maps child termination into typed process outcomes.

Those are not side concerns. They are the difference between "the cluster wants this action" and "a real OS process is now running on this node with logs, timeouts, and failure modes".

## Why HA should not shell out directly

HA lowers decisions into effects and dispatches process actions through dedicated process-dispatch paths instead of spawning commands inline. That keeps cluster policy focused on coordination, trust, and recovery logic rather than on subprocess minutiae.

The payoff is clarity:

- HA reasons about desired behavior
- the process worker reasons about command execution
- logging and job-state publication stay attached to the place where execution actually happens

## Why observability belongs at the execution boundary

The worker emits events for request receipt, busy rejection, preflight failures, spawn failures, timeouts, exits, and output emission failures. That gives one consistent place to understand how requested work translated into real local activity.

If subprocess handling were scattered across HA and runtime code, those execution details would also be scattered. Centralizing them makes failure analysis much more direct.

## The tradeoff

The boundary adds indirection and asynchronous job handling. That is extra machinery. The project accepts it because the alternative is coupling HA policy to low-level process handling, which would make both harder to test and harder to debug.
