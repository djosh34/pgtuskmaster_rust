# Explanation

This chapter provides discursive treatment of pgtuskmaster's design and behavior. These pages illuminate why the system works the way it does, exploring architectural decisions, failure modes, and decision-making processes.

- [Introduction](introduction.md) - Overview of the system's purpose, safety model, and runtime shape
- [Architecture](architecture.md) - Core design principles, component organization, trust model, and HA phase machine
- [Failure Modes and Recovery Behavior](failure-modes.md) - How the system responds to component failures and its trust-based safety mechanisms
- [HA Decision Engine](ha-decision-engine.md) - How the engine turns world snapshots into HA phases and decisions through trust gates and phase-driven logic
