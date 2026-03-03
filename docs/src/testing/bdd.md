# BDD Coverage

BDD tests validate system behavior from the perspective of external users of the interfaces.

```mermaid
flowchart LR
  Spec[BDD feature] --> API[HTTP requests]
  API --> Node[Node runtime]
  Node --> Result[Observed state]
  Result --> Spec
```

The goal is not maximum endpoint coverage; it is to validate the most important “contract surfaces”:
- safety restrictions
- allowed operator actions
- consistent state reporting

