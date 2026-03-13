# Introduction

pgtuskmaster is a PostgreSQL HA manager built around three ideas:

- PostgreSQL facts should be observed directly
- cluster coordination should come from DCS-backed state
- operators should have one obvious read surface

That last point now shapes the public API directly:

- `GET /state` for all current runtime observation
- `POST /switchover` to request a planned leadership change
- `DELETE /switchover` to clear that request

The operator-facing CLI, `pgtm`, is a renderer over that model. It reads one seed `/state` document and formats it for status, connection helpers, and switchover workflows.

## Why the API Was Collapsed

Older layouts split observation across multiple overlapping read surfaces. That made the product feel like several APIs plus a smart client that stitched them together.

The current shape is intentionally simpler:

- one verbose read document
- one control noun
- no separate debug history subsystem
- no peer API fanout for normal CLI operation

## Facts and Interpretation

`GET /state` includes both raw facts and derived interpretation:

- `pg` is the node's current PostgreSQL observation
- `dcs` is the node's current DCS-backed cache and trust state
- `ha` is the current authority projection, role intent, worldview, and ordered commands

This keeps operators from having to guess which endpoint is "more real". The node tells one story in one document.
