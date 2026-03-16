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
- `src/cli/connect.rs`
- `src/cli/status.rs`
- `src/config/schema.rs`
- `src/dcs/mod.rs`
- `src/dcs/startup.rs`
- `src/dcs/state.rs`
- `src/dcs/worker.rs`
- `src/dev_support/auth.rs`
- `src/ha/state.rs`
- `src/ha/types.rs`
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
- Switchover state is split across API, CLI args, CLI client, HA, DCS, and CLI status (`SwitchoverRequest`, `SwitchoverRequestArgs`, `SwitchoverRequestInput`, `ClusterSwitchoverView`, `SwitchoverRecord`) and still relies on the older `SwitchoverTarget` split in `src/state/coordination.rs`.
- Role-token carriers are duplicated across config, API runtime, CLI client, and dev support (`ResolvedApiRoleTokens`, `CliAuthConfig`, `ApiRoleTokensConfig`, `ApiRoleTokens`).
- The mandatory Postgres role triplet is duplicated across config, role reconciliation, and process runtime (`MandatoryPostgresRolesConfig`, `MandatoryManagedRoleSet`, `MandatoryPostgresRuntimeRoles`).
- PostgreSQL connection and endpoint modeling is split across pginfo, CLI, and shared state (`PgConnInfo`, `PgLocalProbeTarget`, `ConnectionTarget`, `PgTcpTarget`, `PgUnixTarget`, `PgConnectTarget`), and at least one current type stores rendered DSN data alongside structured fields.
- DCS currently keeps a parallel record/view/cache tree (`ClusterMemberView`, `MemberLeaseRecord`, `MemberRecord`, `ClusterView`, `NotTrustedView`, `LeadershipRecord`, `SwitchoverRecord`, `DcsCache`) plus duplicate member-postgres shapes (`MemberPostgresView` and `MemberPostgresRecord`) to describe nearly the same cluster picture.
- API runtime wiring currently repacks the same fields through `ApiRuntimeRequest`, `ApiServerCtx`, `ApiControlPlane`, `ApiServingPlan`, and `ApiAppState`.
- DCS runtime wiring currently repacks the same fields through `DcsAdvertisedEndpoints`, `DcsRuntimeRequest`, `DcsEtcdConfig`, `DcsCadence`, `DcsLocalMemberAdvertisement`, `DcsObservedState`, `DcsStateChannel`, `DcsControlPlane`, `DcsRuntime` (`src/dcs/state.rs`), `DcsRuntime` (`src/dcs/startup.rs`), `DcsWorkerCtx`, and `DcsWorkerBootstrap`.
- There are three duplicate local `ChildGuard` structs in `src/api/worker.rs`, `src/process/postmaster.rs`, and `src/process/worker.rs`.
- There are duplicate test runner seed carriers in `tests/ha/support/observer/pgtm.rs` and `tests/ha/support/runner/mod.rs`.
- There is also a high-value naming collision that must be resolved while touching the same area: `ha::types::ProcessState` and `process::state::ProcessState` should not both survive under the same name. Even if both remain semantically necessary, the HA one must be renamed to something meaningfully different such as `ProcessAssessment` or `ProcessProjection`.

**Required end-state properties:**
- One canonical `NodeIdentity`.
- One canonical node snapshot/read-model type.
- One canonical switchover ADT.
- One canonical role-token carrier.
- One canonical fixed-role-slot carrier.
- One canonical PostgreSQL endpoint/connection ADT.
- One canonical DCS snapshot/read-model tree.
- One canonical API runtime context instead of multiple field-shuffle carriers.
- One canonical DCS runtime context instead of multiple field-shuffle carriers.
- One shared internal `ChildGuard`.
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

For switchover, prefer a single explicit enum over encoding intent through `Option<MemberId>` plus a second target enum:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SwitchoverSpec {
    AnyHealthyReplica,
    Specific(MemberId),
}
```

Presence or absence of a pending switchover should be represented with `Option<SwitchoverSpec>` at the storage/state level, not by inventing a second request/view/record tree.

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoleTokens<T> {
    pub read_token: T,
    pub admin_token: T,
}
```

Do not keep layering `Option<SecretSource>` everywhere without evaluating a clearer ADT. If the config/runtime/client semantics are meaningfully more than "maybe present", prefer a small explicit enum over a naked `Option`:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionalSecret<T> {
    Missing,
    Configured(T),
}
```

One acceptable end-state is:
- `RoleTokens<OptionalSecret<SecretSource>>` for config
- `RoleTokens<OptionalSecret<String>>` for resolved runtime or CLI auth
- `RoleTokens<String>` for dev/test fixtures

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedPostgresRoles<T> {
    pub superuser: T,
    pub replicator: T,
    pub rewinder: T,
}
```

The most direct use is:

```rust
pub type MandatoryPostgresRolesConfig = FixedPostgresRoles<PostgresRoleConfig>;
pub type MandatoryManagedRoleSet = FixedPostgresRoles<ManagedRoleSpec>;
pub type MandatoryPostgresRuntimeRoles =
    FixedPostgresRoles<MandatoryPostgresRoleCredential>;
```

If alias ergonomics or serde bounds make that exact spelling awkward, another dedicated generic slot ADT is acceptable. What is not acceptable is keeping three same-shape structs.

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PgEndpoint {
    Tcp { host: String, port: u16 },
    Unix { socket_dir: std::path::PathBuf, port: u16 },
}
```

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PgConnectionSpec {
    pub member_id: Option<MemberId>,
    pub endpoint: PgEndpoint,
    pub user: String,
    pub dbname: String,
    pub application_name: Option<String>,
    pub connect_timeout_s: Option<u32>,
    pub ssl_mode: PgSslMode,
    pub ssl_root_cert: Option<std::path::PathBuf>,
    pub options: Option<String>,
}
```

Rendered DSNs should be methods or helper functions, not stored parallel data.

Collapse the DCS member-postgres duplicate view/record shapes into one canonical enum:

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DcsMemberPostgres {
    Unknown {
        readiness: Readiness,
        timeline: Option<TimelineId>,
        system_identifier: Option<SystemIdentifier>,
    },
    Primary {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        committed_wal: ObservedWalPosition,
    },
    Replica {
        readiness: Readiness,
        system_identifier: Option<SystemIdentifier>,
        upstream: Option<MemberId>,
        replay_wal: Option<ObservedWalPosition>,
        follow_wal: Option<ObservedWalPosition>,
    },
}
```

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsMemberState {
    pub lease_ttl_ms: Option<u64>,
    pub postgres_endpoint: PgEndpoint,
    pub postgres: DcsMemberPostgres,
}
```

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DcsSnapshot {
    pub mode: DcsMode,
    pub observed_leadership: Option<LeaseEpoch>,
    pub leadership: Option<LeaseEpoch>,
    pub switchover: Option<SwitchoverSpec>,
    pub members: std::collections::BTreeMap<MemberId, DcsMemberState>,
}
```

If `DcsView` survives, it must collapse to a thin alias/newtype over the canonical snapshot rather than preserving a second record/view/cache tree.

```rust
#[derive(Clone)]
pub(crate) struct ApiRuntimeCtx {
    pub(crate) identity: NodeIdentity,
    pub(crate) observed: ApiObservedState,
    pub(crate) runtime_config: StateSubscriber<RuntimeConfig>,
    pub(crate) dcs_handle: DcsHandle,
    pub(crate) bind: ApiBindConfig,
    pub(crate) auth: ApiAuthState,
    pub(crate) transport: ApiServerTransport,
    pub(crate) reload_certificates: ApiTlsCertificateReloadHandle,
    pub(crate) log: LogSender,
}
```

```rust
pub(crate) struct DcsRuntimeCtx {
    pub(crate) identity: NodeIdentity,
    pub(crate) endpoints: Vec<DcsEndpoint>,
    pub(crate) client: DcsClientConfig,
    pub(crate) poll_interval: std::time::Duration,
    pub(crate) member_ttl_ms: u64,
    pub(crate) advertised_postgres: PgEndpoint,
    pub(crate) pg: StateSubscriber<PgInfoState>,
    pub(crate) publisher: StatePublisher<DcsSnapshot>,
    pub(crate) cache: DcsSnapshot,
    pub(crate) command_inbox: tokio::sync::mpsc::UnboundedReceiver<DcsCommand>,
    pub(crate) log: LogSender,
}
```

Use one shared internal child-guard type instead of three copies:

```rust
pub(crate) struct ChildGuard(pub Option<tokio::process::Child>);
```

**Alternatives and design questions that must be explicitly resolved during execution, not ignored:**
- `MandatoryPostgresRolesConfig`: the default preferred answer is the generic `FixedPostgresRoles<T>` ADT above, but the implementer should explicitly compare it against any more type-driven alternative they believe is better. A keyed map is not preferred because it loses exhaustiveness unless you wrap it in another ADT.
- `Option<SecretSource>`: `SecretSource` is already an enum today in `src/config/schema.rs`. The open design question is whether token presence should remain a naked `Option` or become a clearer enum such as `OptionalSecret<T>`. The final code must intentionally choose one and eliminate the current spread of slightly different optional token carriers.
- `SwitchoverSpec`: prefer the single enum above over a struct with `Option<MemberId>`, because it makes the intent explicit and removes "meaning encoded in None". If the implementer disagrees, they must document why and still converge the whole repo onto one canonical switchover ADT.

**Out of scope:**
- Do not preserve duplicate structs just because they sit in different modules.
- Do not stop after introducing aliases if the underlying duplicate struct declarations still remain.
- Do not keep a second "view" or "record" tree unless it adds genuinely different information that cannot be expressed by the canonical ADT.
- Do not weaken tests or skip validation to get through the refactor faster.

**Expected outcome:**
- The listed duplicate types are unified onto a small set of canonical domain ADTs.
- The repo carries fewer struct declarations than before, and that reduction is proven with committed evidence.
- API, CLI, DCS, HA, process, pginfo, config, and test support all depend on the same domain types instead of converting between near-identical twins.
- Naming collisions such as the two different `ProcessState` types are resolved so the domain boundaries read clearly.
</description>

<acceptance_criteria>
- [ ] Generate `.ralph/evidence/duplicate-struct-cleanup/type_inventory.before.txt` before the refactor and `.ralph/evidence/duplicate-struct-cleanup/type_inventory.after.txt` after the refactor using `tools/type_inventory_gen/` or a stricter improved successor.
- [ ] Record and verify the before/after `struct` declaration counts under `src/` and `tests/`; the after count must be lower. If it is not lower, this task is not done.
- [ ] `src/api/controller.rs`: delete the separate `NodeStateSnapshot` struct declaration and replace call sites with the canonical node snapshot ADT.
- [ ] `src/api/mod.rs`: delete the separate `NodeState` struct declaration and replace it with the canonical node snapshot ADT or a type alias onto it.
- [ ] Create the canonical `NodeSnapshot` ADT and use it as the single node-state shape.
- [ ] `src/api/worker.rs`: delete the separate `ApiClusterIdentity` struct declaration and switch the API runtime to `NodeIdentity`.
- [ ] `src/dcs/state.rs`: delete the separate `DcsNodeIdentity` struct declaration and switch DCS runtime/state to `NodeIdentity`.
- [ ] `src/ha/state.rs`: delete the separate `HaNodeIdentity` struct declaration and switch HA runtime/state to `NodeIdentity`.
- [ ] `src/pginfo/state.rs`: delete the separate `PgNodeIdentity` struct declaration and switch pginfo runtime/state to `NodeIdentity`.
- [ ] `src/process/state.rs`: delete the separate `ProcessNodeIdentity` struct declaration and switch process runtime/state to `NodeIdentity`.
- [ ] `src/api/controller.rs`: delete the local `SwitchoverRequest` struct declaration and move API input shaping to the canonical switchover ADT.
- [ ] `src/cli/args.rs`: delete the separate `SwitchoverRequestArgs` struct declaration and parse CLI input directly into the canonical switchover ADT.
- [ ] `src/cli/client.rs`: delete the separate `SwitchoverRequestInput` struct declaration and serialize the canonical switchover ADT instead.
- [ ] `src/ha/types.rs`: delete the separate `SwitchoverRequest` struct declaration and store the canonical switchover ADT instead.
- [ ] `src/cli/status.rs`: delete the separate `ClusterSwitchoverView` struct declaration and derive status output from the canonical switchover ADT.
- [ ] `src/dcs/state.rs`: delete the separate `SwitchoverRecord` struct declaration and persist/use the canonical switchover ADT instead.
- [ ] Create the canonical `SwitchoverSpec` ADT and use it repo-wide for pending switchover state, API input, CLI parsing, and DCS persistence.
- [ ] `src/api/worker.rs`: delete the separate `ResolvedApiRoleTokens` struct declaration and replace it with the canonical role-token carrier.
- [ ] `src/cli/client.rs`: delete the separate `CliAuthConfig` struct declaration and replace it with the canonical role-token carrier or a type alias onto it.
- [ ] `src/config/schema.rs`: delete the separate `ApiRoleTokensConfig` struct declaration and replace it with the canonical role-token carrier or a type alias onto it.
- [ ] `src/dev_support/auth.rs`: delete the separate `ApiRoleTokens` struct declaration and replace it with the canonical role-token carrier or a type alias onto it.
- [ ] Create the canonical `RoleTokens<T>` ADT and make it the single role-token carrier.
- [ ] Resolve the current `Option<SecretSource>` duplication question explicitly: either create and use a clearer enum such as `OptionalSecret<T>` or document why a naked `Option` remains the canonical choice. Do not leave mixed optional-token shapes behind.
- [ ] `src/config/schema.rs`: delete the separate `MandatoryPostgresRolesConfig` struct declaration and replace it with the canonical fixed-role-slot ADT or a type alias onto it.
- [ ] `src/postgres_roles.rs`: delete the separate `MandatoryManagedRoleSet` struct declaration and replace it with the canonical fixed-role-slot ADT or a type alias onto it.
- [ ] `src/process/state.rs`: delete the separate `MandatoryPostgresRuntimeRoles` struct declaration and replace it with the canonical fixed-role-slot ADT or a type alias onto it.
- [ ] Create the canonical `FixedPostgresRoles<T>` ADT and make it the single mandatory-role-slot carrier.
- [ ] `src/pginfo/conninfo.rs`: delete the separate `PgConnInfo` struct declaration and replace it with the canonical connection ADT.
- [ ] `src/pginfo/state.rs`: delete the separate `PgLocalProbeTarget` struct declaration and replace it with the canonical connection or endpoint ADT.
- [ ] `src/cli/connect.rs`: delete the separate `ConnectionTarget` struct declaration and replace it with the canonical connection or endpoint ADT.
- [ ] `src/state/net.rs`: delete the separate `PgTcpTarget` struct declaration and replace it with the canonical endpoint ADT.
- [ ] `src/state/net.rs`: delete the separate `PgUnixTarget` struct declaration and replace it with the canonical endpoint ADT.
- [ ] `src/state/net.rs`: delete the separate `PgConnectTarget` enum declaration and replace it with the canonical endpoint or connection ADT.
- [ ] Create the canonical `PgEndpoint` ADT and make it the single endpoint model.
- [ ] Create the canonical `PgConnectionSpec` ADT and make it the single structured connection model.
- [ ] `src/dcs/state.rs`: delete the separate `ClusterMemberView` struct declaration and replace it with the canonical DCS snapshot/member ADTs.
- [ ] `src/dcs/state.rs`: delete the separate `MemberLeaseRecord` struct declaration and fold that data into the canonical DCS snapshot/member ADTs.
- [ ] `src/dcs/state.rs`: delete the separate `MemberRecord` struct declaration and replace it with the canonical DCS snapshot/member ADTs.
- [ ] `src/dcs/state.rs`: delete the separate `ClusterView` struct declaration and replace it with the canonical DCS snapshot ADT.
- [ ] `src/dcs/state.rs`: delete the separate `NotTrustedView` struct declaration and replace it with the canonical DCS snapshot ADT.
- [ ] `src/dcs/state.rs`: delete the separate `LeadershipRecord` struct declaration and fold that data into the canonical DCS snapshot ADT.
- [ ] `src/dcs/state.rs`: delete the separate `DcsCache` struct declaration and replace it with the canonical DCS snapshot ADT.
- [ ] `src/dcs/state.rs`: delete the separate `MemberPostgresView` enum declaration and converge on one canonical DCS member-postgres ADT.
- [ ] `src/dcs/state.rs`: delete the separate `MemberPostgresRecord` enum declaration and converge on one canonical DCS member-postgres ADT.
- [ ] Create the canonical `DcsMemberPostgres` ADT and use it as the single DCS member-postgres model.
- [ ] Create the canonical `DcsMemberState` ADT and use it as the single per-member DCS snapshot model.
- [ ] Create the canonical `DcsSnapshot` ADT and use it as the single cluster-level DCS read/persisted/cache model.
- [ ] `src/api/startup.rs`: delete the separate `ApiRuntimeRequest` struct declaration and replace it with the canonical API runtime context.
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
- [ ] `src/api/worker.rs`: delete the local `ChildGuard` struct declaration and replace it with the shared internal child-guard type.
- [ ] `src/process/postmaster.rs`: delete the local `ChildGuard` struct declaration and replace it with the shared internal child-guard type.
- [ ] `src/process/worker.rs`: delete the local `ChildGuard` struct declaration and replace it with the shared internal child-guard type.
- [ ] Create one shared internal `ChildGuard` type and use it in every existing child-process guard site.
- [ ] `tests/ha/support/observer/pgtm.rs`: delete the separate `RunnerApiSeed` struct declaration and use `RunnerSeed` or another single canonical test seed ADT instead.
- [ ] `tests/ha/support/runner/mod.rs`: keep only one canonical runner seed ADT and remove the duplicate seed carrier split.
- [ ] Resolve the `ha::types::ProcessState` versus `process::state::ProcessState` name collision so the HA-side type no longer survives under the same name.
- [ ] Audit module re-exports such as `src/dcs/mod.rs`, `src/api/mod.rs`, `src/config/mod.rs`, and any CLI/test support surfaces so they expose only the canonical ADTs and do not keep old duplicate names alive accidentally.
- [ ] Update all affected tests and fixtures so they build through the canonical types rather than reconstructing parallel duplicate DTOs.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
