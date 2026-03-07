# How To Read This Book

Use the shortest path that answers the question you have right now, but pick the right depth on purpose. This book is written so that chapter openings stay fast to read while deeper subsections repeat enough context to work out of order.

## Fast routing table

| If you need to know... | Read this next | Why this is the right entry point |
|---|---|---|
| how to get a node running in the supported lab path | **Quick Start** | It follows the checked-in Compose assets and explains what each validation step proves. |
| how to configure or operate a deployment after first launch | **Operator Guide** | It focuses on configuration groups, deployment assumptions, observability, and troubleshooting workflow. |
| why the node is in a given HA phase or refusing a role change | **System Lifecycle** | It explains the sequence, phase gates, and safety reasoning behind the behavior. |
| whether a surprising behavior is protective, conservative, or a real defect | **Architecture Assurance** | It turns the design into explicit invariants, assumptions, and tradeoffs. |
| the exact request/response or CLI contract | **Interfaces** | It documents the control surfaces and points back to Lifecycle when semantics matter. |
| how the implementation is wired together | **Contributors** | It is for implementation work rather than day-2 operation. |

## Recommended reading paths

### Overview path

Read this if you need a high-level map before you touch the system:

1. Start Here
2. Quick Start index
3. Operator Guide index
4. System Lifecycle index
5. Architecture Assurance index
6. Interfaces index

That path gives you the chapter-level structure without forcing you into the deepest operational detail immediately.

### First-run path

Read this if you are standing up the lab for the first time:

1. Quick Start
2. Operator Guide: Deployment and Configuration
3. Initial Validation again after the stack is live
4. Interfaces if you want exact API or CLI shapes for your checks

This path assumes the checked-in container deployment is your starting point. It is the fastest way to prove the repository-owned stack works before you translate it into something more production-like.

### Incident path

Read this if the cluster is already running and behavior is surprising:

1. Operator Guide: Observability and Troubleshooting
2. System Lifecycle for the phase you are observing
3. Interfaces for the exact endpoint or command involved
4. Architecture Assurance if you need to decide whether the conservatism is expected

This route is intentionally repetitive. During an incident you should not need to reconstruct the full book order just to understand a waiting phase or a refused transition.

### Architecture and safety path

Read this if you need to judge design confidence rather than execute an operational step:

1. Start Here: Problem and Solution
2. System Lifecycle
3. Architecture Assurance
4. Interfaces only after the decision model is clear

This order helps you avoid mistaking an API shape for the real safety contract. The endpoints expose state and intent, but the safety model lives in the interaction between trust, leadership evidence, PostgreSQL reachability, and recovery feasibility.

## Reading out of order

It is safe to jump directly into a deep page if you know why you are there. The detailed chapters repeat essential context on purpose. When you do that, keep three assumptions in mind:

- The current implementation is conservative by design, so delays and refusals may be correct outcomes rather than signs of a broken control loop.
- `/ha/state` and related signals describe the current local understanding, not an omniscient cluster truth independent of evidence freshness.
- Planned operations such as switchover still go through the same safety gates as unplanned failover and recovery.

If any deep page raises questions it does not answer, go one level up first. The chapter landing pages are written to hand you to the next most useful explanation rather than force a linear front-to-back read.
