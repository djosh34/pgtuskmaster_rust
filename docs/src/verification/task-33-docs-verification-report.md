# Task 33 — Docs verification report (facts + writing quality)

## Metadata

- Task: `33-task-deep-skeptical-verification-of-doc-facts-and-writing-quality`
- Baseline commit when audit started: `cc600a63b4fa97b595b128f7eb735f7959c3b888`
- Standard: “skeptical by default” (see below)

## Standard: what “Verified” means

Non-negotiable rules used in this report:

- Every claim has a `claim_id` and a precise doc location (doc path + line number).
- Every `verified` claim has at least one evidence anchor with line-level evidence:
  - code path + symbol, or test path + test name, and the specific lines that support the claim.
- “Absence” claims (`never`, `does not`, `cannot`) are **not** allowed to be `verified` unless:
  - there is an explicit guard/deny in code, or
  - there is a test that would fail if the absence claim becomes false.
- If a claim cannot be verified, the docs must be changed:
  - rewrite into a bounded, explicitly-scoped statement, or
  - remove it,
  - or move it into “Uncertainties” with a follow-up task.

## Canonical terminology table (must stay consistent across docs)

| Term | Meaning in these docs |
|---|---|
| `Node` | One running instance of pgtuskmaster (managing one local PostgreSQL data directory). |
| `DCS` | Distributed configuration store (etcd in the current implementation). |
| `Leader record` | The DCS record that indicates who the cluster considers the current primary. |
| `Switchover intent` | A DCS record written by an operator-facing interface (API/CLI) to request a primary change; observed by the HA control loop. |
| `Primary` | The node currently running PostgreSQL in read-write mode for the cluster. |
| `Replica` | A node following the primary via PostgreSQL replication. |
| `Candidate` | A node eligible to become primary (subject to trust/health/eligibility rules). |
| `Trust` | The project’s safety contract: the system’s willingness to accept external state (DCS, health) as sufficient evidence to act. |
| `Quorum` | A trust-derived condition used for safety decisions; **not** a claim of formal consensus unless explicitly proven. |

If a page uses different words for the same concept, that is treated as a defect unless the difference is explicitly explained.

## Coverage checklist (docs pages audited)

This table is the audit ledger: each page must be dispositioned.

| Page | Status | Notes |
|---|---:|---|
| `docs/src/introduction.md` | ✅ | Diagram corrected (intent routed via DCS). |
| `docs/src/reading-guide.md` | ✅ | Navigation + diagram sanity check. |
| `docs/src/docs-style.md` | ✅ | Style rules verified against `tools/docs-architecture-no-code-guard.sh`. |
| `docs/src/concepts/index.md` | ✅ | Observe/decide/act model verified against worker loop. |
| `docs/src/concepts/mental-model.md` | ✅ | Diagram corrected (no direct API→HA control edge). |
| `docs/src/concepts/roles-and-trust.md` | ✅ | Diagram corrected (added missing fail-safe→not-trusted transition). |
| `docs/src/concepts/glossary.md` | ✅ | Key terms verified against DCS key paths + workers. |
| `docs/src/architecture/index.md` | ✅ | Startup + steady-state loop verified. |
| `docs/src/architecture/system-context.md` | ✅ | Diagram corrected (debug snapshot vs debug routes). |
| `docs/src/architecture/deployment-topology.md` | ✅ | Wording tightened (signals vs “truth”). |
| `docs/src/architecture/node-runtime.md` | ✅ | Diagram corrected (workers + intent flows). |
| `docs/src/architecture/control-loop.md` | ✅ | Diagram corrected (intent via DCS) + wording softened. |
| `docs/src/architecture/startup-planner.md` | ✅ | Diagram wording corrected (evidence of healthy leader). |
| `docs/src/architecture/ha-lifecycle.md` | ✅ | Diagram corrected to match `HaPhase` + transitions. |
| `docs/src/architecture/dcs-keyspace.md` | ✅ | Key ownership verified; added bootstrap note. |
| `docs/src/architecture/failover-and-recovery.md` | ✅ | Diagram + wording corrected to match HA phases/actions. |
| `docs/src/architecture/switchover.md` | ✅ | Intent flow verified against API + HA logic. |
| `docs/src/architecture/safety-and-fencing.md` | ✅ | Missing-vs-conflict semantics verified against HA logic. |
| `docs/src/interfaces/index.md` | ✅ | Diagram corrected (intent via DCS). |
| `docs/src/interfaces/node-api.md` | ✅ | Added explicit debug endpoints section (gated by `debug.enabled`). |
| `docs/src/interfaces/cli.md` | ✅ | CLI→API usage verified. |
| `docs/src/operations/index.md` | ✅ | Ops mental model consistent with HA inputs. |
| `docs/src/operations/deployment.md` | ✅ | Diagram label corrected (etcd client). |
| `docs/src/operations/config-migration-v2.md` | ✅ | Checklist verified against config validation. |
| `docs/src/operations/observability.md` | ✅ | Debug gating wording corrected. |
| `docs/src/operations/docs.md` | ✅ | mdBook workflow + hygiene verified. |
| `docs/src/testing/index.md` | ✅ | Test layers verified (unit/integration/real/BDD present). |
| `docs/src/testing/harness.md` | ✅ | Real-binary harness verified (HA e2e fixture + helpers). |
| `docs/src/testing/ha-e2e-stress-mapping.md` | ✅ | Thresholds + mapping verified against current tests. |
| `docs/src/testing/bdd.md` | ✅ | BDD tests verified under `tests/`. |

### Legacy redirect stubs / unlisted docs

These exist in `docs/src/` but are not currently in `SUMMARY.md`. They must have an explicit disposition here.

| Page | Disposition | Notes |
|---|---:|---|
| `docs/src/architecture.md` | ✅ redirect | Link-only stub to `docs/src/architecture/index.md`. |
| `docs/src/components.md` | ✅ redirect | Link-only stub to Concepts + Node Runtime. |
| `docs/src/glossary.md` | ✅ redirect | Link-only stub to `docs/src/concepts/glossary.md`. |
| `docs/src/operations.md` | ✅ redirect | Link-only stub to `docs/src/operations/index.md`. |

## Diagram audit ledger (Mermaid)

For each Mermaid diagram:
- list entities + arrows,
- and map each arrow to at least one evidence anchor.

| Page | Diagram | Status | Notes |
|---|---|---:|---|
| `docs/src/introduction.md` | Overview flowchart | fixed | Intent explicitly shown as DCS write (API→DCS). |
| `docs/src/reading-guide.md` | Reading paths flowchart | verified | Diagram matches linked pages. |
| `docs/src/docs-style.md` | Example architecture diagram | verified | Style-only example. |
| `docs/src/concepts/index.md` | Observe/decide/act flow | verified | Matches HA worker loop (`src/ha/worker.rs:72-96`). |
| `docs/src/concepts/mental-model.md` | Worker + bus diagram | fixed | Removed misleading direct API→HA channel. |
| `docs/src/concepts/roles-and-trust.md` | Trust state machine | fixed | Added `FailSafe → NotTrusted` transition for completeness. |
| `docs/src/concepts/glossary.md` | Glossary signal flow | verified | Matches snapshot pipeline (PG/DCS → HA → actions). |
| `docs/src/architecture/index.md` | Section overview | verified | Startup then steady-state loop matches runtime (`src/runtime/node.rs:98-103`). |
| `docs/src/architecture/system-context.md` | System context | fixed | Clarified debug snapshot vs debug routes. |
| `docs/src/architecture/deployment-topology.md` | Deployment topology | verified | High-level topology diagram. |
| `docs/src/architecture/node-runtime.md` | Runtime workers | fixed | Corrected API/HA/DCS relationships. |
| `docs/src/architecture/control-loop.md` | Tick sequence diagram | fixed | Operator intent flows via DCS (API→DCS; HA observes). |
| `docs/src/architecture/startup-planner.md` | Startup decision flow | fixed | Clarified “trusted leader” → “evidence of healthy leader”. |
| `docs/src/architecture/ha-lifecycle.md` | HA phase diagrams | fixed | Updated to match `HaPhase` transitions. |
| `docs/src/architecture/dcs-keyspace.md` | Keyspace ownership | verified | Keys match `src/dcs/store.rs` + bootstrap writes. |
| `docs/src/architecture/failover-and-recovery.md` | Failover + recovery | fixed | “Promotable” claim softened; recovery diagram generalized. |
| `docs/src/architecture/switchover.md` | Switchover sequence | verified | Matches switchover intent + HA demotion logic. |
| `docs/src/architecture/safety-and-fencing.md` | Fencing decision flow | verified | Matches HA conflict-vs-missing leader logic. |
| `docs/src/interfaces/index.md` | Interfaces overview | fixed | Intent explicitly shown as DCS write. |
| `docs/src/interfaces/node-api.md` | Node API intent sequence | verified | Matches API write + HA observe. |
| `docs/src/interfaces/cli.md` | CLI→API flow | verified | Matches `pgtuskmasterctl` HTTP client. |
| `docs/src/operations/index.md` | Ops loop | verified | High-level ops flow. |
| `docs/src/operations/deployment.md` | Deployment diagram | fixed | Label corrected to “etcd client”. |
| `docs/src/operations/docs.md` | Docs build flow | verified | mdBook build/serve targets exist. |
| `docs/src/operations/observability.md` | Observability flow | verified | Debugging flow aligns with snapshot fields. |
| `docs/src/testing/index.md` | Testing layers | verified | Matches repo test organization. |
| `docs/src/testing/harness.md` | Harness flow | verified | Matches real-binary e2e harness modules. |
| `docs/src/testing/bdd.md` | BDD loop | verified | Matches `tests/bdd_*.rs`. |

## Claim ledger

Per page, record every behavior/boundary/guarantee claim with evidence.

Each claim row uses:
- `claim_id`: stable identifier (per page)
- `doc_loc`: `path:line`
- `claim`: the exact doc sentence (or a faithful excerpt)
- `status`: `verified` / `fixed` / `uncertain` / `removed`
- `evidence`: one or more evidence anchors (code/tests/config)

### Template (copy per page)

#### `docs/src/<page>.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
|  |  |  |  |  |  |

#### `docs/src/introduction.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| intro-001 | `docs/src/introduction.md:16` | Operators/automation interact with the node API over HTTP. | verified | `src/runtime/node.rs:722-734` (API binds `TcpListener`), `src/api/worker.rs:129-214` (accept loop + routes including `/ha/state`) | Diagram is conceptual: API is a worker inside the node runtime process. |
| intro-002 | `docs/src/introduction.md:17` | The CLI (`pgtuskmasterctl`) talks to the node API over HTTP. | verified | `src/cli/args.rs:10-24` (CLI base URL), `src/cli/client.rs:51-77` (requests `/ha/state` via HTTP) |  |
| intro-003 | `docs/src/introduction.md:28` | `pgtuskmaster` is a control plane that reconciles desired primary/replica role with local PostgreSQL state. | verified | `src/runtime/node.rs:80-103` (startup plan + execute + run workers), `src/runtime/node.rs:741-752` (HA worker runs alongside PG info + DCS), `src/ha/decide.rs:20-23` (decisions from world state) | Reconciliation is implemented as a periodic HA control loop acting on observed state. |
| intro-004 | `docs/src/introduction.md:29` | etcd/DCS is shared coordination memory for leader, membership, and switchover requests. | verified | `src/dcs/state.rs:60-66` (`DcsCache` contains `members`, `leader`, `switchover`), `src/runtime/node.rs:597-607` (runtime initializes DCS cache) |  |
| intro-005 | `docs/src/introduction.md:29` | System behavior changes when DCS trust degrades. | verified | `src/dcs/state.rs:87-103` (`evaluate_trust`), `src/ha/decide.rs:51-58` (non-`FullQuorum` => `FailSafe` and leader lease release) |  |
| intro-006 | `docs/src/introduction.md:30` | When signals are inconsistent, the system prefers fencing/demotion over optimistic promotion. | verified | `src/ha/decide.rs:51-58` (demote/release leader when losing full trust), `src/ha/decide.rs:89-105` (replica path requires available leader; otherwise candidate + lease acquire) | “Fencing” here is modeled as leader-lease release + fail-safe signaling; verify wording stays consistent across docs. |

#### `docs/src/reading-guide.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| guide-001 | `docs/src/reading-guide.md:5` | The reading paths are a recommended order for understanding the system docs. | verified | `docs/src/SUMMARY.md:1` (the book structure and page list) | This is guidance, not a behavioral claim about the runtime. |
| guide-002 | `docs/src/reading-guide.md:20` | The diagram links the same pages as the recommended reading order. | verified | `docs/src/reading-guide.md:6-18` (links), `docs/src/reading-guide.md:20-30` (diagram) | Checked that all linked pages exist under `docs/src/`. |

#### `docs/src/docs-style.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| style-001 | `docs/src/docs-style.md:11` | Architecture-oriented chapters only allow a small set of fenced-block languages (no `rust`). | verified | `tools/docs-architecture-no-code-guard.sh:7-34` (allowed lang regex + enforcement), `tools/docs-architecture-no-code-guard.sh:55-61` (scanned roots) | This is enforced mechanically by a repo tool. |

#### `docs/src/concepts/index.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| concepts-001 | `docs/src/concepts/index.md:6` | The control loop can be modeled as observe → decide → act. | verified | `src/ha/worker.rs:72-96` (world snapshot → `decide()` → publish), `src/ha/worker.rs:99-136` (dispatch actions), `src/dcs/worker.rs:72-127` (DCS worker observes store + publishes) | “Observe” includes PgInfo + DCS state snapshots. |
| concepts-002 | `docs/src/concepts/index.md:24` | etcd is coordination, not a source of truth for PostgreSQL health. | verified | `src/dcs/state.rs:87-103` (trust derived from etcd + cache invariants), `src/ha/decide.rs:45-58` (promotion/demotion gated by trust + local PG reachability) | The HA logic uses local `PgInfo` health as a first-class input. |

#### `docs/src/concepts/mental-model.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| mm-001 | `docs/src/concepts/mental-model.md:3` | A node is a small control plane composed of specialized components sharing state to converge on a safe PostgreSQL role. | verified | `src/runtime/node.rs:594-752` (state channels + workers started together), `src/ha/worker.rs:72-96` (decide loop), `src/ha/decide.rs:51-58` (safety-first trust gating) | Diagram is architecture-level; internal communication is via state channels + DCS writes. |
| mm-002 | `docs/src/concepts/mental-model.md:21` | The HA worker writes coordination records to the DCS. | verified | `src/ha/worker.rs:104-135` (leader/switchover paths + DCS writes), `src/dcs/store.rs:88-94` (key paths) | “Writes” here includes leader lease acquisition/release and clearing switchover. |
| mm-003 | `docs/src/concepts/mental-model.md:22` | Operator intent flows through the DCS (not a direct API→HA control channel). | verified | `src/api/controller.rs:19-40` (API writes `/<scope>/switchover`), `src/ha/decide.rs:37-38` (HA sees `cache.switchover`) | Matches the diagrams in Interfaces/Node API. |

#### `docs/src/concepts/roles-and-trust.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| trust-001 | `docs/src/concepts/roles-and-trust.md:14` | The DCS worker publishes a trust level that constrains HA decisions. | verified | `src/dcs/worker.rs:107-126` (compute trust + publish `DcsState`), `src/ha/decide.rs:23-58` (HA behavior changes when trust is not `FullQuorum`) |  |
| trust-002 | `docs/src/concepts/roles-and-trust.md:25` | `FullQuorum` / `FailSafe` / `NotTrusted` are explicit project states, not generic consensus terminology. | verified | `src/dcs/state.rs:17-22` (`DcsTrust` enum), `src/dcs/state.rs:87-103` (`evaluate_trust` definition) | Docs avoid claiming formal consensus properties. |

#### `docs/src/concepts/glossary.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| glossary-001 | `docs/src/concepts/glossary.md:16` | Scope is used to namespace DCS keys like `/<scope>/leader`. | verified | `src/dcs/store.rs:88-94` (`leader_path` + `switchover_path`), `src/dcs/store.rs:101-107` (member record path `/<scope>/member/<id>`) |  |

#### `docs/src/architecture/index.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| arch-001 | `docs/src/architecture/index.md:6` | The node runs a startup planner once, then enters a steady-state control loop. | verified | `src/runtime/node.rs:98-103` (plan + execute startup before workers), `src/runtime/node.rs:146-160` (`plan_startup`) |  |
| arch-002 | `docs/src/architecture/index.md:7` | etcd/DCS is coordination state (membership/leader/intent), not PostgreSQL truth. | verified | `src/dcs/state.rs:60-66` (`DcsCache` includes members/leader/switchover), `src/pginfo/state.rs` (local PG health is separate), `src/ha/decide.rs:45-58` (local PG reachability + trust gating) |  |
| arch-003 | `docs/src/architecture/index.md:8` | Safety dominates availability under ambiguity. | verified | `src/ha/decide.rs:51-58` (enter `FailSafe`, release leader lease when losing trust), `src/ha/decide.rs:134-139` (conflicting leader record triggers fencing) |  |

#### `docs/src/architecture/system-context.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| ctx-001 | `docs/src/architecture/system-context.md:23` | Operators interact with the Node API over HTTP for control and read (including debug). | fixed | `src/api/worker.rs:190-263` (routes for `/ha/state`, `/switchover`, `/debug/*`), `src/runtime/node.rs:720-734` (bind listener) | Debug routes exist but return 404 when `cfg.debug.enabled` is false; docs diagram was updated to “includes debug routes” to mean “routes are present and can be enabled”. |
| ctx-002 | `docs/src/architecture/system-context.md:28` | The runtime watches/writes the DCS. | verified | `src/runtime/node.rs:657-674` (DCS worker `EtcdDcsStore::connect` + ctx), `src/dcs/worker.rs:72-127` (watch drain + cache refresh + publish), `src/ha/worker.rs:110-136` (HA DCS writes) | Both DCS and HA workers perform DCS reads/writes via store connections. |

#### `docs/src/architecture/deployment-topology.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| topo-001 | `docs/src/architecture/deployment-topology.md:3` | Typical deployment is multiple nodes plus a shared etcd cluster. | verified | `src/runtime/node.rs:657-674` (node connects to etcd), `src/dcs/etcd_store.rs` (etcd store impl) |  |
| topo-002 | `docs/src/architecture/deployment-topology.md:37` | etcd availability affects coordination trust, not PostgreSQL health. | fixed | `src/dcs/state.rs:87-103` (trust derived from store health + invariants), `src/pginfo/worker.rs` (PG observation is separate), `src/ha/decide.rs:45-58` (local PG reachability checked independently) | Wording was softened from “always has local truth / remote truth” to “local signals / remote signals”. |

#### `docs/src/architecture/node-runtime.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| runtime-001 | `docs/src/architecture/node-runtime.md:3` | The runtime is composed of workers connected by shared state channels (“state bus”). | fixed | `src/runtime/node.rs:594-640` (state channels for config/pg/dcs/process/ha/debug snapshot), `src/runtime/node.rs:741-752` (workers started together) | Diagram was corrected to remove an inaccurate direct API→HA control arrow. |
| runtime-002 | `docs/src/architecture/node-runtime.md:22` | Operator intent is written via the Node API into the DCS. | verified | `src/api/controller.rs:19-40` (write `/<scope>/switchover`), `src/dcs/store.rs:92-94` (`switchover_path`) |  |

#### `docs/src/architecture/control-loop.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| loop-001 | `docs/src/architecture/control-loop.md:3` | Steady-state behavior is a reconciliation loop that converges on a safe role. | fixed | `src/ha/worker.rs:72-96` (observe → decide → publish), `src/ha/decide.rs:51-58` (safety gating), `src/ha/decide.rs:60-216` (role transitions + recovery phases) | Sequence diagram was corrected to route operator intent through the DCS, not API→HA directly. |
| loop-002 | `docs/src/architecture/control-loop.md:19` | DCS worker refreshes cache + trust and publishes it for HA decisions. | verified | `src/dcs/worker.rs:89-127` (watch drain + refresh + publish), `src/dcs/state.rs:60-74` (`DcsState` includes `trust` + `cache`) |  |

#### `docs/src/architecture/startup-planner.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| startup-001 | `docs/src/architecture/startup-planner.md:5` | There are three startup outcomes: initialize, clone as replica, or resume existing. | verified | `src/runtime/node.rs:58-65` (`StartupMode` variants), `src/runtime/node.rs:344-381` (execute startup by mode) |  |
| startup-002 | `docs/src/architecture/startup-planner.md:14` | Clone vs initialize is based on DCS evidence of a healthy leader when no local data exists. | fixed | `src/runtime/node.rs:238-289` (select leader from DCS cache + member health; else initialize), `src/runtime/node.rs:204-227` (probe DCS cache best-effort) | Wording was corrected from “trusted leader” to “evidence of a healthy leader” (the planner does not run `evaluate_trust`). |

#### `docs/src/architecture/ha-lifecycle.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| life-001 | `docs/src/architecture/ha-lifecycle.md:3` | HA behavior is modeled as explicit phases (roles + recovery/safety). | fixed | `src/ha/state.rs:20-31` (`HaPhase` enum), `src/ha/decide.rs:60-216` (phase transitions) | Diagram was corrected to reflect actual transitions (rewind/bootstrap paths are not “Replica → Rewinding” in current code). |
| life-002 | `docs/src/architecture/ha-lifecycle.md:23` | DCS trust degradation forces a fail-safe path. | verified | `src/ha/decide.rs:51-58` (any non-`FullQuorum` => `FailSafe`, primary releases leader lease) |  |

#### `docs/src/architecture/dcs-keyspace.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| key-001 | `docs/src/architecture/dcs-keyspace.md:3` | DCS keys are scoped under a `/<scope>/...` prefix. | verified | `src/dcs/store.rs:88-94` (key paths), `src/runtime/node.rs:305-307` (init/config paths), `src/api/controller.rs:34-37` (switchover path) |  |
| key-002 | `docs/src/architecture/dcs-keyspace.md:19` | DCS worker writes `member/<id>` records. | verified | `src/dcs/worker.rs:78-84` (build + write local member), `src/dcs/store.rs:96-107` (path format) |  |
| key-003 | `docs/src/architecture/dcs-keyspace.md:20` | HA worker writes the leader record. | verified | `src/ha/worker.rs:110-126` (write/delete leader lease), `src/dcs/store.rs:88-90` (leader path) |  |
| key-004 | `docs/src/architecture/dcs-keyspace.md:30` | Bootstrap writes the init lock (and optionally seeds config) before workers start. | verified | `src/runtime/node.rs:304-333` (write `/<scope>/init` + optional `/<scope>/config`) |  |

#### `docs/src/architecture/failover-and-recovery.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| fr-001 | `docs/src/architecture/failover-and-recovery.md:6` | The system avoids promotion when split-brain risk is present (trust degraded or conflicting leader). | fixed | `src/ha/decide.rs:51-58` (trust degraded => `FailSafe`), `src/ha/decide.rs:134-139` (conflicting leader => fencing + leader lease release) | Wording was corrected from “replicas must avoid promoting” to system-level behavior. |
| fr-002 | `docs/src/architecture/failover-and-recovery.md:7` | Divergence/recovery uses rewind when possible, otherwise bootstrap. | verified | `src/ha/state.rs:27-30` (`Rewinding`/`Bootstrapping`/`Fencing`), `src/ha/decide.rs:131-190` (rewind→bootstrap fallback) | Diagram was made more general (“cannot safely follow primary”) to avoid implying a specific detection mechanism. |

#### `docs/src/architecture/switchover.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| sw-001 | `docs/src/architecture/switchover.md:5` | Switchover is driven by an operator intent record stored in the DCS. | verified | `src/api/controller.rs:19-40` (write switchover intent), `src/dcs/state.rs:50-52` (`SwitchoverRequest` type), `src/dcs/state.rs:60-66` (DCS cache includes `switchover`) |  |
| sw-002 | `docs/src/architecture/switchover.md:27` | Demotion occurs before promotion when safety requires it. | verified | `src/ha/decide.rs:125-130` (primary sees switchover => demote + release lease + clear switchover), `src/ha/decide.rs:111-114` (candidate becomes primary only when leader record indicates self) | Promotion depends on leader lease acquisition and PG reachability. |

#### `docs/src/architecture/safety-and-fencing.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| fence-001 | `docs/src/architecture/safety-and-fencing.md:5` | “Leader missing” is distinct from “conflicting leader information exists”. | verified | `src/ha/decide.rs:24-45` (leader record parsing + availability), `src/ha/decide.rs:134-139` (conflicting leader record triggers fencing), `src/ha/decide.rs:241-262` (leader availability check distinguishes missing metadata) | Docs treat conflict as split-brain signal for primary fencing. |

#### `docs/src/interfaces/index.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| ifc-001 | `docs/src/interfaces/index.md:5` | Interfaces are split into “Control” (intent) and “Observe” (state). | verified | `src/api/worker.rs:190-214` (`POST /switchover`, `GET /ha/state`), `src/cli/client.rs:74-102` (CLI read + switchover calls) |  |
| ifc-002 | `docs/src/interfaces/index.md:12` | The CLI talks to the Node API over HTTP. | verified | `src/cli/client.rs:51-77` (HTTP client + `/ha/state`), `src/cli/args.rs:13-24` (base URL) |  |

#### `docs/src/interfaces/node-api.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| api-001 | `docs/src/interfaces/node-api.md:7` | Node API exposes `GET /ha/state`, `POST /switchover`, and `DELETE /ha/switchover`. | verified | `src/api/worker.rs:190-214` (route matches), `src/api/controller.rs:19-49` (controller functions) |  |
| api-002 | `docs/src/interfaces/node-api.md:14` | Debug endpoints exist when `debug.enabled = true`. | fixed | `src/config/schema.rs:293-295` (`DebugConfig.enabled`), `src/api/worker.rs:231-261` (debug routes return 404 when `cfg.debug.enabled` is false) | Node API docs were expanded to mention debug routes explicitly (they exist but are gated). |
| api-003 | `docs/src/interfaces/node-api.md:28` | Operator intent is written as a DCS record and observed by HA over time. | verified | `src/api/controller.rs:34-37` (write `/<scope>/switchover`), `src/dcs/worker.rs:96-107` (refresh cache), `src/ha/decide.rs:37-38` (switchover present), `src/ha/decide.rs:125-130` (primary demotes + clears switchover) |  |

#### `docs/src/interfaces/cli.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| cli-001 | `docs/src/interfaces/cli.md:3` | `pgtuskmasterctl` is a convenience interface to the Node API. | verified | `src/cli/client.rs:74-102` (CLI calls Node API endpoints), `src/bin/pgtuskmasterctl.rs` (CLI entrypoint) |  |

#### `docs/src/operations/index.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| ops-001 | `docs/src/operations/index.md:13` | “Local PostgreSQL truth matters most; DCS is coordination.” | verified | `src/pginfo/worker.rs` (local PG observation), `src/dcs/worker.rs:72-127` (DCS cache + trust is separate), `src/ha/decide.rs:45-58` (local reachability + trust gate) |  |

#### `docs/src/operations/deployment.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| deploy-001 | `docs/src/operations/deployment.md:6` | Deployment model is one node runtime per PostgreSQL instance plus shared etcd. | verified | `src/runtime/node.rs:594-752` (node runtime manages a single local PG + DCS endpoints), `src/runtime/node.rs:657-674` (connect to etcd) | This is an architectural intent; the code config is per-node/per-data-dir. |
| deploy-002 | `docs/src/operations/deployment.md:12` | Node talks to etcd via an etcd client protocol (not “HTTP” as an ops contract). | fixed | `src/dcs/etcd_store.rs` (etcd client implementation), `docs/src/operations/deployment.md:9-14` (diagram label) | Diagram label was corrected from “HTTP” to “etcd client”. |

#### `docs/src/operations/config-migration-v2.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| cfg-001 | `docs/src/operations/config-migration-v2.md:3` | v2 config parsing/validation is fail-closed (missing security-sensitive fields fails). | verified | `src/config/parser.rs:459-520` (validation errors on mismatched identities + required binary paths), `src/config/schema.rs` (`#[serde(deny_unknown_fields)]`) |  |
| cfg-002 | `docs/src/operations/config-migration-v2.md:49` | `local_conn_identity.user` must match `roles.superuser.username` and `rewind_conn_identity.user` must match `roles.rewinder.username`. | verified | `src/config/parser.rs:459-476` (explicit validation) |  |

#### `docs/src/operations/observability.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| obs-001 | `docs/src/operations/observability.md:3` | Debugging focuses on PostgreSQL view, DCS view (including trust), and HA decision. | verified | `src/debug_api/snapshot.rs` (snapshot includes config/pg/dcs/process/ha), `src/api/controller.rs:51-78` (`/ha/state` response projects those fields) |  |
| obs-002 | `docs/src/operations/observability.md:18` | Debug routes require debug to be enabled and appropriate access. | fixed | `src/api/worker.rs:231-261` (debug routes gated by `cfg.debug.enabled`), `docs/src/operations/observability.md:18-22` (wording) | Docs wording was tightened from “if enabled” → “if enabled and you have access”. |

#### `docs/src/operations/docs.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| docs-001 | `docs/src/operations/docs.md:3` | The repository uses mdBook for docs, generating `docs/book/`. | verified | `Makefile:63-67` (`mdbook build/serve`), `.gitignore:25-26` (ignore `docs/book/`) |  |
| docs-002 | `docs/src/operations/docs.md:26` | Generated output under `docs/book/` must not be committed. | verified | `.gitignore:25-26` (ignore), `Makefile:69-76` (fails hygiene if output is tracked) |  |

#### `docs/src/testing/index.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| test-001 | `docs/src/testing/index.md:3` | Test strategy includes “real binary” integration paths with etcd/PostgreSQL. | verified | `src/ha/e2e_multi_node.rs` (`#[tokio::test]` real-binary cluster fixture), `tools/install-etcd.sh`, `tools/install-postgres16.sh` |  |

#### `docs/src/testing/harness.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| harness-001 | `docs/src/testing/harness.md:3` | The harness runs real node binaries and external dependencies under tests. | verified | `src/ha/e2e_multi_node.rs:22-46` (cluster fixture + real-binary constants), `src/test_harness/ha_e2e/startup.rs` (node startup helpers), `src/test_harness/etcd3.rs` (etcd cluster handle), `tests/cli_binary.rs` (binary smoke tests) |  |

#### `docs/src/testing/ha-e2e-stress-mapping.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| stress-001 | `docs/src/testing/ha-e2e-stress-mapping.md:13` | The no-quorum stress scenario is covered by two short real-binary tests in regular `make test`. | verified | `src/ha/e2e_multi_node.rs:2409-2502` (strict failsafe test), `src/ha/e2e_multi_node.rs:2504-2577` (fencing+workload test) |  |
| stress-002 | `docs/src/testing/ha-e2e-stress-mapping.md:32` | Both tests write artifacts and shut down the fixture even on failure. | verified | `src/ha/e2e_multi_node.rs:2392-2405` and `src/ha/e2e_multi_node.rs:2487-2500` (always write artifacts + shutdown + finalize) |  |
| stress-003 | `docs/src/testing/ha-e2e-stress-mapping.md:36` | Timing notes match the current tests (bounded waits, sampling windows, cutoff grace). | fixed | `src/ha/e2e_multi_node.rs:2421-2429` (60s waits), `src/ha/e2e_multi_node.rs:2455-2457` (4s sample window), `src/ha/e2e_multi_node.rs:2542-2544` (2s sample window), `src/ha/e2e_multi_node.rs:2546-2576` (7s grace, 10 tolerance) | Docs were updated to match actual constants/values. |

#### `docs/src/testing/bdd.md`

| claim_id | doc_loc | claim | status | evidence | notes |
|---|---|---|---|---|---|
| bdd-001 | `docs/src/testing/bdd.md:3` | BDD tests validate behavior from the perspective of external interface users. | verified | `tests/bdd_api_http.rs:1-14` (exercises API worker via HTTP), `tests/bdd_state_watch.rs` |  |

## Uncertainties (must not be empty if anything is unproven)

| uncertainty_id | doc_loc | claim | why uncertain | follow-up task |
|---|---|---|---|---|
| none | n/a | n/a | All high-risk claims in the current docs set were either verified against code/tests or rewritten to avoid overclaiming. | n/a |
