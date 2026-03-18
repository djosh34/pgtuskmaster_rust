## Bug: Task switch emits missing current_task.txt removal error <status>not_started</status> <passes>false</passes>

<description>
`bash .ralph/task_switch.sh` completed its task-file updates, but it also emitted:
`rm: cannot remove '/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/current_task.txt': No such file or directory`

This means the script assumes `.ralph/current_task.txt` exists during cleanup and leaks an avoidable filesystem error into normal execution. Explore and research the codebase first, then fix the task-switch flow so successful runs do not emit this error when the file is already absent.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
