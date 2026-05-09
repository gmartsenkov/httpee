# Introduction

`httpee` runs HTTP requests from TOML templates. You define each request once
as a small TOML file and execute it by name from the shell, with variable
interpolation, environment-aware secrets, and per-template overrides.

![httpee demo](https://raw.githubusercontent.com/gmartsenkov/httpee/master/docs/assets/demo.gif?v=4)

## When you'd use it

- You hit the same set of endpoints across local/staging/prod and want a single
  source of truth for the request shape.
- You want HTTPie-style ergonomics, but with the request committed to the repo
  rather than scrolling back through your shell history.
- You want shell completion that knows your project's templates.

## What's in these docs

- [Installation](./installation.md): Homebrew, shell installer, Cargo, prebuilt binaries
- [Usage](./usage.md): CLI flags, listing templates, variable overrides
- [Templates](./templates.md): schema, rendering, built-in functions
- [Configuration](./configuration.md): `httpee.toml`, additional configs, local overrides
- [Shell completions](./completions.md): bash, zsh, fish setup
- [Response output](./output.md): verbose, status-only, headers-only modes
- [Sharing requests](./sharing.md): export a template as a curl one-liner

## Links

- Repository: <https://github.com/gmartsenkov/httpee>
- Issue tracker: <https://github.com/gmartsenkov/httpee/issues>
- Releases: <https://github.com/gmartsenkov/httpee/releases>
