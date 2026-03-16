## Task: Collapse Duplicate Struct Trees Into Canonical Domain ADTs And Prove The Struct Count Went Down <status>not_started</status> <passes>false</passes>

<priority>high</priority>
<blocked_by>Full completion of `.ralph/tasks/story-dcs-simplification/02-task-fully-rewrite-etcd-into-a-much-simpler-model-with-derive-support.md`</blocked_by>

<description>
**Goal:** Unify the duplicate struct and enum trees that currently describe the same domain concepts in API, CLI, DCS, HA, pginfo, process, config, and tests. The higher-order goal is to stop carrying multiple shapes for the same meaning, stop repacking the same fields through startup/bootstrap/runtime carrier structs, and force each domain concept onto one canonical ADT with one obvious owner. This task must reduce code, reduce conversion churn, reduce naming collisions, and make impossible states harder to represent.

**Mandatory execution order from the request:**
- First remove exact duplicates.
- Then introduce one canonical shape per domain concept.
- Then flatten the runtime/bootstrap carrier structs that only reshuffle the same fields.

**Non-negotiable completion rule from the request:**
- This task is not done unless the number of struct declarations under `src/` and `tests/` is verified to have gone down.
- You must prove that reduction with evidence. If the final verified struct count is unchanged or higher, the task is not complete, even if the code compiles.
- Renaming or moving duplicated structs without reducing the total number of struct declarations does not satisfy this task.

**Specific request that this task must preserve verbatim in spirit:**
- "create task to clean up duplicate structs in new story"
- "I would do this in three passes: first remove exact duplicates, then introduce one canonical shape per domain concept, then flatten the runtime/bootstrap carrier structs that only reshuffle the same fields."
- "all listed structs must be unified"
- "each to be deleted struct is one acceptance criteria"
- "each newly create struct is acceptance critera"
- "the task must be VERY clear that if the number of structs didn't go down (must be verified, you are not done)"

**Original general architectural request that this task must preserve:**
- "just like the dcs refactor task, i want a fully general improvement finding task"
- "make packages/mods more private"
- "reduce code interface between other components, make as small as possible interface"
- "find/checks/refactors radically internally to reduce code duplication. tries to simplify logic, de-spagthify, clean up old legacy logic/tests/shit"
- "untangle spagethi dependencies: just like dcs was controlled in many parts of the code, instead of a single worker. Find some other component that can be untangled, made almost fully private except very scoped/small interface, and thereby massively improving code quality, testability, reducing code in general (less code = better), cleaning up shit, making it more readable"

**Scope:**
- `src/api/controller.rs`
- `src/api/mod.rs`
- `src/api/startup.rs`
- `src/api/worker.rs`
- `src/cli/args.rs`
- `src/cli/client.rs`
- `src/cli/config.rs`
- `src/cli/connect.rs`
- `src/cli/status.rs`
- `src/config/schema.rs`
- `src/config/materialize.rs`
- `src/config/parser.rs`
- `src/dcs/mod.rs`
- `src/dcs/startup.rs`
- `src/dcs/state.rs`
- `src/dcs/worker.rs`
- `src/dev_support/auth.rs`
- `src/ha/decide.rs`
- `src/ha/state.rs`
- `src/ha/types.rs`
- `src/ha/worker.rs`
- `src/pginfo/conninfo.rs`
- `src/pginfo/state.rs`
- `src/postgres_roles.rs`
- `src/process/postmaster.rs`
- `src/process/state.rs`
- `src/process/worker.rs`
- `src/state/coordination.rs`
- `src/state/ids.rs`
- `src/state/net.rs`
- `tests/ha/support/observer/pgtm.rs`
- `tests/ha/support/runner/mod.rs`
- Any compile-fallout sites, tests, or module re-exports that still keep parallel duplicate shapes alive.

**Context from research:**
- There is a pure duplicate node-state DTO split between `NodeStateSnapshot` in `src/api/controller.rs` and `NodeState` in `src/api/mod.rs`.
- `NodeIdentity` already exists in `src/state/ids.rs`, but API, DCS, HA, pginfo, and process still carry smaller module-local identity fragments (`ApiClusterIdentity`, `DcsNodeIdentity`, `HaNodeIdentity`, `PgNodeIdentity`, `ProcessNodeIdentity`) that repeat the same concept.
- Switchover state is split across API, CLI args, CLI client, HA, DCS, and CLI status (`SwitchoverRequest`, `SwitchoverRequestArgs`, `SwitchoverRequestInput`, `SwitchoverState`, `SwitchoverTarget`, `SwitchoverView`, `ClusterSwitchoverView`, `SwitchoverRecord`) instead of being one canonical enum.
- Role-token carriers are duplicated across config, API runtime, CLI client, and dev support (`ResolvedApiRoleTokens`, `CliAuthConfig`, `ApiRoleTokensConfig`, `ApiRoleTokens`), and the surrounding auth envelope is duplicated too (`ApiAuthConfig`, `PgtmApiAuthConfig`, `ApiAuthState`).
- The mandatory Postgres role triplet is duplicated across config, role reconciliation, and process runtime (`MandatoryPostgresRolesConfig`, `MandatoryManagedRoleSet`, `MandatoryPostgresRuntimeRoles`).
- PostgreSQL connection and endpoint modeling is split across pginfo, CLI, and shared state (`PgConnInfo`, `PgLocalProbeTarget`, `ConnectionTarget`, `PgTcpTarget`, `PgUnixTarget`, `PgConnectTarget`), and at least one current type stores rendered DSN data alongside structured fields.
- DCS currently keeps a parallel record/view/cache tree (`DcsView`, `NotTrustedView`, `ClusterView`, `ClusterMemberView`, `LeadershipObservation`, `SwitchoverView`, `MemberLeaseRecord`, `MemberRecord`, `LeadershipRecord`, `SwitchoverRecord`, `DcsCache`) plus duplicate member-postgres shapes (`MemberPostgresView` and `MemberPostgresRecord`) to describe nearly the same cluster picture.
- API runtime wiring currently repacks the same fields through `ApiRuntimeRequest`, `ApiRuntime`, `ApiServer`, `ApiServerCtx`, `ApiControlPlane`, `ApiServingPlan`, and `ApiAppState`.
- DCS runtime wiring currently repacks the same fields through `DcsAdvertisedEndpoints`, `DcsRuntimeRequest`, `DcsEtcdConfig`, `DcsCadence`, `DcsLocalMemberAdvertisement`, `DcsObservedState`, `DcsStateChannel`, `DcsControlPlane`, `DcsRuntime` (`src/dcs/state.rs`), `DcsRuntime` (`src/dcs/startup.rs`), `DcsWorkerCtx`, and `DcsWorkerBootstrap`.
- There are three duplicate local `ChildGuard` structs in `src/api/worker.rs`, `src/process/postmaster.rs`, and `src/process/worker.rs`.
- There are duplicate test runner seed carriers in `tests/ha/support/observer/pgtm.rs` and `tests/ha/support/runner/mod.rs`.
- There is also a high-value naming collision that must be resolved while touching the same area: `ha::types::ProcessState` and `process::state::ProcessState` should not both survive under the same name. Even if both remain semantically necessary, the HA one must be renamed to something meaningfully different such as `ProcessAssessment` or `ProcessProjection`.
- Most of the new internal canonical carriers do not need `pub(crate)`. Default to private, then `pub(super)` if a parent module needs access, and only use wider visibility when the code genuinely crosses a larger boundary.
- The DCS redesign in the previous draft was still too wrapper-heavy and too tolerant of stale data. This task must remove `DcsMode` entirely and refactor HA first so it does not depend on `DcsMode` or on stale DCS state at all. If etcd is unavailable or quorum is lost, DCS must publish `NotTrusted` with no cluster data. There is no exception to this rule.
- Hanging runtime/config scalars such as `member_ttl_ms`, `poll_interval`, bind settings, and similar cadence/setting fields should not be threaded as loose fields through every runtime carrier. Reuse the existing config structs where they already model the setting, or introduce one dedicated settings/config ADT per domain if the existing config boundary is wrong.

**Required end-state properties:**
- One canonical `NodeIdentity`.
- One canonical node snapshot/read-model type.
- One canonical switchover ADT.
- One canonical role-token carrier.
- One canonical auth envelope ADT.
- One canonical fixed-role-slot carrier.
- One canonical PostgreSQL endpoint/connection ADT.
- One canonical DCS snapshot/read-model tree.
- One canonical API runtime context instead of multiple field-shuffle carriers.
- One canonical DCS runtime context instead of multiple field-shuffle carriers.
- One shared internal `ChildGuard`.
- Better names where the current ones encode layering instead of meaning.
- No stale DCS data is ever exposed or consumed outside quorum.
- The duplicate-type count goes down in a measurable, proven way.

**Important implementation rule about aliases:**
- If a deleted duplicate struct name still adds readability, it may survive only as a type alias onto the canonical ADT.
- It must not survive as its own separate struct declaration.
- The point is to reduce declarations and representations, not just redirect constructors.

**Required verification workflow for the struct-count reduction:**
- Before touching code, generate a baseline inventory with the local extractor at `tools/type_inventory_gen/` and save it under `.ralph/evidence/duplicate-struct-cleanup/type_inventory.before.txt`.
- After the refactor, generate `.ralph/evidence/duplicate-struct-cleanup/type_inventory.after.txt`.
- Use those evidence files, or a stricter improved version of the same tooling, to prove that the number of `struct` declarations under `src/` and `tests/` is lower after the refactor.
- If the inventory tool is not precise enough for the struct-only count, improve the tool as part of this task and keep the before/after evidence in the same evidence directory.
- The task is not complete until that before/after reduction is written down in the task notes or other committed evidence.

**Canonical shapes to land in this task:**

**Delete checklist for `NodeSnapshot`:**
- `NodeStateSnapshot` in `src/api/controller.rs`
- `NodeState` in `src/api/mod.rs`

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeSnapshot {
    #[serde(flatten)]
    pub identity: NodeIdentity,
    pub pg: PgInfoState,
    pub process: crate::process::state::ProcessState,
    pub dcs: DcsSnapshot,
    pub ha: HaState,
}
```

`NodeIdentity` already exists and should be the single identity ADT used by API, DCS, HA, pginfo, and process:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NodeIdentity {
    pub cluster_name: ClusterName,
    pub scope: ScopeName,
    pub member_id: MemberId,
}
```

**Delete checklist for switchover unification:**
- `SwitchoverRequest` in `src/api/controller.rs`
- `SwitchoverRequestArgs` in `src/cli/args.rs`
- `SwitchoverRequestInput` in `src/cli/client.rs`
- `SwitchoverRequest` in `src/ha/types.rs`
- `SwitchoverState` in `src/ha/types.rs`
- `SwitchoverTarget` in `src/state/coordination.rs`
- `SwitchoverView` in `src/dcs/state.rs`
- `ClusterSwitchoverView` in `src/cli/status.rs`
- `SwitchoverRecord` in `src/dcs/state.rs`

For switchover, use one canonical enum that represents both "no pending switchover" and the requested target. Do not keep a state enum wrapping a request struct wrapping a target enum:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwitchoverState {
    None,
    AnyHealthyReplica,
    Specific(MemberId),
}
```

This should replace all of `SwitchoverRequest`, `SwitchoverTarget`, `SwitchoverView`, and the current wrapper-style `SwitchoverState`.

**Delete checklist for auth/token unification:**
- `ResolvedApiRoleTokens` in `src/api/worker.rs`
- `CliAuthConfig` in `src/cli/client.rs`
- `ApiRoleTokensConfig` in `src/config/schema.rs`
- `ApiRoleTokens` in `src/dev_support/auth.rs`
- `ApiAuthConfig` in `src/config/schema.rs`
- `PgtmApiAuthConfig` in `src/config/schema.rs`
- `ApiAuthState` in `src/api/worker.rs`

Keep the token pair concrete, not generic. Do not introduce a generic optional-secret wrapper here.

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SecretSource {
    None,
    Env { env: String },
    File { path: std::path::PathBuf },
    String { value: String },
}
```

The existing `SecretSource` enum in `src/config/schema.rs` should be rewritten to this simpler canonical shape. Do not keep `Path`, `PathConfig`, and `Inline` plus separate optional wrappers.

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleTokens {
    pub read_token: SecretSource,
    pub admin_token: SecretSource,
}
```

Unify the auth envelope too, not just the inner pair of tokens:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "tokens")]
pub enum TokenAuth {
    Disabled,
    RoleTokens(RoleTokens),
}
```

One acceptable end-state is:
- config/runtime/CLI all model token sources as `TokenAuth`
- resolved token materialization happens only at the last possible use site
- dev/test fixtures use `SecretSource::String`

**Delete checklist for mandatory-role-slot unification:**
- `MandatoryPostgresRolesConfig` in `src/config/schema.rs`
- `MandatoryManagedRoleSet` in `src/postgres_roles.rs`
- `MandatoryPostgresRuntimeRoles` in `src/process/state.rs`

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresRoleSlots<T> {
    pub superuser: T,
    pub replicator: T,
    pub rewinder: T,
}
```

The most direct use is:

```rust
pub type MandatoryPostgresRolesConfig = PostgresRoleSlots<PostgresRoleConfig>;
pub type MandatoryManagedRoleSet = PostgresRoleSlots<ManagedRoleSpec>;
pub type MandatoryPostgresRuntimeRoles =
    PostgresRoleSlots<MandatoryPostgresRoleCredential>;
```

If alias ergonomics or serde bounds make that exact spelling awkward, another dedicated generic slot ADT is acceptable. What is not acceptable is keeping three same-shape structs.

**Delete checklist for PostgreSQL endpoint/access unification:**
- `PgConnInfo` in `src/pginfo/conninfo.rs`
- `PgLocalProbeTarget` in `src/pginfo/state.rs`
- `ConnectionTarget` in `src/cli/connect.rs`
- `PgTcpTarget` in `src/state/net.rs`
- `PgUnixTarget` in `src/state/net.rs`
- `PgConnectTarget` in `src/state/net.rs`

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PgEndpoint {
    Tcp { host: String, port: u16 },
    UnixSocket {
        socket_dir: std::path::PathBuf,
        port: u16,
    },
}
```

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgClientTls {
    pub mode: PgSslMode,
    pub root_cert: Option<std::path::PathBuf>,
    pub client_cert: Option<std::path::PathBuf>,
    pub client_key: Option<SecretSource>,
}
```

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgConnInfo {
    pub endpoint: PgEndpoint,
    pub user: String,
    pub tls: PgClientTls,
}
```

Do not keep `member_id`, `dbname`, `application_name`, `connect_timeout_s`, or opaque `options` inside the canonical connection-info ADT unless current repo usage proves they are actually required. In the current codebase, those look like derived policy/settings concerns, not core connection identity.

Hard requirement: fields removed from the canonical connection-info ADT must move into the owning config structs or one dedicated PostgreSQL client-settings/config ADT, whichever gives the smallest truthful boundary. They must not reappear as loose parameters threaded through clunky `build_*` helpers.

The canonical `PgConnInfo` should own DSN conversion behavior through impls on the type itself rather than through free helper functions:

```rust
impl std::fmt::Display for PgConnInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // render the supported DSN subset from the canonical ADT
    }
}

impl std::str::FromStr for PgConnInfo {
    type Err = PgConnInfoParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        // parse the supported DSN subset into the canonical ADT
    }
}
```

Use `to_string()` via `Display` for DSN rendering and `FromStr` for parsing. Do not store a parallel rendered DSN field, and do not introduce a new free `render_*dsn` / `parse_*dsn` layer if the behavior belongs on the canonical type.

**Delete checklist for DCS snapshot unification:**
- `DcsMode` in `src/dcs/state.rs`
- `DcsView` in `src/dcs/state.rs`
- `NotTrustedView` in `src/dcs/state.rs`
- `ClusterView` in `src/dcs/state.rs`
- `ClusterMemberView` in `src/dcs/state.rs`
- `LeadershipObservation` in `src/dcs/state.rs`
- `SwitchoverView` in `src/dcs/state.rs`
- `MemberLeaseRecord` in `src/dcs/state.rs`
- `MemberRecord` in `src/dcs/state.rs`
- `LeadershipRecord` in `src/dcs/state.rs`
- `SwitchoverRecord` in `src/dcs/state.rs`
- `DcsCache` in `src/dcs/state.rs`
- `MemberPostgresView` in `src/dcs/state.rs`
- `MemberPostgresRecord` in `src/dcs/state.rs`

Do not invent a second DCS-specific Postgres enum if `PgInfoState` already carries the meaning. Prefer using `PgInfoState` directly.

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DcsMemberState {
    pub postgres_endpoint: PgEndpoint,
    pub postgres: PgInfoState,
}
```

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DcsSnapshot {
    NotTrusted,
    Quorum {
        leadership: Option<LeaseEpoch>,
        switchover: SwitchoverState,
        members: std::collections::BTreeMap<MemberId, DcsMemberState>,
    },
}
```

Hard requirement: `DcsSnapshot::Quorum { .. }` is always fresh quorum-backed data. `DcsSnapshot::NotTrusted` carries no stale members, no stale leadership, and no stale switchover. Refactor the HA loop first so it never consumes DCS data unless it received `Quorum { .. }`. There is no exception to this rule.

**Delete checklist for API runtime-carrier unification:**
- `ApiRuntimeRequest` in `src/api/startup.rs`
- `ApiRuntime` in `src/api/startup.rs`
- `ApiServer` in `src/api/startup.rs`
- `ApiServerCtx` in `src/api/worker.rs`
- `ApiControlPlane` in `src/api/worker.rs`
- `ApiServingPlan` in `src/api/worker.rs`
- `ApiAppState` in `src/api/worker.rs`

```rust
#[derive(Clone)]
pub(super) struct ApiRuntimeCtx {
    pub(super) identity: NodeIdentity,
    pub(super) observed: ApiObservedState,
    pub(super) runtime_config: StateSubscriber<RuntimeConfig>,
    pub(super) dcs_handle: DcsHandle,
    pub(super) transport: ApiServerTransport,
    pub(super) reload_certificates: ApiTlsCertificateReloadHandle,
    pub(super) log: LogSender,
}
```

Do not carry loose bind/auth settings separately through the runtime if they are already available in the existing config structs.

Hard requirement: if a field is removed from the runtime carrier because it is really configuration, it must move into the owning config struct or one dedicated settings ADT. Do not reintroduce it indirectly through a `build_*` function that reconstructs the same carrier by hand.

**Delete checklist for DCS runtime-carrier unification:**
- `DcsAdvertisedEndpoints` in `src/dcs/startup.rs`
- `DcsRuntimeRequest` in `src/dcs/startup.rs`
- `DcsEtcdConfig` in `src/dcs/state.rs`
- `DcsCadence` in `src/dcs/state.rs`
- `DcsLocalMemberAdvertisement` in `src/dcs/state.rs`
- `DcsObservedState` in `src/dcs/state.rs`
- `DcsStateChannel` in `src/dcs/state.rs`
- `DcsControlPlane` in `src/dcs/state.rs`
- `DcsRuntime` in `src/dcs/state.rs`
- `DcsRuntime` in `src/dcs/startup.rs`
- `DcsWorkerCtx` in `src/dcs/state.rs`
- `DcsWorkerBootstrap` in `src/dcs/worker.rs`

```rust
pub(super) struct DcsRuntimeCtx {
    pub(super) identity: NodeIdentity,
    pub(super) config: crate::config::DcsConfig,
    pub(super) advertised_postgres: PgEndpoint,
    pub(super) pg: StateSubscriber<PgInfoState>,
    pub(super) publisher: StatePublisher<DcsSnapshot>,
    pub(super) command_inbox: tokio::sync::mpsc::UnboundedReceiver<DcsCommand>,
    pub(super) log: LogSender,
}
```

Do not keep loose cadence/config fields such as `member_ttl_ms` and `poll_interval` hanging off the runtime context. Keep settings in existing config structs or one dedicated settings ADT.

Hard requirement: these removed loose fields must end up in the owning config/settings layer. The refactor must preserve the behavior while relocating the configuration to the correct owner, and it must not recreate the same loose-field plumbing behind a `build_*` helper.

**Conversion-builder cleanup that should follow from this migration:**
- The goal is not to ban every `build_*` function in the repo. The goal is to remove the `build_*` helpers whose only job is to translate between duplicate DTOs, duplicate enum trees, or repacked carrier structs.
- Output/render-layer assembly functions may survive if they are genuinely presentation-specific.
- Domain/data transition functions should become direct ADT constructors, `From`/`TryFrom` implementations, or small methods on the owning type, instead of free functions that rebuild the same data under a different wrapper.

**Concrete `build_*` / conversion helpers to remove or collapse as part of this task:**
- `src/api/controller.rs`
  - `build_node_state`
- `src/dcs/state.rs`
  - `build_dcs_view`
  - `build_member_view`
  - `build_local_member_record`
- `src/dcs/worker.rs`
  - `build_worker_ctx`
- `src/ha/worker.rs`
  - `build_next_state`
  - `build_data_dir_state`
  - `build_postgres_state`
  - `build_local_postgres_state`
  - `build_replication_state`
  - `build_storage_state`
  - `build_global_knowledge`
  - `build_peer_knowledge_from_member`
  - `build_self_peer`
  - `build_leadership_view`
- `src/cli/connect.rs`
  - `build_connection_view`
  - `build_connection_target`
  - `build_primary_connection_target`

These should either disappear entirely or collapse into direct constructors / `From` implementations on the canonical ADTs. The point is to stop rebuilding the same semantics through builder-shaped translation functions after the duplicate-type migration.

**Delete checklist for child-process guard unification:**
- `ChildGuard` in `src/api/worker.rs`
- `ChildGuard` in `src/process/postmaster.rs`
- `ChildGuard` in `src/process/worker.rs`

Use one shared internal child-guard type instead of three copies:

```rust
pub(super) struct ChildGuard(pub Option<tokio::process::Child>);
```

**Delete checklist for runner-seed unification:**
- `RunnerApiSeed` in `tests/ha/support/observer/pgtm.rs`
- duplicate seed carrier split between `RunnerApiSeed` and `RunnerSeed`

**Alternatives and design questions that must be explicitly resolved during execution, not ignored:**
- `MandatoryPostgresRolesConfig`: the default preferred answer is the generic `PostgresRoleSlots<T>` ADT above, but the implementer should explicitly compare it against any more type-driven alternative they believe is better. A keyed map is not preferred because it loses exhaustiveness unless you wrap it in another ADT.
- `SecretSource`: this task should not introduce an `OptionalSecret<T>` wrapper. The intended direction is one concrete Rust enum for secret sourcing, including absence, file, env, and string.
- Switchover modeling: prefer the single enum above over keeping both a state enum and a target enum, because it makes the intent explicit and removes wrapper churn. If the implementer disagrees, they must document why and still converge the whole repo onto one canonical switchover ADT.
- PostgreSQL connection modeling: keep the canonical `PgConnInfo` shape small. Settings such as timeout, default database name, application name, and arbitrary libpq options belong in config/policy or in a truly separate ADT if current usage proves they are needed.
- Removed fields are not optional deletions. If a field is taken out of a canonical domain ADT because it is really configuration, the task requires moving it into the owning config structs or a dedicated settings ADT and updating all call sites accordingly, without reintroducing the same shape through `build_*` plumbing.
- DCS semantics: remove `DcsMode` entirely and require HA and every other consumer to branch on `DcsSnapshot::Quorum { .. }` versus `DcsSnapshot::NotTrusted`. Never keep or expose stale DCS data under `NotTrusted`.
- Visibility: for the new internal types, prefer private, then `pub(super)`. Do not default new internal ADTs to `pub(crate)`.

**Out of scope:**
- Do not preserve duplicate structs just because they sit in different modules.
- Do not stop after introducing aliases if the underlying duplicate struct declarations still remain.
- Do not keep a second "view" or "record" tree unless it adds genuinely different information that cannot be expressed by the canonical ADT.
- Do not weaken tests or skip validation to get through the refactor faster.

**Expected outcome:**
- The listed duplicate types are unified onto a small set of canonical domain ADTs.
- The repo carries fewer struct declarations than before, and that reduction is proven with committed evidence.
- API, CLI, DCS, HA, process, pginfo, config, and test support all depend on the same domain types instead of converting between near-identical twins.
- Naming collisions such as the two different `ProcessState` types are resolved so the domain boundaries read clearly, and the surviving canonical names read like owned domain concepts rather than transport wrappers.
</description>

<acceptance_criteria>
- [ ] Generate `.ralph/evidence/duplicate-struct-cleanup/type_inventory.before.txt` before the refactor and `.ralph/evidence/duplicate-struct-cleanup/type_inventory.after.txt` after the refactor using `tools/type_inventory_gen/` or a stricter improved successor.
- [ ] Record and verify the before/after `struct` declaration counts under `src/` and `tests/`; the after count must be lower. If it is not lower, this task is not done.
- [ ] `src/api/controller.rs`: delete the separate `NodeStateSnapshot` struct declaration and replace call sites with the canonical node snapshot ADT.
- [ ] `src/api/mod.rs`: delete the separate `NodeState` struct declaration and replace it with the canonical node snapshot ADT or a type alias onto it.
- [ ] Create the canonical `NodeSnapshot` ADT and use it as the single node-state shape.
- [ ] `src/api/controller.rs`: delete `build_node_state`; the canonical node snapshot/read model must be constructed directly by the owning ADT or a direct conversion implementation, not by a duplicate-shape builder function.
- [ ] `src/api/worker.rs`: delete the separate `ApiClusterIdentity` struct declaration and switch the API runtime to `NodeIdentity`.
- [ ] `src/dcs/state.rs`: delete the separate `DcsNodeIdentity` struct declaration and switch DCS runtime/state to `NodeIdentity`.
- [ ] `src/ha/state.rs`: delete the separate `HaNodeIdentity` struct declaration and switch HA runtime/state to `NodeIdentity`.
- [ ] `src/pginfo/state.rs`: delete the separate `PgNodeIdentity` struct declaration and switch pginfo runtime/state to `NodeIdentity`.
- [ ] `src/process/state.rs`: delete the separate `ProcessNodeIdentity` struct declaration and switch process runtime/state to `NodeIdentity`.
- [ ] `src/api/controller.rs`: delete the local `SwitchoverRequest` struct declaration and move API input shaping to the canonical switchover ADT.
- [ ] `src/cli/args.rs`: delete the separate `SwitchoverRequestArgs` struct declaration and parse CLI input directly into the canonical switchover ADT.
- [ ] `src/cli/client.rs`: delete the separate `SwitchoverRequestInput` struct declaration and serialize the canonical switchover ADT instead.
- [ ] `src/ha/types.rs`: delete the current wrapper-style `SwitchoverState` enum declaration and replace it with the canonical single-enum switchover ADT.
- [ ] `src/ha/types.rs`: delete the separate `SwitchoverRequest` struct declaration and store the canonical switchover ADT instead.
- [ ] `src/state/coordination.rs`: delete the separate `SwitchoverTarget` enum declaration and fold that meaning into the canonical single-enum switchover ADT.
- [ ] `src/dcs/state.rs`: delete the separate `SwitchoverView` enum declaration and replace it with the canonical single-enum switchover ADT.
- [ ] `src/cli/status.rs`: delete the separate `ClusterSwitchoverView` struct declaration and derive status output from the canonical switchover ADT.
- [ ] `src/dcs/state.rs`: delete the separate `SwitchoverRecord` struct declaration and persist/use the canonical switchover ADT instead.
- [ ] Create the canonical single-enum `SwitchoverState` ADT and use it repo-wide for pending switchover state, API input, CLI parsing, CLI status, DCS persistence, and DCS command publication.
- [ ] `src/config/schema.rs`: delete the separate `ApiAuthConfig` enum declaration and replace it with the canonical auth-envelope ADT.
- [ ] `src/config/schema.rs`: delete the separate `PgtmApiAuthConfig` enum declaration and replace it with the canonical auth-envelope ADT.
- [ ] `src/config/schema.rs`: rewrite the existing `SecretSource` enum to the canonical `None | Env | File | String` shape and remove the older `Path`, `PathConfig`, and `Inline` variants.
- [ ] `src/api/worker.rs`: delete the separate `ApiAuthState` enum declaration and replace it with the canonical auth-envelope ADT.
- [ ] `src/api/worker.rs`: delete the separate `ResolvedApiRoleTokens` struct declaration and replace it with the canonical role-token carrier.
- [ ] `src/cli/client.rs`: delete the separate `CliAuthConfig` struct declaration and replace it with the canonical role-token carrier or a type alias onto it.
- [ ] `src/config/schema.rs`: delete the separate `ApiRoleTokensConfig` struct declaration and replace it with the canonical role-token carrier or a type alias onto it.
- [ ] `src/dev_support/auth.rs`: delete the separate `ApiRoleTokens` struct declaration and replace it with the canonical role-token carrier or a type alias onto it.
- [ ] Create the canonical concrete `RoleTokens` ADT and make it the single role-token carrier.
- [ ] Create the canonical concrete `TokenAuth` ADT and make it the single auth envelope for config, runtime, CLI, and dev-support auth modeling.
- [ ] Remove the current optional-token wrapper churn; do not introduce or keep a generic `OptionalSecret<T>` style ADT in this task.
- [ ] `src/config/schema.rs`: delete the separate `MandatoryPostgresRolesConfig` struct declaration and replace it with the canonical fixed-role-slot ADT or a type alias onto it.
- [ ] `src/postgres_roles.rs`: delete the separate `MandatoryManagedRoleSet` struct declaration and replace it with the canonical fixed-role-slot ADT or a type alias onto it.
- [ ] `src/process/state.rs`: delete the separate `MandatoryPostgresRuntimeRoles` struct declaration and replace it with the canonical fixed-role-slot ADT or a type alias onto it.
- [ ] Create the canonical `PostgresRoleSlots<T>` ADT and make it the single mandatory-role-slot carrier.
- [ ] `src/pginfo/conninfo.rs`: delete the separate `PgConnInfo` struct declaration and replace it with the canonical connection ADT.
- [ ] `src/pginfo/state.rs`: delete the separate `PgLocalProbeTarget` struct declaration and replace it with the canonical connection or endpoint ADT.
- [ ] `src/cli/connect.rs`: delete the separate `ConnectionTarget` struct declaration and replace it with the canonical connection or endpoint ADT.
- [ ] `src/state/net.rs`: delete the separate `PgTcpTarget` struct declaration and replace it with the canonical endpoint ADT.
- [ ] `src/state/net.rs`: delete the separate `PgUnixTarget` struct declaration and replace it with the canonical endpoint ADT.
- [ ] `src/state/net.rs`: delete the separate `PgConnectTarget` enum declaration and replace it with the canonical endpoint or connection ADT.
- [ ] Create the canonical `PgEndpoint` ADT and make it the single endpoint model.
- [ ] Create the canonical `PgClientTls` ADT and make it the single TLS bundle for PostgreSQL client connections that require root certs and optional client cert/key material.
- [ ] Create the canonical `PgConnInfo` ADT and make it the single structured PostgreSQL connection model.
- [ ] Implement DSN conversion on the canonical `PgConnInfo` type itself via impls such as `Display`/`FromStr`; do not keep a parallel stored DSN field and do not replace it with a new free helper-function layer.
- [ ] Keep timeout/default-database/application-name/libpq-options style settings out of the canonical `PgConnInfo` ADT unless current repo usage proves they are required; if they are required, place them in a separate config/policy ADT rather than the endpoint identity ADT.
- [ ] Any fields removed from the canonical PostgreSQL connection ADT must be relocated into the owning config structs or one dedicated PostgreSQL client-settings ADT; they must not be dropped or reintroduced as loose hanging parameters.
- [ ] `src/dcs/state.rs`: delete the separate `DcsMode` enum declaration completely.
- [ ] `src/dcs/state.rs`: delete the separate `DcsView` enum declaration and replace it with the canonical DCS snapshot ADT or a thin alias/newtype onto it.
- [ ] `src/dcs/state.rs`: delete the separate `ClusterMemberView` struct declaration and replace it with the canonical DCS snapshot/member ADTs.
- [ ] `src/dcs/state.rs`: delete the separate `MemberLeaseRecord` struct declaration and fold that data into the canonical DCS snapshot/member ADTs.
- [ ] `src/dcs/state.rs`: delete the separate `MemberRecord` struct declaration and replace it with the canonical DCS snapshot/member ADTs.
- [ ] `src/dcs/state.rs`: delete the separate `ClusterView` struct declaration and replace it with the canonical DCS snapshot ADT.
- [ ] `src/dcs/state.rs`: delete the separate `NotTrustedView` struct declaration and replace it with the canonical DCS snapshot ADT.
- [ ] `src/dcs/state.rs`: delete the separate `LeadershipObservation` enum declaration and fold that meaning into the canonical DCS snapshot ADT.
- [ ] `src/dcs/state.rs`: delete the separate `LeadershipRecord` struct declaration and fold that data into the canonical DCS snapshot ADT.
- [ ] `src/dcs/state.rs`: delete the separate `DcsCache` struct declaration and replace it with the canonical DCS snapshot ADT.
- [ ] `src/dcs/state.rs`: delete the separate `MemberPostgresView` enum declaration and replace it by using `PgInfoState` directly in the canonical DCS member state.
- [ ] `src/dcs/state.rs`: delete the separate `MemberPostgresRecord` enum declaration and replace it by using `PgInfoState` directly in the canonical DCS member state.
- [ ] Create the canonical `DcsMemberState` ADT and use it as the single per-member DCS snapshot model.
- [ ] Create the canonical `DcsSnapshot` ADT and use it as the single cluster-level DCS read/persisted/cache model.
- [ ] `src/dcs/state.rs`: delete `build_dcs_view`, `build_member_view`, and `build_local_member_record`; the DCS domain should no longer rebuild the same cluster semantics through record/view/cache translation helpers after the canonical snapshot migration.
- [ ] The canonical `DcsSnapshot` must be an enum with `NotTrusted` and `Quorum { leadership, switchover, members }`; do not keep `mode`, `observed_leadership`, or stale-cluster payloads in the snapshot shape.
- [ ] `src/ha/worker.rs` and `src/ha/decide.rs`: refactor the HA loop first so it does not depend on `DcsMode` and never consumes DCS data unless it has `DcsSnapshot::Quorum { .. }`.
- [ ] `src/dcs/worker.rs`: when etcd is unavailable or quorum is lost, publish `DcsSnapshot::NotTrusted` with no stale members, no stale leadership, and no stale switchover data. This is a hard requirement with no exceptions.
- [ ] Add or update focused tests proving that stale DCS data is never exposed under `NotTrusted` and that HA does not act on non-quorum data.
- [ ] `src/api/startup.rs`: delete the separate `ApiRuntimeRequest` struct declaration and replace it with the canonical API runtime context.
- [ ] `src/api/startup.rs`: delete the separate `ApiRuntime` struct declaration and replace it with the canonical API runtime context or direct worker runtime.
- [ ] `src/api/startup.rs`: delete the separate `ApiServer` tuple-struct declaration and replace it with the canonical API runtime context or direct worker runtime.
- [ ] `src/api/worker.rs`: delete the separate `ApiServerCtx` struct declaration and replace it with the canonical API runtime context.
- [ ] `src/api/worker.rs`: delete the separate `ApiControlPlane` struct declaration and fold it into the canonical API runtime context.
- [ ] `src/api/worker.rs`: delete the separate `ApiServingPlan` struct declaration and fold it into the canonical API runtime context.
- [ ] `src/api/worker.rs`: delete the separate `ApiAppState` struct declaration and replace it with the canonical API runtime context or a thin read-model derived from it.
- [ ] Create the canonical `ApiRuntimeCtx` ADT and use it as the single API runtime carrier.
- [ ] `src/dcs/startup.rs`: delete the separate `DcsAdvertisedEndpoints` struct declaration and fold it into the canonical DCS runtime context.
- [ ] `src/dcs/startup.rs`: delete the separate `DcsRuntimeRequest` struct declaration and replace it with the canonical DCS runtime context.
- [ ] `src/dcs/state.rs`: delete the separate `DcsEtcdConfig` struct declaration and fold it into the canonical DCS runtime context or a truly distinct client config ADT if one remains necessary.
- [ ] `src/dcs/state.rs`: delete the separate `DcsCadence` struct declaration and fold it into the canonical DCS runtime context or a truly distinct runtime-only ADT if one remains necessary.
- [ ] `src/dcs/state.rs`: delete the separate `DcsLocalMemberAdvertisement` struct declaration and fold it into the canonical DCS runtime context.
- [ ] `src/dcs/state.rs`: delete the separate `DcsObservedState` struct declaration and fold it into the canonical DCS runtime context.
- [ ] `src/dcs/state.rs`: delete the separate `DcsStateChannel` struct declaration and fold it into the canonical DCS runtime context.
- [ ] `src/dcs/state.rs`: delete the separate `DcsControlPlane` struct declaration and fold it into the canonical DCS runtime context.
- [ ] `src/dcs/state.rs`: delete the separate `DcsRuntime` struct declaration and replace it with the canonical DCS runtime context.
- [ ] `src/dcs/startup.rs`: delete the separate `DcsRuntime` struct declaration and replace it with the canonical DCS runtime context.
- [ ] `src/dcs/state.rs`: delete the separate `DcsWorkerCtx` struct declaration and replace it with the canonical DCS runtime context.
- [ ] `src/dcs/worker.rs`: delete the separate `DcsWorkerBootstrap` struct declaration and replace it with the canonical DCS runtime context.
- [ ] Create the canonical `DcsRuntimeCtx` ADT and use it as the single DCS runtime carrier.
- [ ] Fold loose cadence/settings fields such as `member_ttl_ms` and `poll_interval` into the existing config structs or one dedicated settings ADT; do not keep them as hanging runtime-context scalars.
- [ ] Any fields removed from API/DCS/runtime carrier ADTs because they are really configuration must be relocated into the owning config structs or one dedicated settings ADT; they must not disappear during the refactor.
- [ ] `src/dcs/worker.rs`: delete `build_worker_ctx`; DCS startup/runtime assembly should happen through the canonical runtime/context ADT directly rather than a repacking builder function.
- [ ] `src/ha/worker.rs`: delete the builder-style world/state translation helpers (`build_next_state`, `build_data_dir_state`, `build_postgres_state`, `build_local_postgres_state`, `build_replication_state`, `build_storage_state`, `build_global_knowledge`, `build_peer_knowledge_from_member`, `build_self_peer`, `build_leadership_view`) and replace them with direct methods / conversions on the owning ADTs.
- [ ] `src/cli/connect.rs`: delete `build_connection_view`, `build_connection_target`, and `build_primary_connection_target`; the CLI connection output should derive directly from the canonical endpoint/connection ADTs rather than rebuilding a parallel connection-target DTO via builder helpers.
- [ ] After this migration, translation between canonical domain ADTs must happen through direct constructors, methods, or `From`/`TryFrom` implementations where appropriate, not through a new layer of free `build_*` functions that preserve the old duplicate-shape churn.
- [ ] `src/api/worker.rs`: delete the local `ChildGuard` struct declaration and replace it with the shared internal child-guard type.
- [ ] `src/process/postmaster.rs`: delete the local `ChildGuard` struct declaration and replace it with the shared internal child-guard type.
- [ ] `src/process/worker.rs`: delete the local `ChildGuard` struct declaration and replace it with the shared internal child-guard type.
- [ ] Create one shared internal `ChildGuard` type and use it in every existing child-process guard site.
- [ ] `tests/ha/support/observer/pgtm.rs`: delete the separate `RunnerApiSeed` struct declaration and use `RunnerSeed` or another single canonical test seed ADT instead.
- [ ] `tests/ha/support/runner/mod.rs`: keep only one canonical runner seed ADT and remove the duplicate seed carrier split.
- [ ] Resolve the `ha::types::ProcessState` versus `process::state::ProcessState` name collision so the HA-side type no longer survives under the same name.
- [ ] Audit module re-exports such as `src/dcs/mod.rs`, `src/api/mod.rs`, `src/config/mod.rs`, and any CLI/test support surfaces so they expose only the canonical ADTs and do not keep old duplicate names alive accidentally.
- [ ] New internal canonical helper/runtime ADTs default to private or `pub(super)`; do not leave them as `pub(crate)` unless a wider boundary is explicitly justified by the module structure.
- [ ] Update all affected tests and fixtures so they build through the canonical types rather than reconstructing parallel duplicate DTOs.
- [ ] mandatory rerun of `cargo run --manifest-path tools/type_inventory_gen/Cargo.toml -- type_inventory.txtcargo run --manifest-path tools/type_inventory_gen/Cargo.toml -- type_inventory.txtcargo run --manifest-path tools/type_inventory_gen/Cargo.toml -- type_inventory.txt`, to make sure no new/existing duplicate types (types that have large semantic overlap in fields), exist. Do not commit that output file
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
