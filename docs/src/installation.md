# Installation

Pick whichever method matches your platform. After installing, jump to
[Shell completions](./completions.md) to get tab completion for your project's
templates.

## Homebrew

```sh
brew install gmartsenkov/tap/httpee
```

## Shell (macOS / Linux)

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/gmartsenkov/httpee/releases/latest/download/httpee-installer.sh | sh
```

## PowerShell (Windows)

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/gmartsenkov/httpee/releases/latest/download/httpee-installer.ps1 | iex"
```

## Cargo

```sh
cargo install httpee
```

## Prebuilt binaries

Download a release archive from the
[releases page](https://github.com/gmartsenkov/httpee/releases/latest)
and place the `httpee` binary on your `PATH`. Builds are published for:

- `aarch64-apple-darwin` (Apple Silicon macOS)
- `x86_64-apple-darwin` (Intel macOS)
- `aarch64-unknown-linux-gnu` (ARM64 Linux)
- `x86_64-unknown-linux-gnu` (x86_64 Linux)

## From source

```sh
git clone https://github.com/gmartsenkov/httpee
cd httpee
cargo install --path .
```

## Verify

```sh
httpee --version
```
