# Interface Design for Rust Testability

Good Rust interfaces keep the real inputs in one place and avoid wrapper types
that only rename the same fields.

The process layer in `src/process/` should not turn `cfg` into `runtime`,
`runtime` into `plan`, and `plan` into `spec` just to pass the same facts back down.
That is the wrong shape.

## Rules

1. Create one owning context once.
   - Prefer a single `ProcessContext::new(cfg)` or an equivalent `Self::new(cfg)`.
   - `cfg` already contains identity, DCS, timings, and the other stable config data, so do not pass them separately if they are already inside `cfg`.
   - After that, call methods on the context, such as `ctx.plan(intent)`.
   - Do not build a chain of wrapper types just to carry the same state again.

2. Keep config-derived values on the context.
   - If `data_dir` always comes from `cfg`, it must use cfg directly and never be stored. (store cfg in context then)
   - Do not copy `data_dir`, `wait_seconds`, timeout values, or other config-like fields through `plan`, `spec`, `session`, or other ferry structs.
   - Do not create helper structs whose job is only to regroup fields that already belong in `cfg`.
   - Almost all (98%+) of the fields can directly be fully determined by `cfg`, read it from `cfg` in the lowest layer that actually owns it.
     - when you have something, you think that changes, minimize it to only containing what changes, and never copy the fields
     - easiest way to spot a bad pattern, is if you see .clone(). If you feel you need to clone(), you did it wrong.

3. Do not hide runtime state inside snapshot wrappers.
   - If a value really changes over time, make it a top-level field on the owning struct.
   - Do not wrap `cfg` and the changing fields into a second snapshot type just to move them around.
   - If a nested struct only repeats existing fields, delete it.

4. Use `Self` aggressively inside an `impl`.
   - Prefer `Self::new(...)`, `Self::plan_bootstrap(...)`, and `Self { ... }`.
   - Prefer private methods on the same type over free functions that only forward arguments.
   - If a helper only works on the owning context, keep it on the context.

5. Keep public signatures tiny.
   - Outside code should pass the fewest possible arguments.
   - If a method already lives on the context, it should not need `cfg` or other stable inputs again.
   - The context should own the repeated inputs so callers do not.

6. Keep structs flat.
   - Favor one top-level `match` over nested helper chains that only recombine the same data.
   - If a function only passes data through, remove the function.
   - If a nested struct only exists to ferry values, delete the nesting.

7. Return values, do not hide work in constructors.
   - Build the final value where the inputs are already available.
   - Do not create one struct just so another struct can read the same values back out.

8. Prefer pure construction over mutation.
   - Avoid `mut` unless mutation is genuinely simpler.
   - Build the final value in one expression when possible.

## Before

This is the shape we want to stop using:

```rust
pub(crate) struct ProcessIntentPlanner;

pub(crate) struct ProcessRuntimePlan {
    pub(crate) data_dir: PathBuf,
    pub(crate) socket_dir: PathBuf,
    pub(crate) log_file: PathBuf,
    pub(crate) port: u16,
}

pub(crate) struct ProcessObservedSnapshot {
    pub(crate) runtime_config: RuntimeConfig,
    pub(crate) dcs: DcsSnapshot,
    pub(crate) managed_recovery_state: ManagedRecoverySignal,
}

pub(crate) enum ClusterProcessPlan {
    StartManagedPostgres(ManagedStartPlan),
}

pub(crate) struct ManagedStartPlan {
    pub(crate) mode: PostgresStartMode,
    pub(crate) desired_session: DesiredManagedPostgresSession,
}

pub(crate) struct PreparedManagedPostgresSession {
    pub(crate) config: ManagedPostgresConfig,
}

pub(crate) struct ManagedPostgresConfig {
    pub(crate) postgresql_conf_path: PathBuf,
    pub(crate) hba_path: PathBuf,
    pub(crate) ident_path: PathBuf,
    pub(crate) standby_passfile_path: Option<PathBuf>,
    pub(crate) tls_cert_path: Option<PathBuf>,
    pub(crate) tls_key_path: Option<PathBuf>,
    pub(crate) tls_client_ca_path: Option<PathBuf>,
    pub(crate) standby_signal_path: PathBuf,
    pub(crate) recovery_signal_path: PathBuf,
    pub(crate) postgresql_auto_conf_path: PathBuf,
    pub(crate) quarantined_postgresql_auto_conf_path: PathBuf,
}

pub(crate) struct StartPostgresSpec {
    pub(crate) mode: PostgresStartMode,
    pub(crate) data_dir: PathBuf,
    pub(crate) socket_dir: PathBuf,
    pub(crate) port: u16,
    pub(crate) config_file: PathBuf,
    pub(crate) log_file: PathBuf,
    pub(crate) wait_seconds: Option<u64>,
    pub(crate) timeout_ms: Option<u64>,
}

let runtime = ProcessRuntimePlan::from_config(cfg);
let observed = ProcessObservedSnapshot {
    runtime_config: cfg.clone(),
    dcs: dcs.clone(),
    managed_recovery_state,
};

let plan = planner.plan(&identity, &runtime, &observed, &intent)?;
let prepared_session = session.materialize(&runtime_config, &runtime, &plan)?;
let request = lowerer.lower_execution_request(request_id, &plan, &runtime, &observed, prepared_session.as_ref())?;
```

The problem is not the syntax. The problem is the layering:

- There are many subtypes for one flow: `ProcessRuntimePlan`, `ProcessObservedSnapshot`, `ClusterProcessPlan`, `ManagedStartPlan`, `PreparedManagedPostgresSession`, `ManagedPostgresConfig`, and `StartPostgresSpec`.
- The flow is object-poor and arg-heavy: instead of one owning `Self`, work is split across planner/materializer/lowerer types that keep passing state around.
- `cfg` already exists, but it is rewrapped into `runtime`.
- `cfg` is cloned into `observed.runtime_config`.
- `dcs` and the changing state are bundled into a second snapshot wrapper.
- `prepared_session` is yet another wrapper that only exposes generated file paths.
- `PreparedManagedPostgresSession` is just a one-field wrapper around `ManagedPostgresConfig`.
- `ManagedPostgresConfig` is itself just a bag of generated paths.
- `StartPostgresSpec` recopies `data_dir`, `socket_dir`, `log_file`, `port`, and timeouts even though the reader already has access to `cfg`.
- 98% of the fields in those extra types are stable config fields, so they never belonged in `spec`, `plan`, `intent`, or snapshot wrappers.
- The repeated `.clone()` calls are the smell: stable values are being copied into new types instead of being read from one owning context.
- `plan` and `session` are then used to re-encode the same facts again.
- `data_dir` is carried through specs even when it should simply come from `cfg`.

## After

Create one context and call the final operation on it:

```rust
let ctx = ProcessContext::new(cfg);
let request = ctx.execution_request(request_id, intent)?;
```

Then keep the implementation flat inside the type:

```rust
pub(crate) struct ProcessContext {
    cfg: RuntimeConfigV2,
    managed_recovery_state: ManagedRecoverySignal,
}

impl ProcessContext {
    pub(crate) fn new(cfg: RuntimeConfigV2) -> Self {
        Self {
            cfg,
            managed_recovery_state: ManagedRecoverySignal::None,
        }
    }

    pub(crate) fn execution_request(
        &self,
        request_id: JobId,
        intent: &ProcessIntent,
    ) -> Result<ProcessExecutionRequest, ProcessError> {
        match intent {
            ProcessIntent::Bootstrap => Ok(ProcessExecutionRequest {
                id: request_id,
                kind: ProcessExecutionKind::Bootstrap,
            }),
            ProcessIntent::Promote => Ok(ProcessExecutionRequest {
                id: request_id,
                kind: ProcessExecutionKind::Promote,
            }),
            ProcessIntent::Demote(mode) => Ok(ProcessExecutionRequest {
                id: request_id,
                kind: ProcessExecutionKind::Demote { mode: *mode },
            }),
            ProcessIntent::Start(PostgresStartIntent::Primary) => {
                if self.managed_recovery_state != ManagedRecoverySignal::None {
                    return Err(ProcessError::InvalidSpec(
                        "managed recovery state blocks primary start".to_string(),
                    ));
                }
                Ok(ProcessExecutionRequest {
                    id: request_id,
                    kind: ProcessExecutionKind::StartPostgres {
                        mode: PostgresStartMode::Primary,
                    },
                })
            }
            _ => Err(ProcessError::InvalidSpec("unsupported intent".to_string())),
        }
    }

    fn start_postgres_command(&self, mode: PostgresStartMode) -> ProcessCommandSpec {
        ProcessCommandSpec {
            program: self.cfg.binaries.pg_ctl.clone(),
            args: vec![
                "-D".to_string(),
                self.cfg.postgres.data_dir.display().to_string(),
                "-l".to_string(),
                self.cfg.postgres.log_file.display().to_string(),
                "-o".to_string(),
                format!(
                    "-c config_file={}",
                    self.cfg.postgres.data_dir.join("pgtm.postgresql.conf").display()
                ),
                "start".to_string(),
            ],
            mode,
        }
    }
}
```

This is the collapse process:

1. Delete `ProcessObservedSnapshot` because it repeats `cfg` plus one changing value.
2. Delete `ProcessRuntimePlan` because it mostly re-encodes `cfg`.
3. Delete `PreparedManagedPostgresSession` because it only wraps generated paths.
4. Delete `ClusterProcessPlan` if it only shuffles cloned config into the next function.
5. Delete `StartPostgresSpec` if the command builder already has `self.cfg`.
6. Let the code that needs `data_dir`, `socket_dir`, `log_file`, `port`, and timeouts read them directly from `self.cfg`.

## Example: `ProcessObservedSnapshot` was unnecessary

Before:

```rust
pub(crate) struct ProcessObservedSnapshot {
    pub(crate) runtime_config: RuntimeConfig,
    pub(crate) dcs: DcsSnapshot,
    pub(crate) managed_recovery_state: ManagedRecoverySignal,
}
```

After:

```rust
pub(crate) struct ProcessContext {
    cfg: RuntimeConfigV2,
    managed_recovery_state: ManagedRecoverySignal,
}
```

The point is simple:

- `cfg` already owns the stable config data, including identity.
- `managed_recovery_state` is the real changing value, so it belongs directly on the owning top-level struct.
- `ProcessObservedSnapshot` adds a second layer without adding meaning.
- `PreparedManagedPostgresSession` adds another layer without adding meaning.
- `spec` is also unnecessary if the reader already has `self.cfg`.
- If a value is stable, keep it in `cfg`.
- If a value changes, put it on the top-level owning struct.
- Do not bury either one inside a wrapper that just repeats fields.

## What Good Looks Like

- One context owns the stable inputs.
- `cfg` already contains identity and the stable config values.
- Callers pass almost nothing after construction.
- `Self` handles same-type construction and private helpers.
- Intermediate structs exist only when they carry new meaning, not when they merely rename data.
- A field such as `data_dir` appears once, on the owning context, not through every wrapper or spec clone.
