---
## Bug: Process worker real job tests accept failure outcomes <status>not_started</status> <passes>false</passes>

<description>
Real-binary process worker tests in [src/process/worker.rs](/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/src/process/worker.rs) accept failure outcomes, so they can pass even when the binary invocation or behavior is broken. Examples:
- `real_promote_job_executes_binary_path`
- `real_demote_job_executes_binary_path`
- `real_restart_job_executes_binary_path`
- `real_fencing_job_executes_binary_path`
These tests currently treat `JobOutcome::Failure` as acceptable, which means regressions (bad binaries, wrong args, or failed operations) can be masked. Tighten these tests so they fail when the real operation fails, or explicitly assert that the intended binary ran and produced expected effects.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
