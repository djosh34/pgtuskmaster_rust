## Task: Reduce Code Loop <status>in_progress</status> <passes>false</passes> <priority>high</priority>

<description>
Your only goal is to reduce code and clean up the codebase.

use improve-code-boundaries skill to find improvements

Be BOLD! Do LARGE refactors for improvements.
YOU MUST REDUCE CODE. WE KNOW FROM ANALYSIS THAT MORE THAN 50% IS OVER-ABSTRACTION AND DUPLICATIVE CODE
Be smart, see the bigger picture, look across many files, follow chains of dependencies step by step.
The best refactors are found with the lsp going to definitions on and on, and writing up the entire chain of dependency.
Aim for Large refactors first, HIGH-HANGING FRUITS!

End goal, remove 50-75% of code from `src/` and `tests/`.

Check current code reduction since commit `.ralph/git_diff_lines.txt` via:
`bash .ralph/git_diff_lines_since.sh`

Check current total lines with:
`bash .ralph/git_current_lines.sh`

Start number of lines:

- `src/`: 28,744 lines across 94 git-tracked files
- `tests/`: 9,331 lines across 53 git-tracked files
- Total (`src/` + `tests/`): 38,075 lines across 147 git-tracked files
</description>

YOU ARE ONLY DONE IFF THE NUMBER OF LINES WAS REDUCED MORE THAN 50% and thus src/ and tests/ have less than 19k lines of code.
DO TRY TO GO LOWER IF POSSIBLE!
DO COMMIT BETWEEN EACH IMPROVEMENT!

Plan: `.ralph/tasks/story-config-v2-direct-cfg-collapse/05-reduce-loop_plans/61-collapse-config-parser-boundaries-and-path-guarantees.md`

Plan:

<steps>
- [ ] If fresh (based on progress history), untick all boxes, remove existing path to plan file
- [ ] Find a potential plan if it exists, otherwise find new ones
- [ ] Choose one improvement and write a plan, add the path to that plan inside task_file, remove potential plan file after plan written
- [ ] Do improvement
- [ ] Validate if improvement actually reduced lines, otherwise go replan: no line count reduction = failure
- [ ] test&validate new code
- [ ] task_switch + commit everything (including .ralph etc) + push
- [ ] if not lower than 19k lines, DO NOT SET passes:true
</steps>
<acceptance_criteria>
- [ ] `make check`
- [ ] `make lint`
- [ ] `make test`
- [ ] `make test-long`
</acceptance_criteria>
