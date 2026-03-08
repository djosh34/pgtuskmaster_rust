Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Draft a new explanation page.

[Output path]
- docs/src/explanation/process-worker-boundary.md

[Page title]
- # Why process execution is isolated behind the process worker

[Audience]
- Engineers who want the reasoning behind a dedicated process worker instead of direct subprocess launches from HA or runtime code.

[User need]
- Understand why subprocess execution, preflight checks, timeouts, output draining, and job state publication live behind one worker boundary.

[mdBook context]
- Link naturally to process worker, HA state machine, logging, and node runtime reference pages.

[Diataxis guidance]
- Explanation only.

[Verified facts that are true]
- The process worker builds absolute-path commands and rejects invalid specs when the program path is not absolute.
- It tracks active jobs, captures output optionally, drains stdout and stderr with bounded buffers, supports cancellation, and maps child exit status to ProcessExit.
- The worker distinguishes request receipt, busy rejection, preflight failures, spawn failures, timeouts, exits, and output emission failures through process events.
- There are explicit preflight behaviors for fencing and start-postgres jobs and a timeout_for_kind helper.
- HA lowers decisions into effects and dispatches process actions through process_dispatch and apply_effect_plan rather than spawning commands directly.

[Relevant repo grounding]
- src/process/worker.rs
- src/ha/apply.rs

[Design tensions to explain]
- Why command execution is centralized.
- Why HA should express intent instead of owning subprocess details.
- Why logging and bounded output draining belong with process execution.
- Tradeoffs such as extra indirection and asynchronous job handling.

[Required structure]
- Explain the process worker as an execution membrane.
- Explain safety and observability reasons for the boundary.
- Explain how this reduces HA complexity.

[Facts that must not be invented or changed]
- Do not claim the process worker can run arbitrary relative-path commands.
- Do not claim HA directly shells out.
