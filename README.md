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

Limit matches per provider:

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
- `arch`
- `aur`
- `alpine`
- `nuget`
- `nixpkgs`
- `gitlab`
- `winget`
- `mise`

GitHub search requires the `gh` command to be installed and authenticated.  Homebrew search uses `brew` when available, then falls back to the Homebrew API.  GitLab searches public projects on GitLab.com.

Nixpkgs search requires `nix` with the `nix-command` and `flakes` features enabled and searches the `nixpkgs` flake for the current system.  It may download and evaluate Nixpkgs on first use; the search times out after 30 seconds.  mise search uses `mise registry --json`, including registry aliases such as `rg`.  It checks registry shorthands, rather than every identifier supported by mise backends or personal tool aliases.

WinGet search requires `winget` and checks exact package IDs, display names, monikers, and declared commands in the `winget` source.  Matches are case-sensitive, as defined by `winget --exact`.  It does not search for similar names.  Source agreements must already be accepted.  Alpine searches the edge branch for x86_64 packages and commands in `/bin`, `/sbin`, `/usr/bin`, and `/usr/sbin`.  Arch and AUR check package names.

Missing commands, timeouts, and failed requests produce `unknown`, rather than `available`.

## Match kinds

Each result identifies what matched:

- `package`: package name or identifier.
- `command`: an executable path, a command manual page, or a declared WinGet command.
- `repository`: repository name, independent of its owner or namespace.
- `manual-page`: a FreeBSD base manual page outside command sections 1, 6, and 8.
- `tool-alias`: a mise registry shorthand or alias, or a WinGet moniker.

An `:exact` suffix marks exact matches, for example `[command:exact]`.  Similar matches have a kind label without that suffix.  The provider's `taken` line lists all exact match kinds found, even when `--limit 0` hides the individual results.  Exact matches appear first.

`taken` means the name is already in use within the searched scope.  A repository match does not prevent creating a repository under another owner.  `available` means no exact match was found within that provider's scope; it does not reserve a name or guarantee trademark availability.

JSON retains `status`, `detail`, and `matches`, and adds `matched_kinds` to each provider result and `kind` and `exact` to each match.

## Author

Copyright (c) 2026 Akinori Musha.

Licensed under the MIT license.  See `LICENSE` for details.

Visit the [GitHub Repository](https://github.com/knu/avail) for the latest information.
