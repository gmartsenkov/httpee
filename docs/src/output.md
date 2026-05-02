# Response output

Responses are syntax-highlighted based on `Content-Type` (JSON, HTML, XML).
Highlighting is automatically disabled when piping output.

## Verbose

Show status, headers, timing, size, and body:

```
$ httpee example/ping --verbose
  → GET  http://example.com/
  ✓ 200 OK   32ms   528 B

  ── headers ───────────────────────────────────────
    content-type   text/html
    ...

  ── body ──────────────────────────────────────────
  <!doctype html>
  ...
```

## Status only

```
$ httpee example/ping --status
200
```

Useful in scripts: the exit code is non-zero on transport failures, and the
status line is plain enough to feed into `if` / `case`.

## Headers only

```
$ httpee example/ping --headers
content-type: text/html; charset=UTF-8
content-length: 1256
```
