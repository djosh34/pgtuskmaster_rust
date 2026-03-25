## Smell Set: dcs-bootstrap-removal-and-worker-collapse <status>completed</status> <passes>true</passes>

Please refer to skill `improve-code-boundaries` to see what smells there are.

Inside dirs:
- `src/dcs`

Solve each smell:

---
- [x] Smell 3, wrong place-ism and bootstrap spaghetti: `src/dcs/startup.rs` must go fully
`src/dcs/startup.rs` is a bootstrap courier. Runtime disassembles `RuntimeConfig`, passes pieces into `bootstrap(...)`, `bootstrap(...)` reconstructs `DcsRuntimeCtx`, then returns `DcsRuntime { state, handle, worker }`, and `startup::run(...)` forwards directly into `worker::run(...)`.

This is exactly the smell-3 corridor:
- runtime knows DCS field soup
- startup restates DCS field soup
- `DcsRuntime` just wraps `{ state, handle, worker }`
- `startup::run` adds no invariant

The standalone bootstrap layer should be deleted. DCS should own its own composition boundary, and runtime should stop knowing DCS-internal context fields such as `member_ttl_ms`, `advertised_postgres`, `members`, `leadership`, `switchover`, and `last_emitted_authority`.

Mandatory completion threshold for this smell:
- `src/dcs/startup.rs` must disappear entirely, not merely shrink
- at least 70% of the listed bootstrap/courier types and wrappers must disappear entirely through deletion or merge
- more removal is better; 70% is only the minimum acceptable outcome
- if that threshold is not reached, the implementer must assume the bootstrap diagnosis was still correct and rethink the boundary

Types and wrappers that must overwhelmingly go away in this corridor:
- `DcsRuntime`
- `DcsRuntimeCtx` in its current broad courier shape
- `startup::bootstrap`
- `startup::run`

code:
```rust
// src/dcs/startup.rs
pub(crate) struct DcsRuntime {
    pub(crate) state: crate::state::StateSubscriber<DcsSnapshot>,
    pub(crate) handle: DcsHandle,
    pub(crate) worker: DcsRuntimeCtx,
}

pub(crate) async fn run(ctx: DcsRuntimeCtx) -> Result<(), WorkerError> {
    super::worker::run(ctx).await
}
```

```rust
// src/dcs/startup.rs
pub(crate) fn bootstrap(
    identity: NodeIdentity,
    cfg: &RuntimeConfig,
    pg_subscriber: StateSubscriber<PgInfoState>,
    log: LogSender,
) -> Result<DcsRuntime, DcsError> {
    let (publisher, state) = new_state_channel(DcsSnapshot::starting());
    let (handle, command_inbox) = dcs_command_channel();
    ...
    let ctx = DcsRuntimeCtx {
        identity,
        endpoints: cfg.dcs.endpoints.clone(),
        client: cfg.dcs.client.clone(),
        poll_interval: Duration::from_millis(cfg.ha.loop_interval_ms),
        member_ttl_ms: cfg.ha.lease_ttl_ms,
        advertised_postgres: PgEndpoint::tcp(...)?,
        pg: pg_subscriber,
        publisher,
        members: BTreeMap::new(),
        leadership: None,
        switchover: SwitchoverState::None,
        command_inbox,
        log,
        last_emitted_authority: None,
    };
    Ok(DcsRuntime { state, handle, worker: ctx })
}
```

```rust
// src/runtime/node.rs
let dcs =
    crate::dcs::startup::bootstrap(identity.clone(), &cfg, pginfo.state.clone(), log.clone())
        .map_err(|err| RuntimeError::Worker(format!("dcs store connect failed: {err}")))?;
```

---
- [x] Smell 10, remove the damn helpers: `src/dcs/worker.rs` is fragmented into a fake reusable API and must lose almost all of its helper functions
`src/dcs/worker.rs` currently has a single owning actor loop but it is cut into a large pile of private helper functions that mostly have one real caller. This is artificial decomposition, not real modularity.

The file currently spreads one workflow across all of these helpers:
- `connect_session`
- `sync_local_member`
- `handle_connected_command`
- `handle_disconnected_command`
- `acquire_local_leadership`
- `release_local_leadership`
- `publish_switchover`
- `clear_switchover`
- `refresh_leader_keepalive`
- `handle_connected_failure`
- `handle_leader_lease_expired`
- `handle_initial_connect_failure`
- `publish_current_view`
- `connected_failure_event`
- `initial_connect_failure_event`
- `load_snapshot`
- `apply_watch_response`
- `apply_key_value`
- `apply_delete`
- `parse_key`
- `scope_prefix`
- `member_path`
- `leader_path`
- `switchover_path`
- `ttl_seconds_from_ms`
- `leader_keepalive_interval`
- `build_connect_options`
- `timeout_etcd`
- `now_unix_millis`

That count is absurd for one worker file that owns one actor. This smell set should fail if the reduction is not drastic. The target is to collapse these into a handful of real artifacts:
- one actor loop
- one session-state artifact
- one key artifact
- one snapshot-apply path
- one small etcd utility boundary

Hard reduction target:
- at least 90% of the listed private worker helper functions must disappear entirely through inlining, merge, or collapse into a few owning artifacts
- more removal is better; 90% is only the minimum acceptable outcome
- do not stop after deleting only a few helpers; if 90% have not disappeared, the task is incomplete

code:
```rust
// src/dcs/worker.rs
pub(super) async fn run(mut ctx: DcsRuntimeCtx) -> Result<(), WorkerError> {
    let mut reconnect_at = Instant::now();
    let mut session = None::<ConnectedSession>;
    let mut tick = tokio::time::interval(ctx.poll_interval);
    ...
    loop {
        if let Some(connected) = session.as_mut() {
            ...
            let outcome = match step {
                ConnectedStep::Tick | ConnectedStep::PgChanged => { ... sync_local_member(...).await }
                ConnectedStep::Command(command) => { ... handle_connected_command(...).await ... }
                ConnectedStep::Watch(Ok(Some(response))) => apply_watch_response(...),
                ConnectedStep::KeepAlive => refresh_leader_keepalive(connected).await,
                ...
            };
            ...
            continue;
        }
        ...
        match step {
            DisconnectedStep::Reconnect => match connect_session(&mut ctx).await { ... }
            DisconnectedStep::Tick | DisconnectedStep::PgChanged => {}
            DisconnectedStep::Command(Some(command)) => {
                handle_disconnected_command(command);
            }
            ...
        }
    }
}
```

---
- [x] Smell 5, duplicate parse/render and repeated mutation paths in DCS key handling
The worker duplicates DCS key knowledge across parse helpers, render helpers, snapshot load, watch apply, delete apply, and write paths. That should become one reusable artifact, not five tiny functions and two mutation pipelines.

Specific duplication that should be collapsed into one `DcsKey`-style artifact with parse/render/apply support:
- `parse_key`
- `scope_prefix`
- `member_path`
- `leader_path`
- `switchover_path`
- `apply_key_value`
- `apply_delete`
- the write sites in `sync_local_member`, `acquire_local_leadership`, `publish_switchover`, and `clear_switchover`

The same domain facts are currently repeated:
- scope trimming and formatting
- mapping a path to `{member, leader, switchover}`
- deciding how the in-memory state changes for put/delete
- serializing payloads for those keys

Mandatory completion threshold for this smell:
- at least 70% of the listed key-shaping types/functions must disappear entirely through deletion or merge into one key artifact
- more removal is better; 70% is only the minimum acceptable outcome
- if that threshold is not reached, the implementer must assume the key-boundary diagnosis was still correct

code:
```rust
// src/dcs/worker.rs
enum KeyPath {
    Member(MemberId),
    Leader,
    Switchover,
}

fn parse_key(scope: &str, full_path: &str) -> Option<KeyPath> { ... }
fn scope_prefix(scope: &str) -> String { ... }
fn member_path(scope: &str, member_id: &MemberId) -> String { ... }
fn leader_path(scope: &str) -> String { ... }
fn switchover_path(scope: &str) -> String { ... }
```

```rust
// src/dcs/worker.rs
fn apply_key_value(...) -> Result<(), DcsError> {
    match parse_key(scope, path) {
        Some(KeyPath::Member(member_id)) => { ... }
        Some(KeyPath::Leader) => { ... }
        Some(KeyPath::Switchover) => { ... }
        None => {}
    }
    Ok(())
}

fn apply_delete(...) {
    match parse_key(scope, path) {
        Some(KeyPath::Member(member_id)) => { ... }
        Some(KeyPath::Leader) => { ... }
        Some(KeyPath::Switchover) => { ... }
        None => {}
    }
}
```

---
- [x] Smell 1, useless overabstraction and duplicated branching in session/error handling
There are multiple pairs of functions that do almost the same thing but split on phase names instead of real invariants:
- `connected_failure_event` and `initial_connect_failure_event`
- connected reconnect flow and disconnected reconnect flow inside `run`
- `handle_connected_failure` and `handle_initial_connect_failure`
- `load_snapshot` and `apply_watch_response` both decode etcd payloads into the same in-memory state

These are not good boundaries. They are stage-specific wrappers around the same DCS transition logic.

Required direction:
- introduce one explicit stage enum if the stage truly matters
- otherwise keep one error-to-log mapping function
- keep one state-application pipeline that can apply full snapshot items and watch events through the same code path

Mandatory completion threshold for this smell:
- at least 70% of the listed stage-split wrappers and duplicated branches must disappear entirely through deletion or merge
- more removal is better; 70% is only the minimum acceptable outcome
- if that threshold is not reached, the implementer must assume the branching is still overabstracted

code:
```rust
// src/dcs/worker.rs
fn connected_failure_event(err: &DcsError) -> DcsLogEvent { ... }
fn initial_connect_failure_event(err: &DcsError) -> DcsLogEvent { ... }
```

```rust
// src/dcs/worker.rs
async fn handle_connected_failure(
    ctx: &mut DcsRuntimeCtx,
    session: &mut ConnectedSession,
    err: &DcsError,
) -> Result<(), WorkerError> { ... }

fn handle_initial_connect_failure(
    ctx: &mut DcsRuntimeCtx,
    err: &DcsError,
) -> Result<(), WorkerError> { ... }
```

<plan>
- [x] Delete [`src/dcs/startup.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/startup.rs) entirely and move DCS-owned composition into the owning DCS boundary so [`src/runtime/node.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/runtime/node.rs) stops constructing DCS internals. Runtime should only receive the three externally needed artifacts directly: the DCS state subscriber, the DCS command handle, and the actor/worker to run.
- [x] Replace the current broad `DcsRuntimeCtx` courier in [`src/dcs/state.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/state.rs) with a flatter actor-owned shape that separates immutable runtime inputs from mutable cluster state. Keep only real ownership boundaries, for example: actor inputs (`identity`, DCS config/client inputs, pg subscriber, command inbox, log sender, publisher) and actor state (`members`, `leadership`, `switchover`, `last_emitted_authority`, optional live session). Do not introduce another wrapper that merely restates `{ state, handle, worker }`.
- [x] Introduce one scope-owned DCS key artifact inside [`src/dcs/worker.rs`](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/dcs/worker.rs) that owns prefix rendering/parsing plus record application. The same artifact should cover member/leader/switchover path rendering, snapshot/watch decode, and put/delete application so `parse_key`, `scope_prefix`, `member_path`, `leader_path`, `switchover_path`, `apply_key_value`, `apply_delete`, and the write sites disappear together instead of being replaced one-by-one.
- [x] Collapse the worker around one actor loop plus one real connected-session artifact. Inline the one-caller helpers into the loop or into the owning artifact, keep only a small etcd utility boundary for connect/TLS/timeout/lease timing, and replace the connected-vs-initial failure split with one explicit phase enum plus one error-to-log mapping function. `load_snapshot` and watch handling should feed the same apply pipeline rather than maintaining separate mutation paths.
- [x] After the type and ownership rewrite compiles, run the required validation gates in repo order: `make check`, `make lint`, `make test`, and `make test-long`. If execution shows the ADTs are still wrong, switch this task back to `TO BE VERIFIED`, explain the remaining design gap precisely in this file, and stop immediately.
NOW EXECUTE
</plan>

### Execution notes
1. Start with the boundary move, not helper cleanup: remove `startup.rs`, rewrite the runtime call site, and make DCS own its own channel/config/advertisement assembly before touching the worker internals.
2. Flatten types before repairing the full compile: the current `DcsRuntime` wrapper and broad `DcsRuntimeCtx` courier are the wrong shapes, so replace them first and then let compiler fallout drive the remaining call-site updates.
3. When collapsing helpers, prefer one readable match or one impl block over chains of tiny private functions. The only helpers that should survive are the small etcd utility boundary and any method that enforces a real session invariant across more than one caller.
4. Keep the key boundary typed. No new ad hoc string formatting/parsing helpers should survive outside the single DCS key artifact.
5. Do not preserve stage-specific wrappers just because the log event enum currently splits “initial connect” and “connected step” events. If the phase matters, model it once with a small enum and map from that.
