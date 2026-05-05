# avail

Check tool name availability and uniqueness.

`avail` checks tool names across package registries, operating system package indexes, and repositories.

## Installation

```console
% cargo install avail-cli --locked
```

Or install a prebuilt binary from GitHub Releases:

```console
% curl --proto '=https' --tlsv1.2 -LsSf https://github.com/knu/avail/releases/latest/download/avail-cli-installer.sh | sh
```

If you use `cargo-binstall`:

```console
% cargo binstall avail-cli
```

On Windows:

```console
> powershell -ExecutionPolicy Bypass -c "irm https://github.com/knu/avail/releases/latest/download/avail-cli-installer.ps1 | iex"
```

## Usage

```console
% avail <name>
```

Limit the search to specific providers:

```console
% avail -p cargo -p npm my-name
```

Limit fuzzy matches per provider:

```console
% avail --limit 10 my-name
```

Print JSON output:

```console
% avail --json my-name
```

Available providers:

- `cargo`
- `npm`
- `pypi`
- `gem`
- `debian`
- `freebsd-base`
- `freebsd-ports`
- `homebrew`
- `github`

GitHub search requires the `gh` command to be installed and authenticated.  Homebrew search uses `brew` when available, then falls back to the Homebrew API.

## Author

Copyright (c) 2026 Akinori Musha.

Licensed under the MIT license.  See `LICENSE` for details.

Visit the [GitHub Repository](https://github.com/knu/avail) for the latest information.
