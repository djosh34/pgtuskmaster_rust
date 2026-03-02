---
## Bug: Process worker real job tests fail with state channel closed <status>not_started</status> <passes>false</passes>

<description>
`make test` failed while running real process worker job tests. Multiple tests panic because process state publish fails with `state channel is closed`.

Repro:
- `make test`
- Failing tests:
- `process::worker::tests::real_demote_job_executes_binary_path`
- `process::worker::tests::real_fencing_job_executes_binary_path`
- `process::worker::tests::real_promote_job_executes_binary_path`
- `process::worker::tests::real_restart_job_executes_binary_path`
- `process::worker::tests::real_start_and_stop_jobs_execute_binary_paths`

Observed trace excerpts:
- `demote job failed: process publish failed: state channel is closed`
- `fencing job failed: process publish failed: state channel is closed`
- `promote job failed: process publish failed: state channel is closed`
- `restart job failed: process publish failed: state channel is closed`
- `stop job failed: process publish failed: state channel is closed`

Please explore and research the codebase first, then implement a fix. Focus on subscriber lifetime and test harness ownership so real job tests keep required watch subscribers alive through all publish calls.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make lint` — grep output file for `congratulations` (pass) or `evaluation failed` (fail)
- [ ] `make test-bdd` — all BDD features pass
</acceptance_criteria>
