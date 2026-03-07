# Architecture Assurance

This section is for readers who want confidence boundaries, not only workflow instructions. The operator chapters tell you what to do. The lifecycle chapters tell you why the node is currently in a given phase. This section goes one level deeper and asks a stricter question: what safety argument is the implementation actually trying to uphold, what evidence does that argument depend on, and where does the system intentionally stop short of promising more.

Use the chapter map by question:

- read [Safety Invariants](./safety-invariants.md) when you need the short list of conditions the design tries hard not to violate
- read [Decision Model](./decision-model.md) when you need to understand how PostgreSQL state, DCS trust, lifecycle phase, and operator intent combine into a bounded next action
- read [DCS Data Model and Write Paths](./dcs-data-model.md) when you need to know which records exist, who owns them, and why stale coordination evidence can mean different things depending on the key
- read [Runtime Topology and Boundaries](./runtime-topology.md) when you need to understand which worker owns observation, decision, execution, and operator-facing publication
- read [Safety Case](./safety-case.md) when you need the cautious structured claim for why the design reduces split-brain risk
- read [Tradeoffs and Limits](./tradeoffs-limits.md) when you need the operational cost side of the same story

Treat this section as a review aid, not marketing text. If a page here sounds more absolute than the implementation and tests justify, the page is wrong. The right outcome from this section is a more realistic mental model: which safety properties are central, what assumptions they rely on, and what kinds of ambiguity the system deliberately answers with restraint rather than speed.
