## Bug: task_switch.sh fails when current_task.txt is absent <status>not_started</status> <passes>false</passes>

<description>
`bash .ralph/task_switch.sh` completed but emitted `rm: cannot remove '/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/current_task.txt': No such file or directory`.
This means the task switch flow is attempting unconditional cleanup of a file that may not exist.
Explore and research the codebase first, then fix the task-switch boundary so it handles an absent current-task file without surfacing a shell error.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] Running `bash .ralph/task_switch.sh` does not emit a missing-file removal error when `.ralph/current_task.txt` is absent
</acceptance_criteria>
