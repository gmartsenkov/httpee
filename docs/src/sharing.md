# Sharing requests

Sometimes you want to hand a teammate the exact request — not the template, not
your `httpee.toml` — just a one-liner they can paste into a terminal or a bug
report. Pass `--as=<format>` and httpee will render the request in that format
and exit without sending it.

## curl

```
$ httpee users/create name=Bob --as=curl
curl \
  --request POST \
  --url 'https://api.example.com/orgs/acme/users' \
  --header 'authorization: Bearer abc123' \
  --header 'content-type: application/json' \
  --data-raw '{"name": "Bob", "email": null}'
```

> **Heads up:** the snippet contains anything you'd send on the wire — including
> auth tokens. Strip them before pasting into a public channel.
