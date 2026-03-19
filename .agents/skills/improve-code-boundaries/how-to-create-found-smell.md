Add a set of smells found to `.ralph/tasks/smells/[smell_set_name].md`

It must look like:

```text
## Smell Set: [smell_set_name] <status>not_started</status> <passes>false</passes>

Please refer to skill 'improve-code-boundaries' to see what smells there are.

Inside dirs:
- `<path_to_code_dir_1>`
- `<path_to_code_dir_2>`

Solve each smell:

---
- [ ] Smell x, [what is smell x] 
[some text describing the things found of smell x]

code:
[all code snippets found to be smell x]

---
- [ ] Smell y, [what is smell y] 
[some text describing the things found of smell y]

code:
[all code snippets found to be smell y]

---
etc,etc with one full smell by smell list of smells found.
you can find multiple instances of the same smell, in one place, list all instances you found.
e.g.

- [ ] smell 1, ...
...
- [ ] smell 1, ...
...
- [ ] smell 1, ...
...
- [ ] smell 2, ...
...


```
