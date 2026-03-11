# Done Tasks Summary

Generated: Wed Mar 11 05:18:38 PM CET 2026

# Task `.ralph/tasks/story-cluster-startup-friction-improvements/task-smooth-the-local-docker-cluster-startup-experience.md`

```
## Task: Smooth The Local Docker Cluster Startup Experience <status>completed</status> <passes>true</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/01-task-rename-the-operator-cli-to-pgtm-and-flatten-the-command-tree.md`

```
## Task: Rename The Operator CLI To `pgtm` And Flatten The Command Tree <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/02-task-add-config-backed-ctl-contexts-and-auto-auth.md`

```
## Task: Add Config-Backed `pgtm` Configuration And Automatic Auth/TLS Discovery <status>done</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/03-task-add-cluster-wide-status-topology-and-table-output.md`

```
## Task: Add Cluster-Wide `pgtm status` UX With Topology And Table Output <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/04-task-add-primary-resolution-and-shell-friendly-connection-helpers.md`

```
## Task: Add Primary Resolution And Shell-Friendly Connection Helpers To `pgtm` <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/05-task-add-debug-reporting-and-incident-surfaces-to-ctl.md`

```
## Task: Add Debug Reporting And Incident Investigation Surfaces To `pgtm` <status>completed</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-ctl-operator-experience/06-task-rewrite-operator-docs-to-prefer-ctl-over-raw-curl.md`

```
## Task: Rewrite Operator Docs To Use `pgtm` Instead Of Raw `curl` <status>done</status> <passes>true</passes>

<priority>medium</priority>

<description>
```

==============

# Task `.ralph/tasks/story-docs-k2-expansion/01-task-run-k2-docs-loop-in-five-way-parallel-batches.md`

```
## Task: Run K2 Docs Loop In Five-Way Parallel Batches Until All Diataxis Sections Have Enough Pages <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-docs-k2-expansion/02-task-write-chapter-overviews-and-landing-pages-with-ask-k2.md`

```
## Task: Write Chapter Overviews, Introductions, README, And Landing Page With Ask-K2 <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-docs-k2-expansion/03-task-run-truth-only-docs-verification.md`

```
## Task: Run Truth-Only Verification For Documentation Accuracy, Mermaid Diagrams, And Navigation <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-greenfield-cucumber-ha-harness/01-task-build-independent-cucumber-docker-ha-harness-and-primary-crash-rejoin.md`

```
## Task: Build Independent Cucumber Docker HA Harness And Primary Crash Rejoin Feature <status>done</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-greenfield-cucumber-ha-harness/02-task-deep-clean-legacy-black-box-test-infrastructure-after-greenfield-migration.md`

```
## Task: Deep Clean Legacy Black-Box Test Infrastructure After Greenfield Migration <status>done</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-greenfield-cucumber-ha-harness/03-task-add-low-hanging-ha-quorum-and-switchover-cucumber-features-on-greenfield-runner.md`

```
## Task: Add Low-Hanging HA Quorum And Switchover Cucumber Features On The Greenfield Runner <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-greenfield-cucumber-ha-harness/04-task-add-advanced-docker-ha-harness-features-and-migrate-remaining-black-box-scenarios.md`

```
## Task: Add Advanced Docker HA Harness Features And Migrate Remaining Black-Box Scenarios <status>completed</status> <passes>true</passes>

<priority>high</priority>

<description>
```

==============

# Task `.ralph/tasks/story-managed-start-intent-architecture/task-remove-managed-conf-parseback-and-rederive-start-intent.md`

```
## Task: Remove Managed Conf Parse-Back And Re-Derive Start Intent <status>completed</status> <passes>true</passes>

<description>
**Goal:** Remove the current pattern where pgtuskmaster reparses its own managed PostgreSQL startup artifacts from `PGDATA` back into typed startup intent. Replace it with a stricter architecture where typed Rust models are the only authoritative internal model, startup intent is re-derived from DCS plus runtime config plus minimal local physical facts, and managed PostgreSQL files are treated as render outputs only.
```

==============

# Task `.ralph/tasks/story-managed-start-intent-architecture/typed-network-endpoints-instead-of-raw-strings.md`

```
## Task: [Improvement] Type network endpoints instead of carrying raw strings across runtime <status>completed</status> <passes>true</passes>

<description>
The codebase carries API and DCS endpoint addresses as raw `String` values deep into runtime and harness paths, then parses or binds them at scattered call sites. This was detected during a representation-integrity scan looking for cases where subsystem boundaries retain ad-hoc primitive encodings instead of canonical typed models.
```

==============

# Task `.ralph/tasks/story-parallel-ha-test-hardening/01-task-add-postgres-proxy-chaos-e2e-coverage.md`

```
## Task: Add PostgreSQL Proxy Chaos E2E Coverage <status>done</status> <passes>true</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-parallel-ha-test-hardening/02-task-add-ha-restart-and-leadership-churn-e2e-coverage.md`

```
## Task: Add HA Restart And Leadership Churn E2E Coverage <status>completed</status> <passes>true</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-parallel-ha-test-hardening/03-task-add-clone-and-rewind-failure-ha-e2e-coverage.md`

```
## Task: Add Clone And Rewind Failure HA E2E Coverage <status>completed</status> <passes>true</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-parallel-ha-test-hardening/04-task-add-non-e2e-api-tls-hostname-and-san-coverage.md`

```
## Task: Add Non-E2E API TLS Hostname And SAN Coverage <status>completed</status> <passes>true</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-switchover-operator-model/task-add-optional-switchover-to-and-targeted-switchover.md`

```
## Task: Add Optional `switchover_to` And Targeted Switchover Support <status>done</status> <passes>true</passes>

<priority>low</priority>

<description>
```

