# Smell 3: Wrong Place-ism

This smell is about knowledge living in the wrong module.

The classic bad shape is:

- A talks to B
- B talks back to A
- both pass similar request or state types around
- the top-level runtime becomes the courier for everyone else's internals

This is closely related to bad config boundaries. When validation has not been finished, raw or half-validated config tends to spray through runtime, worker startup, and helper functions, and each layer starts compensating in its own way.

