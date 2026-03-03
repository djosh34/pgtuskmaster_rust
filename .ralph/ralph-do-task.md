
- [ ] read .ralph/current_task.txt
    - [ ] this file will contain a path to a task
    - [ ] from this on we will call that [task name].md
- [ ] read the [task name].md file from the path specified
- [ ] complete the work like this:
    - [ ] Plan first the full solution out in detail (using explore (or spark for fast and dumb) subagents in parallel to speed up the research phase)
    - [ ] Make sure to write you full plan to [task name].md so that it is visible for next engineer
        - [ ] Once full plan written, add 'TO BE VERIFIED' and QUIT IMMEDIATELY
    - [ ] If you read 'TO BE VERIFIED', do a DEEP and SKEPTICAL review of the plan, you must alter at least one thing in the plan, otherwise your research was not deep enough. (Also use subagents in parallel for this DEEP SKEPTICAL VERIFICATION)
    - [ ] Once your certain about the plan, replace end of plan with 'NOW EXECUTE'
    - [ ] When you read 'NOW EXECUTE', do not explore, just execute the plan as written, and tick off the boxes when you do them. 

- [ ] you are really done if and only if ALL of these are passing 100%:
    - [ ] `make check`
    - [ ] `make test`
    - [ ] `make test`
    - [ ] `make lint`

- [ ] only when you're done, and all checks pass:
    - [ ] set in [task name].md
        - [ ] set `<passing>true</passing>`
    - [ ] CRUCIAL: run `/bin/bash .ralph/task_switch.sh` to indicate that you want to switch task. 
                This can be when you're done or just when you want to switch (e.g. for going to subtask)
    - [ ] commit with: `task finished [task name]: [insert text]`
        - [ ] include summary of what was done in commit message (evidence for tests completing, challenges faced during
          implementation)
        - [ ] Make sure to add all files, please do not forget commiting any file when completing a task, also the stuff in .ralph
    - [ ] push commits with: `git push`
    - [ ] Write any learnings/surprises to AGENTS.md
    - [ ] QUIT IMMEDIATELY
