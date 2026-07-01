# Security

Notes on how this repo limits supply-chain and application risk. Larger design tradeoffs live in [docs/decision.md](docs/decision.md); smaller implementation calls in [docs/minutiae.md](docs/minutiae.md).

## Pinned GitHub Actions (commit SHA)

Third-party GitHub Actions do not ship lockfiles. Floating refs such as `@v5` can be retargeted on the tag, so a workflow can silently pick up new code on the next run.

Every external action in this repo is pinned to a full commit SHA, with the intended release tag in a trailing comment:

```yaml
- uses: actions/checkout@93cb6efe18208431cddfb8368fd83d5badbf9bfd # v5
```

That matches [GitHub's hardening guidance](https://docs.github.com/en/actions/security-for-github-actions/security-guides/security-hardening-for-github-actions#using-third-party-actions) for third-party actions.

**Where:** `.github/workflows/` (`qc.yml`, `website.yml`, `rust-build.yml`, `release-plz.yml`) and `.github/actions/setup-rust/action.yml` (which pins its own upstream actions).

**Updating:** When bumping an action, set the SHA to the commit that matches the release tag you intend to run, not just the tag name. The comment documents which release the SHA came from.

**Local actions:** `./.github/actions/setup-rust` is repo-owned composite code. It still pins the third-party steps it calls (`dtolnay/rust-toolchain`, `swatinem/rust-cache`, and others).

## Locked application dependencies

Rust and Node dependencies are lockfile-driven:

- `cargo test` / `cargo build` use `Cargo.lock`.
- CI runs `pnpm install --frozen-lockfile` for the frontend.

That covers crates and npm packages. SHA pinning covers the CI actions gap.

## Workflow permissions

Pull-request workflows (`qc.yml`, `website.yml`) use `permissions: contents: read` only.

Release and deploy jobs in `release-plz.yml` request broader access where needed (for example `contents: write` to cut releases, GHCR push, Railway deploy via environment secrets). Checkout steps that do not need to push use `persist-credentials: false`.

## Container image scanning

Release builds push the API image to GHCR, then `trivy-scan` runs Trivy (also SHA-pinned) on that image. The job fails on unfixed `HIGH` or `CRITICAL` findings.

**Where:** `.github/workflows/release-plz.yml` (`trivy-scan` job)

## Application hardening

The API rejects unsafe or self-defeating input before storage:

| Control | What it blocks |
| --- | --- |
| Scheme allowlist | Non-HTTP(S) URLs (`javascript:`, `file:`, `data:`, etc.) |
| Owned-host blocklist | Short links that point back at this service's own domains |
| Exact-origin CORS | Browser calls only from configured full origins (see [docs/minutiae.md](docs/minutiae.md#exact-origin-cors-allowlist)) |

Integration tests in `apps/shortener_api/tests/api.rs` and unit tests in `apps/shortener_api/src/routes.rs` cover scheme rejection, disallowed hosts, and normalization behavior.

## Out of scope (for now)

This is a public URL shortener, not an authenticated product. There are no user accounts, API keys, or rate limits. Redirect targets are user-supplied; treat shared short links like any other untrusted redirect.

`/healthz` is a liveness check only (HTTP 200, no database probe). Railway uses it to confirm the process is up, not that PostgreSQL is reachable.
