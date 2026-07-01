# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/zeon256/url-shortener/compare/v0.1.7...v0.2.0) - 2026-07-01

### Added

- *(api)* moka bounded redirect cache

### Other

- release v0.2.0

## [0.2.0](https://github.com/zeon256/url-shortener/compare/v0.1.7...v0.2.0) - 2026-07-01

### Added

- *(api)* moka bounded redirect cache

## [0.1.7](https://github.com/zeon256/url-shortener/compare/v0.1.6...v0.1.7) - 2026-07-01

### Other

- *(api)* integration tests + run them in CI (Postgres service container)
- release v0.1.6

## [0.1.6](https://github.com/zeon256/url-shortener/compare/v0.1.5...v0.1.6) - 2026-07-01

### Fixed

- *(api)* ensure that we use a disallowed list instead of just 1 disallowed host

### Other

- release v0.1.5

## [0.1.6](https://github.com/zeon256/url-shortener/compare/v0.1.5...v0.1.6) - 2026-07-01

### Fixed

- *(api)* ensure that we use a disallowed list instead of just 1 disallowed host

## [0.1.5](https://github.com/zeon256/url-shortener/compare/v0.1.4...v0.1.5) - 2026-07-01

### Fixed

- ensure release-plz includes web

## [0.1.4](https://github.com/zeon256/url-shortener/compare/v0.1.3...v0.1.4) - 2026-07-01

### Other

- release v0.1.3

## [0.1.3](https://github.com/zeon256/url-shortener/compare/v0.1.2...v0.1.3) - 2026-06-30

### Added

- *(api)* use mimalloc

### Other

- release v0.1.2

## [0.1.3](https://github.com/zeon256/url-shortener/compare/v0.1.2...v0.1.3) - 2026-06-30

### Added

- *(api)* use mimalloc

## [0.1.2](https://github.com/zeon256/url-shortener/compare/v0.1.1...v0.1.2) - 2026-06-30

### Added

- *(api)* url validation

### Fixed

- *(api)* replace HOST with CORS_ALLOWED_ORIGINS allowlist

## [0.1.1](https://github.com/zeon256/url-shortener/compare/v0.1.0...v0.1.1) - 2026-06-29

### Fixed

- *(api)* add missing cors

### Other

- complete github actions
- dockerfiles + docker-compose

## [0.1.0](https://github.com/zeon256/url-shortener/releases/tag/v0.1.0) - 2026-06-29

### Added

- *(web)* wire frontend to real API
- *(api)* endpoints + JSON errors

### Other

- fix release-plz git-only workspace

## [0.1.1](https://github.com/zeon256/url-shortener/compare/v0.1.0...v0.1.1) - 2026-06-28

### Added

- *(api)* sqlx pool + links migration
- *(api)* config + tracing + error handling

### Other

- repo skeleton & workspace setup
