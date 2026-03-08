Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If required information is missing, say exactly what is missing.

You are writing prose for an mdBook documentation page.
Return only markdown for the page body requested.
Use ASCII punctuation only.
Do not use em dashes.
Do not invent facts.

[Task]
- Revise the supplied draft so it removes unsupported links and keeps only grounded claims.

[Output path]
- docs/src/explanation/process-worker-boundary.md

[Existing draft to revise]
- docs/tmp/k2_outputs/explanation-09-process-boundary.md

[Problems to fix]
- Remove the "Related concepts" section because it invents explanation-page links that do not exist.
- Keep links only to grounded reference pages that do exist, and only when they help.
- Keep the page in explanation form rather than turning it into a worker walkthrough.

[Verified facts that are true]
- The process worker builds absolute-path commands and rejects invalid specs when the program path is not absolute.
- It tracks active jobs, captures output optionally, drains stdout and stderr with bounded buffers, supports cancellation, and maps child exit status to ProcessExit.
- The worker distinguishes request receipt, busy rejection, preflight failures, spawn failures, timeouts, exits, and output emission failures through process events.
- There are explicit preflight behaviors for fencing and start-postgres jobs and a timeout_for_kind helper.
- HA lowers decisions into effects and dispatches process actions through process_dispatch and apply_effect_plan rather than spawning commands directly.

[Required structure]
- Explain the process worker as an execution membrane.
- Explain safety and observability reasons for the boundary.
- Explain how this reduces HA complexity.
- Discuss tradeoffs such as extra indirection and asynchronous job handling.
