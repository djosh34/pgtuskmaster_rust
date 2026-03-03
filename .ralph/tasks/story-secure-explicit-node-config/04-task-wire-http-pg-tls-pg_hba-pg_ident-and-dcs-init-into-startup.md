---
## Task: Wire HTTP/PG TLS, pg_hba/pg_ident, and DCS init config into startup orchestration <status>not_started</status> <passes>false</passes>

<description>
**Goal:** Make startup consume the expanded config end-to-end so node boot requires explicit secure config and does not infer missing values.

**Scope:**
- Update runtime/process/startup orchestration to consume new HTTP and PostgreSQL TLS cert/key settings.
- Ensure pg_hba and pg_ident config fields are materialized correctly during bootstrap/start.
- Integrate DCS init config field(s) into initialization logic so bootstrap writes are config-driven.
- Confirm HTTP server has complete explicit config fields (listen, auth, TLS policy) and no hidden fallback.

**Context from research:**
- Some startup behavior currently derives values from defaults and legacy fallbacks.
- Secure deterministic startup requires explicit runtime wiring across process worker, API worker, and DCS bootstrapping paths.

**Expected outcome:**
- A node can only start when complete secure config is present, and all startup side effects follow config directly.

**Execution:** Use subagents (Task tool) to implement changes in parallel where possible.
</description>

<acceptance_criteria>
- [ ] Full exhaustive checklist of all files/modules to modify with specific requirements for each
- [ ] Startup path consumes explicit HTTP TLS cert/key and auth config with no implicit fallback
- [ ] Startup path consumes explicit PostgreSQL TLS/hosting/auth config with no implicit fallback identities
- [ ] pg_hba/pg_ident config fields are written/applied deterministically during startup lifecycle
- [ ] DCS init config is explicitly sourced from config and used during bootstrap/init writes
- [ ] Real-binary and integration tests validate startup with explicit config only
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test` — all BDD features pass
</acceptance_criteria>
