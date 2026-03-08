# Why process execution is isolated behind the process worker

The process worker is a dedicated execution membrane that transforms high-level operational intent into safe, observable subprocess lifecycles. All command execution — whether for fencing, starting PostgreSQL, or other node-level operations — flows through this single boundary rather than allowing the HA (high-availability) state machine or node runtime to spawn processes directly.

## An execution membrane, not a thin wrapper

The worker enforces strict invariants before any binary runs. It builds absolute-path commands and rejects specifications that contain non-absolute program paths, preventing ambiguity about which executable runs. This validation occurs during a preflight stage that also selects job-specific timeouts via `timeout_for_kind` and applies dedicated checks for critical operations like fencing and `start-postgres` jobs. By centralizing these rules, the system ensures that policy decisions about “what may run” and “how long it may run” reside in one location, not scattered across the codebase.

Once a job is accepted, the worker becomes the sole owner of its lifecycle. It tracks active jobs in a registry, spawns the child process, and optionally captures stdout and stderr into bounded buffers that prevent memory exhaustion from verbose processes. The worker continuously drains output, handles cancellation signals, maps the child’s exit status to a typed `ProcessExit` result, and emits a stream of `ProcessEvent` values that distinguish every meaningful state: request receipt, busy rejection, preflight failure, spawn failure, timeout, graceful exit, and output emission failure.

## Safety and observability in one place

Concentrating subprocess execution behind this boundary improves safety. Bounded output draining, graceful shutdown coordination, and resource cleanup are implemented once and reused everywhere. Logging, metrics, and job state publication happen consistently because the worker is the only component that interacts with the OS process API.

Observability also becomes uniform. The event stream from the worker provides a complete auditable trace: when the HA layer requested an action, when the worker accepted or rejected it, what preflight checks ran, how the child behaved, and how it finished. This makes debugging systemic failures easier than correlating ad-hoc logs from multiple direct call sites.

## Reducing complexity in the HA state machine

The HA state machine expresses operational intent as effects — such as “fence this node” or “start PostgreSQL on this node” — but does not embed subprocess details. It delegates to `process_dispatch` and `apply_effect_plan`, which serialize the request, transmit it to the worker, and translate the resulting event stream back into state updates. This separation means:

- HA logic remains focused on consensus, timeouts, and transitions, not `fork(2)` or `execve(2)` semantics.
- Process execution can evolve (new preflight checks, different output limits, alternative isolation mechanisms) without changing the HA state chart.
- Testing HA behavior is simpler because the effect layer can be replaced with a deterministic simulator that reproduces the same `ProcessEvent` shapes.

## Tradeoffs

The boundary introduces indirection. Every process launch becomes an asynchronous job handled by a separate worker task, adding latency compared to a direct `std::process::Command` call. Job management, event routing, and bounded buffering also consume memory and CPU. These costs are accepted because they buy a unified policy enforcement layer and prevent HA logic from entangling with low-level execution hazards.

## Related concepts

- For the worker’s API and event shapes, see [Process worker](../reference/process-worker.md).
- For how HA translates decisions into effects, see [HA state machine](../explanation/ha-state-machine.md).
- For output handling and log integration, see [Logging and telemetry](../explanation/logging.md).
- For the runtime that hosts the worker, see [Node runtime](../reference/node-runtime.md).
