# DCS Draft 1

Compass classification: cognition + application.

## Scope

This draft describes the DCS subsystem in `src/dcs/`.

## Candidate structure

- Purpose
- Module layout
- Key namespace
- Store trait and watch events
- Cached state
- Trust evaluation
- Worker step behavior
- Error surfaces

## Notes

- `DcsStore` defines health, KV read/write/delete, conditional create, and watch draining.
- The supported key space is `/<scope>/member/<member_id>`, `/<scope>/leader`, `/<scope>/switchover`, `/<scope>/config`, and `/<scope>/init`.
- `DcsCache` stores member records, leader record, switchover request, config, and init lock.
- Trust states are `full_quorum`, `fail_safe`, and `not_trusted`.
- `refresh_from_etcd_watch` applies `put`, `delete`, and synthesized `reset` events into the cache.
