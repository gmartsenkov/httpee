# httpee

Run HTTP requests from TOML templates.

## Install

```
cargo install --path .
```

## Quick start

Create an `httpee.toml` config in your project directory:

```toml
dirs = ["requests"]

[variables]
token = "my-api-token"
```

Create a template in `requests/health.toml`:

```toml
name = "Health check"
description = "Check if the API is up"

[request]
url = "https://api.example.com/health"
```

Run it:

```
httpee requests/health
```

## Usage

```
httpee [TEMPLATE] [KEY=VALUE...] [OPTIONS]
```

Running `httpee` with no arguments lists all discovered templates:

```
$ httpee
example/ping
    Example.com

example/users/create
    Users - Create (Create an individual user)

example/users/show
    Users - Show (Show an individual user)
```

### Options

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Print status, headers, and body |
| `--status` | Print only the status code |
| `--headers` | Print response headers |
| `--no-color` | Disable syntax highlighting |

### Variable overrides

Override template variables from the command line:

```
httpee example/users/create id=42 token=secret
```

## Templates

Templates are TOML files that define an HTTP request:

```toml
name = "Users - Create"
description = "Create an individual user"

[variables]
id = 100
token = "123"

[request]
url = "https://httpbin.org/anything/{{id}}"
method = "POST"
body = """
{
  "name": "Bob",
  "id": "{{id}}"
}
"""

[request.headers]
content-type = "application/json"
authorization = "Bearer {{token}}"
```

### Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `name` | No | `""` | Display name shown when listing templates |
| `description` | No | `""` | Description shown when listing templates |
| `variables` | No | `{}` | Template-local variables |
| `request.url` | Yes | | Request URL (supports template rendering) |
| `request.method` | No | `GET` | HTTP method |
| `request.body` | No | `""` | Request body (supports template rendering) |
| `request.headers` | No | `{}` | Headers as key-value pairs (values support template rendering) |

### Template rendering

httpee uses [minijinja](https://github.com/mitsuhiko/minijinja) for variable interpolation:

```toml
[request]
url = "https://api.example.com/users/{{id}}"
```

Access environment variables with the `env()` function:

```toml
[request.headers]
authorization = "Bearer {{ env('API_TOKEN') }}"
```

### Variable resolution order

Variables are resolved with the following priority (highest first):

1. CLI overrides (`httpee template id=42`)
2. Template `[variables]`
3. Additional config variables
4. Main `httpee.toml` `[variables]`

## Configuration

httpee looks for `httpee.toml` in the current directory.

```toml
dirs = ["requests", "requests/users"]
additional-configs = ["httpee.local.toml"]

[variables]
id = "5"
token = "default-token"
```

### Fields

| Field | Description |
|-------|-------------|
| `dirs` | Directories to search for `.toml` template files |
| `additional-configs` | Extra config files to merge (useful for local overrides) |
| `variables` | Global variables available to all templates |

### Local overrides

Use `additional-configs` to layer in local settings that you can gitignore:

```toml
# httpee.local.toml
[variables]
token = "my-personal-token"
secret = "dev-secret"
```

## Shell completions

httpee supports dynamic tab completion for template names across bash, zsh, and fish.

### bash

Add to `~/.bashrc`:

```bash
eval "$(COMPLETE=bash httpee)"
```

### zsh

Add to `~/.zshrc`:

```zsh
eval "$(COMPLETE=zsh httpee)"
```

### fish

Add to `~/.config/fish/config.fish`:

```fish
COMPLETE=fish httpee | source
```

Completions are dynamic - they discover templates from the `httpee.toml` config in your current directory at completion time.

## Response output

Responses are syntax-highlighted based on `Content-Type` (JSON, HTML, XML). Highlighting is automatically disabled when piping output.

```
$ httpee example/ping --verbose
200 OK

content-type: text/html; charset=UTF-8
content-length: 1256

<!doctype html>
<html>
...
```

Get just the status code:

```
$ httpee example/ping --status
200
```

Get just the headers:

```
$ httpee example/ping --headers
content-type: text/html; charset=UTF-8
content-length: 1256
```
