# Feature Trace Matrix (Task 15)

Date (UTC): 2026-03-03

| Feature area | Implementation evidence | Test evidence |
|---|---|---|
| Runtime config schema/default/validation | `src/config/schema.rs` (`RuntimeConfig`, `PostgresConfig`, `SecurityConfig`) | config tests under `src/config/*` and full-suite gates |
| Shared state/watch channels | `src/state/watch_state.rs` (`new_state_channel`, `StatePublisher`, `StateSubscriber`) | unit tests in `src/state/watch_state.rs` |
| PgInfo worker | `src/pginfo/worker.rs` (`run`, `step_once`) | pginfo worker tests in module + full suite |
| DCS store/state/worker | `src/dcs/store.rs`, `src/dcs/state.rs`, `src/dcs/worker.rs` | dcs worker/state tests including malformed watch/unknown-key trust degradation |
| Process worker/jobs | `src/process/worker.rs` + jobs/state modules | process worker unit and real-binary job tests (`real_*_executes_binary_path`) |
| HA decide/actions/worker | `src/ha/decide.rs`, `src/ha/actions.rs`, `src/ha/worker.rs` | decide matrix tests + integration transitions in `src/ha/worker.rs` |
| API worker/controller | `src/api/controller.rs`, `src/api/worker.rs` | `tests/bdd_api_http.rs` + API worker security tests |
| Debug API snapshot worker | `src/debug_api/worker.rs` | debug API worker tests + all-target suites |
| Real PG/etcd harness | `src/test_harness/pg16.rs`, `src/test_harness/etcd3.rs`, `src/test_harness/binaries.rs`, `src/test_harness/namespace.rs` | harness tests (`spawn_*_requires_*_and_spawns`) |
| Multi-node real HA e2e | `src/ha/e2e_multi_node.rs` | `e2e_multi_node_real_ha_scenario_matrix` |
| TLS/auth support | `src/test_harness/tls.rs`, `src/test_harness/auth.rs`, `src/api/worker.rs` | API security tests + `tests/bdd_api_http.rs` auth checks |

## Result
- All planned feature areas have concrete implementation modules and test surfaces present in the current workspace.
- Final confidence still depends on mandatory gate execution logs.
