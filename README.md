# avail

Check tool name availability and uniqueness.

`avail` checks tool names across package registries, operating system package indexes, and repositories.

## Installation

```console
% cargo install --path .
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
