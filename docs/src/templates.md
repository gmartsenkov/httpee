# Templates

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

## Fields

| Field | Required | Default | Description |
|-------|----------|---------|-------------|
| `name` | No | `""` | Display name shown when listing templates |
| `description` | No | `""` | Description shown when listing templates |
| `variables` | No | `{}` | Template-local variables |
| `request.url` | Yes | | Request URL (supports template rendering) |
| `request.method` | No | `GET` | HTTP method |
| `request.body` | No | `""` | Request body (supports template rendering) |
| `request.headers` | No | `{}` | Headers as key-value pairs (values support template rendering) |

## Template rendering

httpee uses [minijinja](https://github.com/mitsuhiko/minijinja) for variable
interpolation:

```toml
[request]
url = "https://api.example.com/users/{{id}}"
```

## Built-in functions

| Function | Description | Example |
|----------|-------------|---------|
| `env(name)` | Read an environment variable. Errors if unset. | `{{ env('API_TOKEN') }}` |
| `bearer(token)` | Format a bearer auth header value. | `{{ bearer(token) }}` → `Bearer <token>` |
| `basic(user, pass)` | Format a basic auth header value (base64-encoded). | `{{ basic(user, pass) }}` → `Basic <base64>` |

```toml
[request.headers]
authorization = "{{ bearer(env('API_TOKEN')) }}"
```

## Variable resolution order

Variables are resolved with the following priority (highest first):

1. CLI overrides (`httpee template id=42`)
2. Template `[variables]`
3. Additional config variables
4. Main `httpee.toml` `[variables]`
