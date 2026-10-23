# Changelog

## 0.1.2 (2026-10-24)

### Security

- Update `rustls`, `rustls-webpki`, and `webpki-roots`, including the certificate distrust fixes for CVE-2026-111943.  Published binaries now include these TLS dependency updates.

### Changed

- Update `ureq` to 3.5.0.  Proxy URL credentials are decoded once; escape literal percent signs as `%25`.
- Ignore untracked files when reporting whether a build is dirty.
