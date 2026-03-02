# Current Tasks Summary

Generated: Thu Feb 26 09:50:23 PM CET 2026

**Path:** `.ralph/tasks/bugs/bug-e2e-product-behavior-failures.md`

## Bug: 5 e2e tests skipped due to product behavior issues <status>not_started</status> <passes>false</passes> <priority>medium</priority>

<description>
During the BDD-to-bun:test conversion (task 10), 5 e2e tests were found to consistently fail due to product behavior issues (not test conversion bugs). The test logic was verified line-by-line against the original BDD step definitions and is 100% identical. These tests were also failing in the original BDD form (all runs since Feb 24 exit=1).

---

**Path:** `.ralph/tasks/story-bdd-to-bun-tests/01-task-shared-test-helpers.md`

## Task: Create shared Bun test helpers to replace Cucumber world/step infrastructure <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Build the shared helper layer that all converted tests will use — replacing Cucumber World types and the step definition pattern with simple, reusable TypeScript functions.

---

**Path:** `.ralph/tasks/story-bdd-to-bun-tests/02-task-convert-cluster-init-tests.md`

## Task: Convert cluster-init BDD test to Bun test <status>not_started</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Convert `test/features/cluster-init.feature` and its step definitions into a plain `bun:test` file.

---

**Path:** `.ralph/tasks/story-bdd-to-bun-tests/03-task-convert-cluster-startup-tests.md`

## Task: Convert cluster-startup BDD test to Bun test <status>done</status> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Convert `test/features/cluster-startup.feature` into a plain `bun:test` file.

---

**Path:** `.ralph/tasks/story-bdd-to-bun-tests/04-task-convert-cluster-stop-tests.md`

## Task: Convert cluster-stop BDD test to Bun test <status>done</status> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Convert `test/features/cluster-stop.feature` into a plain `bun:test` file.

---

**Path:** `.ralph/tasks/story-bdd-to-bun-tests/05-task-convert-cluster-connection-tests.md`

## Task: Convert cluster-connection BDD test to Bun test <status>done</status> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Convert `test/features/cluster-connection.feature` into a plain `bun:test` file.

---

**Path:** `.ralph/tasks/story-bdd-to-bun-tests/06-task-convert-cluster-config-tests.md`

## Task: Convert cluster-config BDD test to Bun test <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Convert `test/features/cluster-config.feature` into a plain `bun:test` file.

---

**Path:** `.ralph/tasks/story-bdd-to-bun-tests/07-task-convert-cluster-readiness-tests.md`

## Task: Convert cluster-readiness BDD test to Bun test <status>done</status> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Convert `test/features/cluster-readiness.feature` into a plain `bun:test` file.

---

**Path:** `.ralph/tasks/story-bdd-to-bun-tests/08-task-convert-cluster-controldata-tests.md`

## Task: Convert cluster-controldata BDD test to Bun test <status>not_started</status> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Convert `test/features/cluster-controldata.feature` into a plain `bun:test` file.

---

**Path:** `.ralph/tasks/story-bdd-to-bun-tests/09-task-convert-all-unit-integration-tests.md`

## Task: Convert ALL unit/integration BDD tests to bun:test <status>done</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Convert every remaining `test/features/**/*.feature` file into plain `bun:test`. This is a massive task covering 30 feature files, 100 scenarios, across 5 step-definition domains. Use 16+ subagents to research, verify, and execute in maximum parallelism.

---

**Path:** `.ralph/tasks/story-bdd-to-bun-tests/10-task-convert-all-e2e-tests.md`

## Task: Convert ALL e2e BDD tests to bun:test <status>not_started</status> <passes>true</passes> <priority>high</priority>

<description>
**Goal:** Convert every `test/e2e/features/**/*.feature` file into plain `bun:test`. This covers 11 feature files, 29 scenarios, across 2 step-definition files. Use 16+ subagents to research, verify, and execute in maximum parallelism.

---

**Path:** `.ralph/tasks/story-bdd-to-bun-tests/11-task-cleanup-and-verify.md`

## Task: Remove Cucumber infrastructure, update parallel runner, and final verification <status>done</status> <passing>true</passing> <priority>high</priority>

<description>
**Goal:** Remove all Cucumber-specific infrastructure, simplify the parallel runner, and do a final sweep to verify zero BDD/Cucumber remnants remain. Use 16+ subagents to research, verify, and execute in maximum parallelism.

---

**Path:** `.ralph/tasks/story-docs-verification/01-meta-task-docs-deep-verification-and-improvement.md`

## Task: Deep Docs Verification and Readability Improvement <status>not_started</status> <passes>meta-task</passes>
NEVER TICK OFF THIS TASK. ALWAYS KEEP <passes>meta-task</passes>. This is a recurring deep verification task.

<description>
**This is a RECURRING META-TASK.** Every time this task is picked up, do a FRESH verification from scratch. Previous passes do not count. **NEVER set this task's passes to anything other than meta-task.**

---

**Path:** `.ralph/tasks/story-event-to-logger-migration/01-task-rename-emit-engine-event.md`

## Task: Rename emitEngineEvent to emitEvent across the codebase <status>done</status> <passes>true</passes>

<description>
**Goal:** Rename the `emitEngineEvent` function to `emitEvent` everywhere it appears — definition, imports, and call sites.

---

**Path:** `.ralph/tasks/story-event-to-logger-migration/03-task-setup-logger-singletons.md`

## Task: Setup logger singletons per module with unified LogTape <status>done</status> <passes>true</passes>

<description>
**Goal:** Establish a module-level logger singleton pattern so every source file that currently uses `eventBus.emit()` for logging can instead use a direct `logger.info()` / `logger.debug()` call. All loggers must funnel into the single LogTape setup — there must NEVER be multiple logger setups, not even in tests.

---

**Path:** `.ralph/tasks/story-event-to-logger-migration/04-task-convert-all-eventbus-emit-to-logger.md`

## Task: Convert ALL eventBus.emit calls to logger calls, delete ALL event type files, and trim EngineEventSchema <status>not_started</status> <passes>false</passes>

<blocked_by>03-task-setup-logger-singletons</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-event-to-logger-migration/05-task-remove-global-eventbus-singleton.md`

## Task: Remove global eventBus singleton, clean up events infrastructure, extract standalone poll utility <status>not_started</status> <passes>false</passes>

<blocked_by>04-task-convert-all-eventbus-emit-to-logger</blocked_by>

<description>

---

**Path:** `.ralph/tasks/story-full-verification/01-meta-task-full-test-and-code-verification.md`

## Task: Full Test Verification and Code Smell Audit <status>not_started</status> <passes>meta-task</passes>
NEVER TICK OFF THIS TASK. ALWAYS KEEP <passes>meta-task</passes>. This is a recurring deep verification task.

<description>
**This is a RECURRING META-TASK.** Every time this task is picked up, do a FRESH verification from scratch. Previous passes do not count. **NEVER set this task's passes to anything other than meta-task.**

