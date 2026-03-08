# Why the control API and debug API are separate

The project exposes two HTTP surfaces: a narrow control API and a verbose debug API. Each solves a different problem—one for steering the system, the other for inspecting it.

## Control intent versus system introspection

The control API exists to carry explicit operator intent: create a switchover, delete a scheduled operation, or read the current HA state. It exports only what policy requires. Every endpoint maps directly to a permissible action the e2e tests verify—`GET /ha/state` for observation, switchover endpoints for admin-initiated transitions—and nothing more. This keeps the surface small enough to reason about in policy and small enough to secure.

The debug API exists to make the system transparent to humans and automation. It builds `SystemSnapshot` values by pulling config, PostgreSQL info, DCS, process, and HA states into one coherent view. It tracks a monotonic sequence number, records domain-specific change events, and keeps a bounded timeline of the last 300 state changes. Where the control API answers “what can I do?” the debug API answers “what just happened and why?”

## Snapshots aggregate cross-worker state

No single worker owns the full picture. Configuration, PostgreSQL status, DCS contents, and the HA state machine each live in separate modules. The debug API worker gathers these pieces into a single `SystemSnapshot` so operators see a consistent cut across time. Without this aggregation, debugging would require correlating logs from four different domains by hand. The snapshot trades a small amount of latency for a large gain in clarity.

## Separation serves policy, debugging, and safety

Keeping the control surface narrow lets the e2e policy forbid direct internal steering after startup. The control API cannot accidentally expose a low-level “set HA phase” knob because that knob does not exist. Any change in HA state must follow the governed path—observation, switchover request, worker coordination.

Meanwhile, the debug API can evolve without breaking policy. Adding a new domain event or expanding the snapshot shape does not introduce a new control path. Engineers can iterate on observability without expanding the attack surface.

The split also prevents “debug” features from leaking into production clients. The public control API remains stable while the debug API emits verbose, unstable detail meant for internal dashboards and diagnostic tools.

Finally, the optional debug snapshot subscriber in the control API worker shows the relationship explicitly: the control layer may observe the debug stream to derive the HA state it exposes, but it never couples its endpoints to the internal representation. The two surfaces coexist, share data when helpful, and stay decoupled by design.
