# Changelog

## 0.2.0 (2026-11-03)

### Added

- Search Arch Linux, AUR, Alpine Linux, NuGet, Nixpkgs, GitLab.com, WinGet, and the mise registry.  mise registry searches include shorthand aliases such as `rg`.
- Identify package, command, repository, manual-page, and tool-alias matches in text and terminal output.  Exact matches have an `:exact` label.
- Include `matched_kinds` in each JSON provider result and `kind` and `exact` in each match.

### Changed

- Show exact matches first and retain all detected exact match kinds when results are hidden by `--limit`, including `--limit 0`.
- Use the matched name in the `detail` field of `taken` results.  Package descriptions remain in individual match details.
- Report incomplete GitHub and GitLab searches as `unknown` when no exact match was found.

Nixpkgs, WinGet, and mise searches require their respective commands.  WinGet checks exact matches only.  Missing commands and failed searches report `unknown`.  See the README for provider scopes and requirements.

## 0.1.2 (2026-10-24)

### Security

- Update `rustls`, `rustls-webpki`, and `webpki-roots`, including the certificate distrust fixes for CVE-2026-111943.  Published binaries now include these TLS dependency updates.

### Changed

- Update `ureq` to 3.5.0.  Proxy URL credentials are decoded once; escape literal percent signs as `%25`.
- Ignore untracked files when reporting whether a build is dirty.
