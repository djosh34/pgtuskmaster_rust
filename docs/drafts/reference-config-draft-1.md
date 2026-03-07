# Config Draft 1

Compass classification: cognition + application.

## Scope

This draft describes the runtime configuration surface implemented in `src/config/`.

## Candidate structure

- Purpose
- Module layout
- Top-level runtime sections
- Input shapes
- Loading and normalization
- Validation surfaces
- Error surfaces

## Notes

- `load_runtime_config` reads a TOML document, probes `config_version`, rejects `v1`, and normalizes `v2` input into `RuntimeConfig`.
- `RuntimeConfig` contains `cluster`, `postgres`, `dcs`, `ha`, `process`, `logging`, `api`, and `debug`.
- `InlineOrPath` and `SecretSource` model inline content versus path-backed values.
- Validation covers required strings and paths, absolute binary paths, timeout ranges, DCS and HA invariants, TLS/auth invariants, file sink self-ingest prevention, and `dcs.init.payload_json` JSON decoding into `RuntimeConfig`.
