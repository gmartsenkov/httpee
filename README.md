A lightweight, terminal-native HTTP client for developers who'd rather not run Postman. Requests live as plain TOML files alongside your code, so they version, diff, and review like everything else in the repo.

![httpee demo](https://raw.githubusercontent.com/gmartsenkov/httpee/master/docs/assets/demo.gif?v=3)

## Quick start

Install httpee following the instructions at <https://gmartsenkov.github.io/httpee/>.

Then create an `httpee.toml` config in your project directory:

```toml
dirs = ["users"]

[variables]
org = "acme"
```

Create a template in `users/create.toml`:

```toml
name = "Create user"
description = "POST a new user"

[variables]
name = "Default Name"

[request]
url = "https://api.example.com/orgs/{{ org }}/users"
method = "POST"
body = """
{
  "name": "{{ name }}",
  "email": "{{ email }}"
}
"""

[request.headers]
content-type = "application/json"
authorization = "{{ bearer(env('API_TOKEN')) }}"
```

Run it, with the token sourced from the environment and per-call values passed
as CLI overrides:

```
API_TOKEN=$(pass api/example) httpee users/create name=Bob email=bob@acme.io
```

## Documentation

Full docs are at <https://gmartsenkov.github.io/httpee/book/>: usage, template
schema, configuration, shell completions, and response formatting.
