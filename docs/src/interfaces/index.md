# Interfaces

Interfaces are the operator contract surfaces for control and observation.

There are two interaction modes:

- Observe current HA state and trust posture.
- Submit or cancel planned transition intent.

```mermaid
flowchart LR
  Op[Operator or automation] --> CLI[pgtuskmasterctl]
  Op --> API[Node API]
  CLI --> API
  API --> DCS[(DCS intent records)]
```

Use this section when you need concrete endpoint and CLI workflow behavior.
