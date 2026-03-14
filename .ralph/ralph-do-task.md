
- [ ] read .ralph/current_task.txt
    - [ ] this file will contain a path to a task
    - [ ] from this on we will call that [task name].md
- [ ] read the [task name].md file from the path specified
- [ ] complete the work like this:
    - [ ] If you read nothing/ TO BE VERIFIED, follow these steps:
        - [ ] Change all types into the Goal CDD/'everything as ADT type' pattern, it is very expected this does not compile
        - [ ] You must change ALL types first, and break ALL types at once, before fixing any compiler errors
        - [ ] Iterate on the types, can something be represented better? better enums that make unrepresentable states impossible? Other Structs/Enums in need of big changes to align with new design? change ALL those types first (before fixing compile errors)
        - [ ] Once your happy with type design, replace end of plan with 'NOW EXECUTE'
        - [ ] QUIT IMMEDIATELY!
    - [ ] When you read 'NOW EXECUTE': 
        - [ ] execute the plan as written, fix the compile errors, and tick off the boxes when you do them. 
        - [ ] If at any point you find that the design was not correct, and types are in need of change, switch 'NOW EXECUTE' back to 'TO BE VERIFIED'
        - [ ] If switched, QUIT IMMEDIATELY, else continue until checks pass (only if design is still right)

- [ ] you are really done if and only if ALL of these pass 100%:
    - [ ] `make check`
    - [ ] `make lint`
    - [ ] `make test`
    - [ ] `make test-long`
    - [ ] docs are updated with new/updated/deleted features (remove stale/old docs; use `k2-docs-loop` skill for docs updates), this is only allowed to be done AFTER 'all tests pass' or 'the goal of the task is achieved'

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
