# Email send notes

Mail helper used:

- [reply.sh](/home/joshazimullah.linux/work_mounts/patroni_rewrite/receive_mail/reply.sh)

Invocation notes:

- The helper requires `<to_address>` and `<original_subject>`.
- The receive-mail prompt available in this execution did not preserve the original sender/subject metadata after the process finished.
- Because of that, the email was sent via the helper to the configured notification address from `../receive_mail/.env` using a descriptive review subject: `Post-Greenfield HA Refactor Option Review`.
- This fallback is recorded explicitly here so the artifact set does not pretend the original thread metadata was available when it was not.

Command shape used:

```bash
/bin/bash ../receive_mail/reply.sh "<configured notification address>" "Post-Greenfield HA Refactor Option Review" <<'EOF_REPLY_EMAIL'
...natural-language review summary...
EOF_REPLY_EMAIL
```
