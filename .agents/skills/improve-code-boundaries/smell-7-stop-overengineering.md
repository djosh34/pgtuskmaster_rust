# Smell 7: Stop Overengineering

This smell is intentionally broad.

Sometimes the problem is not only "wrong place" or "wrong type." Sometimes the problem is that the solution is simply too elaborate.

Rules:

- simpler solution is better
- simpler state machine is better
- less remembered state is better
- fewer timing concepts are better
- fewer features are better
- the target question is: what is the minimum needed to keep `test-long` happy?

Do not defend extra machinery just because it sounds robust. Prove it is needed.
