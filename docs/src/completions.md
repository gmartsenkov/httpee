# Shell completions

httpee supports dynamic tab completion for template names across bash, zsh,
and fish. Completions are dynamic: they discover templates from the
`httpee.toml` config in your current directory at completion time.

## bash

Add to `~/.bashrc`:

```bash
eval "$(COMPLETE=bash httpee)"
```

## zsh

Add to `~/.zshrc`:

```zsh
eval "$(COMPLETE=zsh httpee)"
```

## fish

Add to `~/.config/fish/config.fish`:

```fish
COMPLETE=fish httpee | source
```
