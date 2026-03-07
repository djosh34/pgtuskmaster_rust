---
name: opencode-openrouter-ask
description: Query another model through the local opencode CLI with OpenRouter. Use when the user asks to consult another model, asks for an opencode answer, or wants a Kimi K2 thinking response sourced only from prompt text passed through stdin.
---

# opencode OpenRouter Ask

Use this skill when you need a second model answer through `opencode`.

The wrapper script reads the full prompt from stdin. Put every piece of context into the heredoc. Do not rely on files, tools, or unstated workspace context being available to the other model.

Default model: `openrouter/kimi-k2-thinking`

Override the model only when the user asks:

```bash
OPENCODE_MODEL='openrouter/some-other-model' \
.agents/skills/opencode-openrouter-ask/query-opencode.sh <<<"$OPENCODE_INPUT"
```

Preferred calling pattern:

```bash
OPENCODE_INPUT=$(cat <<'OPENCODE_EOF'
Answer using only the information in this prompt.
Do not ask to inspect files, browse, run tools, or fetch more context.
If the prompt is missing something required, say exactly what is missing.

Question:
...
OPENCODE_EOF
)

.agents/skills/opencode-openrouter-ask/query-opencode.sh <<<"$OPENCODE_INPUT"
```

Notes:

- The bundled `opencode.json` only enables `openrouter`.
- The bundled config denies all tool permissions with `"*": "deny"`.
- The script does not post-process output; it streams raw `opencode` output.
