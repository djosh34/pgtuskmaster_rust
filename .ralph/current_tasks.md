# Current Tasks Summary

Generated: Tue Mar 24 02:53:58 AM CET 2026

# Task `.ralph/tasks/bugs/generic-switchover-request-can-stall-without-primary.md`

```
## Bug: Switchover requests can stay pending until manual clear and generic requests can strand the cluster without a primary <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
The switchover lifecycle is not reliably self-clearing in the shipped Docker walkthrough.
```

==============

# Task `.ralph/tasks/bugs/switchover-request-fails-via-non-primary-seed.md`

```
## Bug: Switchover request fails via non-primary seed config <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
`pgtm switchover request` currently posts to the configured seed node API even after it has already read cluster state proving that another member is the authoritative primary.
```

