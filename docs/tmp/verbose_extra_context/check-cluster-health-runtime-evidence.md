# Runtime evidence collected for this docs cycle

This file captures the exact runtime evidence requested by K2, including failures and empty output.

## Requested command

`cargo run --bin pgtuskmasterctl -- --base-url http://127.0.0.1:18081 --output json ha state`

## Observed result

```text
transport error: error sending request for url (http://127.0.0.1:18081/ha/state)
```

## Interpretation constraints

- This is evidence that the command did not reach a responsive API at `127.0.0.1:18081` during this cycle.
- This is not evidence that the command syntax is wrong.
- This is not evidence that the endpoint path is wrong.
- This is not evidence about cluster health itself, because no response payload was received.

## Requested command

`docker compose --env-file .env.docker.example -f docker/compose/docker-compose.cluster.yml ps --format json`

## Observed result

- command exited successfully
- stdout was empty

## Interpretation constraints

- The empty stdout means this exact command did not produce container JSON lines during this cycle.
- Do not fabricate example container rows from this run.
- If documentation includes compose inspection, it should either present the command without promising concrete output values or explicitly mark any sample output as illustrative rather than observed.
