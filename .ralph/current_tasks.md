# Current Tasks Summary

Generated: Mon Mar  9 06:32:55 CET 2026

# Task `.ralph/tasks/bugs/targeted-switchover-request-can-promote-wrong-node.md`

```
## Bug: Targeted Switchover Request Can Promote Wrong Node <status>not_started</status> <passes>false</passes>

<description>
An accepted targeted switchover request is not reliably honored in the HA multi-node E2E environment. During work on repeated leadership-churn coverage, a request targeted at `node-2` was accepted through `POST /switchover`, but the cluster later stabilized on `node-3` as primary instead. The failure was reproduced in `e2e_multi_node_repeated_targeted_switchovers_preserve_single_primary`, which observed `node-3` as the only stable promoted primary after the targeted request to `node-2`.
```

==============

# Task `.ralph/tasks/story-parallel-ha-test-hardening/02-task-add-ha-restart-and-leadership-churn-e2e-coverage.md`

```
## Task: Add HA Restart And Leadership Churn E2E Coverage <status>completed</status> <passes>false</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-parallel-ha-test-hardening/03-task-add-clone-and-rewind-failure-ha-e2e-coverage.md`

```
## Task: Add Clone And Rewind Failure HA E2E Coverage <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

==============

# Task `.ralph/tasks/story-parallel-ha-test-hardening/04-task-add-non-e2e-api-tls-hostname-and-san-coverage.md`

```
## Task: Add Non-E2E API TLS Hostname And SAN Coverage <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
```

