# Configuration

For basic configuration instructions, see [this documentation](https://developers.openai.com/codex/config-basic).

For advanced configuration instructions, see [this documentation](https://developers.openai.com/codex/config-advanced).

For a full configuration reference, see [this documentation](https://developers.openai.com/codex/config-reference).

## Karpathy Mode

Karpathy Mode is an opt-in explanation style inspired by [Andrej Karpathy's October 1, 2026 post](https://x.com/karpathy/status/2105819303471976479). It asks Codex to use plain English, helpful diagrams, interactive HTML explanations, and visual explainer videos when the task and available tools call for them. Simple tasks still get a concise text answer. This is guidance for the model, not a guarantee of a particular response format or formal ASD-STE100 compliance.

Enable **Karpathy Mode** in `/experimental`, or add this to `~/.codex/config.toml`:

```toml
[features]
karpathy_mode = true
```

For one session, run `codex --enable karpathy_mode`. Use `codex features enable karpathy_mode` to save the setting, or `codex features disable karpathy_mode` to turn it off. Changes saved through `/experimental` take effect on the next turn in the current chat. Command-line and profile overrides continue to take precedence.

The mode adds explanation guidance alongside your existing instructions. It does not install media tools, enable paid services, or publish artifacts. Generated local artifacts are linked in the response so you can open them.

## Lifecycle hooks

Admins can set top-level `allow_managed_hooks_only = true` in
`requirements.toml` to ignore user, project, and session hook configs while
still allowing managed hooks from requirements and managed config layers. This
setting is only supported in `requirements.toml`; putting it in `config.toml`
does not enable managed-hooks-only mode.
