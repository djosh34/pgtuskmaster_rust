## Task: Add A Perpetual Accepted-Write Convergence Invariant Runner <status>not_started</status> <passes>false</passes>

<priority>high</priority>

<description>
**Goal:** Add the second scenario-agnostic invariant runner the PO asked for: every write that the cluster accepted as committed must eventually be visible on all nodes. Writes rejected because the target was a replica are fine; accepted writes that later fail to converge are not. The higher-order goal is to move write-safety logic completely out of feature files and out of proof-row/workload bookkeeping, replacing it with one simple always-on Jepsen-style runner.

**Context from research and PO direction:**
- The current suite spreads write-safety logic across proof tables, bounded workloads, workload summaries, proof-row visibility checks, and feature-specific fencing assertions.
- The PO explicitly wants a stronger continuous guarantee instead:
  - have a counter or equivalent write source in a table
  - rejected replica writes are fine
  - any accepted write must eventually be seen on all nodes
  - this guarantee is independent from feature files
- `tests/ha/support/workload/mod.rs` and the proof-row machinery are exactly the complexity this task must replace, not preserve.

**Scope:**
- Add a perpetual write-convergence invariant runner under `tests/ha/support/`.
- Wire it into every HA scenario lifecycle.
- Remove the remaining proof-row/workload feature assumptions after the invariant runner replaces them.

**Required behavior:**
- The runner must generate writes against the cluster in a way that cleanly distinguishes accepted writes from rejected writes.
- Accepted writes must be tracked until all nodes observe them.
- Rejected writes because the target is not writable are allowed and should be recorded as rejections, not failures.
- The scenario must fail immediately if an accepted write does not converge within the intended invariant window.
- The invariant runner must produce direct structured evidence of accepted, rejected, and converged writes.

**Expected outcome:**
- Feature files no longer mention bounded workloads, proof tables, proof rows, or recorded workload evidence.
- The suite enforces one simple continuous write-safety rule for every scenario instead of repeatedly scripting it.
- The old proof/workload bookkeeping tree stays deleted.

</description>

<acceptance_criteria>
- [ ] Add a perpetual HA invariant runner that continuously records accepted versus rejected writes and verifies eventual convergence of every accepted write across all nodes.
- [ ] Start this runner automatically for every HA scenario and stop/clean it up deterministically at scenario end.
- [ ] Treat rejected writes to non-writable targets as allowed rejections, not invariant failures.
- [ ] Fail the scenario immediately when an accepted write does not converge as required.
- [ ] Remove feature-level proof-table, proof-row, bounded-workload, and workload-summary assertions once this runner replaces them.
- [ ] Persist structured artifacts that show accepted writes, rejected writes, and convergence outcomes without requiring log scraping.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>

### Execution plan
1. Replace the deleted proof/workload model with one always-on write-convergence runner.
2. Track accepted and rejected writes explicitly so the invariant only judges accepted writes.
3. Sample all nodes until each accepted write converges or the invariant fails.
4. Remove any remaining feature-local write-safety assertions after the runner owns the property globally.
5. Run the required repo validation gates after the invariant runner is active in the reduced suite.

### Constraints for execution
- Keep the invariant runner simple. Do not recreate the old proof/workload/summary abstraction tree under new names.
- Reuse the minimal direct observation/write surfaces from earlier tasks instead of building another parallel harness.
- Fail on missing convergence, not merely on duplicate tokens or late ad hoc checks.
- Do not run `cargo test`; use the required `make` targets, and use `cargo nextest` only for focused local iteration if absolutely needed before the final validation gates.

NOW EXECUTE
