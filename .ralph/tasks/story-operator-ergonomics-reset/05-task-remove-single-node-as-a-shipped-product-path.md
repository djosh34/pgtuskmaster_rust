## Task: Remove Single-Node As A Shipped Product Path <status>not_started</status> <passes>false</passes>

<priority>low</priority>

<description>
**Goal:** Stop teaching and shipping single-node mode as if it were a first-class product path. The higher-order goal is to align the repo with its actual value proposition: three-node HA operation, not a one-node pseudo-cluster that adds conceptual noise to onboarding, config, docs, and compose assets.

**Scope:**
- Treat the new three-node local quickstart from task 01 as the replacement path. This task should remove single-node from the shipped/operator story once the better default exists, not before.
- Remove single-node Docker compose assets, alternate local compose variants, single-node quickstart/tutorial material, and single-node README positioning from the normal operator journey.
- Remove single-node-specific public examples and commands that keep the docs split between "toy one-node mode" and "real HA mode".
- Audit whether single-node-specific runtime or trust behavior remains in public product documentation or tests.
- If code still contains true single-node product support that materially complicates the user-facing model, remove it as part of this task or create tightly scoped follow-on tasks for any deeper internal deletion that cannot be completed safely in one pass.
- Do not preserve single-node as a hidden default path just because it already exists; this is greenfield and backward compatibility is explicitly not required.

**Context from research:**
- The README still advertises a single-node cluster before the three-node HA cluster.
- There is a dedicated tutorial for single-node setup and a dedicated compose file for it.
- The user explicitly stated that single-node support should likely be removed because it weakens the operator experience and makes the product look more confusing than it is.
- Keeping both one-node and three-node paths active increases doc surface, config surface, and testing surface for little operator value.

**Expected outcome:**
- The public product story starts at three nodes.
- The README, tutorial flow, and shipped Docker assets no longer encourage single-node mode.
- The canonical local product path is the same one-compose, file-based, secure-by-default three-node stack defined by task 01.
- Any remaining single-node support is either deleted or clearly demoted out of the beginner/operator path with a concrete reason.

</description>

<acceptance_criteria>
- [ ] The README no longer presents single-node mode as a primary quickstart or prerequisite to the real HA flow.
- [ ] The shipped single-node Docker compose asset and single-node tutorial are deleted or explicitly demoted out of the normal docs navigation with a written justification.
- [ ] The canonical local product path exposes exactly one operator-facing Docker Compose file.
- [ ] Public examples, local configs, and onboarding material converge on the three-node HA path.
- [ ] Any remaining single-node-specific code or docs that complicate the public product model are removed or captured in narrowly scoped follow-on tasks.
- [ ] The resulting public operator experience answers "how do I run PGTuskMaster?" with one default topology: a 3-node HA cluster.
- [ ] `make check` — passes cleanly
- [ ] `make test` — passes cleanly (default suite; excludes only ultra-long tests moved to `make test-long`)
- [ ] `make lint` — passes cleanly
- [ ] If this task impacts ultra-long tests (or their selection): `make test-long` — passes cleanly (ultra-long-only)
</acceptance_criteria>
