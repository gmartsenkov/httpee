# Usage

```
httpee [TEMPLATE] [KEY=VALUE...] [OPTIONS]
```

Running `httpee` with no arguments lists all discovered templates:

```
$ httpee
  ❯ example/ping             Example.com
  ❯ example/users/create     Create an individual user
  ❯ example/users/show       Show an individual user
```

Run a specific template by name:

```
$ httpee example/ping
```

## Options

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Print status, headers, and body |
| `--status` | Print only the status code |
| `--headers` | Print response headers |
| `--no-color` | Disable syntax highlighting |

## Variable overrides

Override template variables from the command line:

```
httpee example/users/create id=42 token=secret
```

CLI overrides take priority over both template-local and config-level
variables. See [Templates → Variable resolution order](./templates.md#variable-resolution-order)
for the full precedence chain.

## `init` subcommand

Scaffold a new `httpee.toml` in the current directory:

```
httpee init
```
