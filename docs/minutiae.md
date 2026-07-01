# Minutiae

Small decisions that do not need a full ADR. Larger tradeoffs live in [decision.md](decision.md). Security notes live in [SECURITY.md](../SECURITY.md).

## Base62 vs Base64 vs Others

**Why:** Base64 have unfriendly characters for URLs which can break browsers and QR codes. Base62 is a good compromise between compactness and URL friendliness. Base58 is also URL friendly but has a smaller keyspace (58^7 ~ 1.5 trillion) which increases collision risk at scale. As for hashing functions like SHA256, while they guarantee uniquness, they defeat the purpose of having a URL shortener because the resulting code is too long for a URL. Base62 is a good compromise between compactness and URL friendliness. 

**Where:** `apps/shortener_api/src/shortcode.rs` (`CODE_LEN`)

## Seven-character base62 short codes

**Why:** Short enough for URLs and QR codes. The keyspace is 62^7 (~3.5 trillion). Random assignment stays safe at expected scale: ~0.001% collision risk at 100k links, ~13% at 1M (birthday bound kicks in around ~2M). If two codes collide anyway, the insert retry loop generates another.

**Where:** `apps/shortener_api/src/shortcode.rs` (`CODE_LEN`)

## Generation of base62 on the server instead of elsewhere

**Why:** This makes it easy to scale horizontally for the server side. However we are still bottlenecked on Postgres for inserts but reads are fine because we can have read replicas. Perhaps a future extension would be to shard the database and have each server instance write to a different shard. This would allow for more horizontal scaling.

**Where:** `apps/shortener_api/src/routes.rs` (`POST /api/v1/shorten`)

## RedHat UBI images over Alpine Linux

**Why:** Alpine's musl libc is slower compared to standard glibc and maintaining alpine based images are a pain because of the lack of packages and the need to compile from source. RedHat UBI images are based on glibc and have a lot more packages available. They are also small and secure + matained by RedHat. The UBI images are also compatible with RHEL and CentOS which is a plus for enterprise customers.

**Where:** `apps/shortener_api/Dockerfile`, `apps/web/Dockerfile`

## Permanent redirect (301)

**Why:** A short code maps to one stored URL for its lifetime (D2). `301 Moved Permanently` is what services like bit.ly use: the short link is the stable identifier, the destination is fixed. Browsers may cache the redirect and skip repeat lookups. CDN stays off on the API so the edge does not cache redirect targets.

**302 instead?** A `302 Found` tells the client the redirect may change and to keep using the short URL on later visits rather than treating the destination as permanent. That is the usual pick when codes might be reassigned or destinations edited. We do not support reassigning destinations today, so 301 fits. If we add admin overrides or takedowns, revisit and likely switch to 302 (or 307).

**Where:** `apps/shortener_api/src/routes.rs` (`redirect`)

## CI Docker builds are different from local (not multistage)

**Why:** Building dockerfiles that are multistage on CI platforms like Github Actions is slow as hell. Building the binaries and frontend on the CI and then copying them into the final image is significantly faster. The CI build is also more reproducible because it uses the same base image for all builds. Also the binaries can be reused to upload to the final image. The CI build is also more secure because it does not have to run as root and does not have to install any packages.

**Where:** `.github/workflows/qc.yml` (`docker build`), `apps/shortener_api/Dockerfile`, `apps/web/Dockerfile`

## Recent history capped at five entries

**Why:** The UI is a "recent links" panel, not a full account history. Five rows fit the layout without scrolling; IndexedDB stays bounded.

**Where:** `apps/web/src/lib/history.ts` (`MAX_ENTRIES`)

## Exact-origin CORS allowlist

**Why:** Host suffix matching would accept `shorter.inve.rs.evil.com`. The allowlist matches full origins only.

**Where:** `apps/shortener_api/src/routes.rs` (`cors_layer`)

## Redirect cache default of 1000 entries

**Why:** Per-process, in-memory, best-effort cache to skip repeat Postgres reads on hot codes. Bounded so memory stays predictable; Postgres remains source of truth.

**Where:** `REDIRECT_CACHE_CAPACITY`, `apps/shortener_api/src/routes.rs`

## oxlint instead of ESLint

**Why:** One devDependency, fast runs, enough rules for a small SolidJS app. No flat-config or plugin stack to maintain.

**Where:** `apps/web/package.json`, `apps/web/.oxlintrc.json`

## pnpm workspace at the repo root

**Why:** The web app and Railway IaC share one Node toolchain. Root `package.json` pins pnpm and the Railway CLI for deploy scripts and CI.

**Where:** `package.json`, `pnpm-workspace.yaml`, `.railway/railway.ts`

## mimalloc as the API global allocator

**Why:** Drop-in allocator with good throughput for a long-lived HTTP server. Low cost to enable; easy to remove if it ever causes trouble.

**Where:** `apps/shortener_api/src/main.rs`, `apps/shortener_api/Cargo.toml`

## Custom argh fork

**Why:** The upstream `argh` crate does not have env fallback support for the option parser. The fork adds that feature and is published as argh_env

**Where:** `apps/shortener_api/Cargo.toml` (`argh_env`), `apps/shortener_api/src/cli.rs` (option parsing)


## `Box<str>` vs `String`

**Why:** `Box<str>` is a heap-allocated string slice. It is smaller than `String` because it does not store capacity, and it is immutable. The API does not need to mutate the short code or the destination URL after creation, so `Box<str>` is a better fit.

## `&'static [&'static T]` vs `Vec<T>` and `String` vs `&'static str` for CLI args 

**Why:** `&'static [&'static T]` and `&'static str` is used for the CLI parsing because these are used as long as the program is alive. They are made static by leaking the heap-allocated memory. Unlike `Vec<T>` and `String`, they might require calling unnecessary `.clone()`. There is no need to use them because these config is immutable and will not change during the program's lifetime.
