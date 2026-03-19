## Bug: task_switch.sh reports rm error when current_task.txt is already absent <status>not_started</status> <passes>false</passes>

<description>
`bash .ralph/task_switch.sh` completed its main flow but still emitted `rm: cannot remove '/home/joshazimullah.linux/work_mounts/patroni_rewrite/pgtuskmaster_rust/.ralph/current_task.txt': No such file or directory`.
Explore and research the Ralph task-switch flow first, then fix the script so task switching does not emit this avoidable file-removal error when the current-task marker is already missing.
</description>

<acceptance_criteria>
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this bug impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
