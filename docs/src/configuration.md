# Configuration

httpee looks for `httpee.toml` in the current directory.

```toml
dirs = ["requests", "requests/users"]
additional-configs = ["httpee.local.toml"]

[variables]
id = "5"
token = "default-token"
```

## Fields

| Field | Description |
|-------|-------------|
| `dirs` | Directories to search for `.toml` template files |
| `additional-configs` | Extra config files to merge (useful for local overrides) |
| `variables` | Global variables available to all templates |

## Local overrides

Use `additional-configs` to layer in local settings that you can gitignore:

```toml
# httpee.local.toml
[variables]
token = "my-personal-token"
secret = "dev-secret"
```

This is the recommended pattern for tokens and other secrets: keep
`httpee.toml` committed to the repo and `httpee.local.toml` in your
`.gitignore`.
