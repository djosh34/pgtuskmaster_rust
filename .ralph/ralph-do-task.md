
- [ ] read .ralph/current_task.txt
    - [ ] this file will contain a path to a task
    - [ ] from this on we will call that [task name].md
- [ ] read the [task name].md file from the path specified
- [ ] complete the work like this:
    - [ ] If you read nothing, do a EXHAUSTIVE, DEEP and SKEPTICAL review of the plan, and change it if necessary
    - [ ] Once your certain about the plan, replace end of plan with 'NOW EXECUTE'
    - [ ] When you read 'NOW EXECUTE', do not explore, just execute the plan as written, and tick off the boxes when you do them. 

- [ ] you are really done if and only if ALL of these pass 100%:
    - [ ] `make check`
    - [ ] `make test`
    - [ ] `make test-long`
    - [ ] `make lint`
    - [ ] docs are updated with new/updated/deleted features (remove stale/old docs; use `k2-docs-loop` skill for docs updates)

- [ ] only when you're done, and all checks pass:
    - [ ] set in [task name].md
        - [ ] set `<passes>true</passes>`
    - [ ] CRUCIAL: run `/bin/bash .ralph/task_switch.sh` to indicate that you want to switch task. 
                This can be when you're done or just when you want to switch (e.g. for going to subtask)
    - [ ] commit with: `task finished [task name]: [insert text]`
        - [ ] include summary of what was done in commit message (evidence for tests completing, challenges faced during
          implementation)
        - [ ] Make sure to add all files, please do not forget commiting any file when completing a task, also the stuff in .ralph
    - [ ] push commits with: `git push`
    - [ ] Write any learnings/surprises to AGENTS.md, only if it is really needed to help future engineers.
      -  Less in AGENTS.md is better than more
    - [ ] QUIT IMMEDIATELY
