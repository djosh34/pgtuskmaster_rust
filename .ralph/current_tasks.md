# Current Tasks Summary

Generated: Tue Mar 24 03:21:10 AM CET 2026

# Task `.ralph/tasks/bugs/switchover-request-fails-via-non-primary-seed.md`

```
## Bug: Switchover request fails via non-primary seed config <status>not_started</status> <passes>false</passes> <priority>high</priority>

<description>
`pgtm switchover request` currently posts to the configured seed node API even after it has already read cluster state proving that another member is the authoritative primary.
```

