# Decisions

## D1 — Frontend/API contract for shortening (assumed)

**Status:** Assumed — reconcile with #8 before #12b merges.
**Date:** 2026-06-28
**Context:** The frontend (#12) is built against a local mock so Track B can
proceed in parallel with the backend (Track A: #4 → … → #8). To keep the eventual
wiring (#12b) mechanical, both tracks target one agreed contract. Field names
mirror the `links` table from #5.

### Endpoint

```
POST /api/v1/shorten
Request:  { "url": "https://example.gov.sg/x" }

201 Created:
  {
    "short_code":  "aB3xY",
    "original_url": "https://example.gov.sg/x"
  }

4xx (validation / bad request):
  { "error": { "message": "user-facing message" } }
```

- `short_url` is the full link clients display; `short_code` is the raw code.
- The short link is served by the **API host** (`GET /:code` → 302), not the web
  app's origin. The client targets `${VITE_BACKEND_URL}/api/v1/shorten`, where
  `VITE_BACKEND_URL` defaults to `http://localhost:4002` for local dev (see #20).

### `links` table (from #5)

| column         | type          | notes                          |
| -------------- | ------------- | ------------------------------ |
| `id`           | `UUID` PK     | `DEFAULT gen_random_uuid()`    |
| `short_code`   | `TEXT`        | `NOT NULL UNIQUE`              |
| `original_url` | `TEXT`        | `NOT NULL`                     |
| `created_at`   | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()`       |
| `updated_at`   | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()`       |

### Open items to reconcile with #8

- **Path:** this doc assumes `POST /api/v1/shorten` (versioned). Issue #8 currently
  says `POST /links`. Pick one path before #12b wires the real `fetch`.
- **Error shape:** confirm #8 emits `{ "error": { "message": … } }` and the status
  codes for invalid URL, unsupported scheme, and self-shortening rejection.

## D2 — Deduplicate links by `original_url` via a unique index on `md5(original_url)`

**Status:** Accepted — implemented in migration `0002_dedup_original_url.sql`.
**Date:** 2026-06-29
**Context:** `insert_link_with_retry` previously generated a fresh short code on every
request, so shortening the **same URL twice produced two different codes**. We now
deduplicate: if a URL was already shortened, return the existing row
(`201 Created` for a new link, `200 OK` when reusing). To make "the same URL" robust, the
parsed `url::Url` is normalized first (strip fragment, trim trailing slash, sort query
params, on top of the crate's scheme/host lowercasing and default-port handling), and that
normalized serialization is what gets stored and dedup-keyed.

Dedup must be **race-safe**: two concurrent requests for the same URL must not create two
rows. That requires a `UNIQUE` constraint on `original_url` so an atomic
`INSERT … ON CONFLICT DO NOTHING` + `SELECT` resolves new-vs-existing in one round trip.

### Why hash the URL instead of a plain `UNIQUE (original_url)`

A btree index entry cannot exceed **~2700 bytes** (1/3 of the default 8 KB page —
specifically 2704 bytes for btree v4). A plain `UNIQUE` index on `original_url TEXT` indexes
the full URL string, so **any URL longer than that fails to INSERT** with
`index row size … exceeds btree version 4 maximum 2704`. URLs legitimately get long (tracking
/ campaign query strings, signed-URL tokens, etc.), and before this change there was no index
on `original_url`, so long URLs inserted fine — a plain unique constraint would *regress* that.

Indexing a **hash** of the URL instead keeps the index key a small fixed size regardless of
URL length, so dedup works for arbitrarily long URLs. The raw `original_url` is still stored
in full in its (un-indexed) `TEXT` column; only the *index key* is hashed.

### Why `md5`, not `sha256`

First attempt used `sha256(convert_to(original_url, 'UTF8'))`. Postgres rejected it at
migration time: **functions in an index expression must be `IMMUTABLE`**, and `convert_to`
is only `STABLE` (its result depends on the database encoding). `sha256` itself is fine, but
it only accepts `bytea`, so it needs the `convert_to` text→bytea step — which is the
non-immutable part.

`md5(text)` is `IMMUTABLE` **and** takes `text` directly, so it sidesteps `convert_to`
entirely and the functional index is valid:

```sql
CREATE UNIQUE INDEX links_original_url_md5_idx ON links (md5(original_url));
```

md5's cryptographic weakness (forgeable collisions) is acceptable for a *dedup key*: the only
way to exploit it is to craft two different URLs with the same md5, and the sole effect is
that the attacker's own second URL dedups to their own first one — no impact on other users.
If stronger collision resistance is ever wanted without `convert_to`, the alternatives are an
`IMMUTABLE` SQL wrapper around `sha256(convert_to(…))` or pgcrypto's `digest(text, 'sha256')`.

### Consequences

- A short-code collision (a *different* URL drawing an already-used code) is a violation of
  the separate `short_code` unique constraint — **not** the `md5(original_url)` arbiter — so
  it still surfaces as an error and is handled by the existing code-regeneration retry loop.
- One URL maps to exactly one short code for its lifetime (no per-request vanity codes).
- The shorten response returns the **normalized** `original_url`, not the raw request string.
  Because normalization (and dedup) can change what the caller sent — `https://EXAMPLE.com/a/#x`
  → `https://example.com/a` — echoing it back is a *confirmation of the canonical URL we stored
  and will redirect to*, not a redundant repeat of the input. The value is highest exactly when
  it differs from what was submitted.

## D3 — Railway-deploy-everything (Infrastructure as Code + CDN)

**Status:** Accepted — tracked in #15.
**Date:** 2026-06-30
**Context:** #1 originally specified Kubernetes for the backend and Cloudflare Pages for the
frontend. We're consolidating both workloads onto a single Railway project, defined via
Railway's Infrastructure as Code (`.railway/railway.ts`), with Railway's built-in CDN in front
of the frontend service. This replaces the Kubernetes + Cloudflare Pages plan (#48 is closed in
favor of #15).

### Why Railway instead of Kubernetes

- Initially I wanted to deploy to Kubernetes because I already have an existing cluster running on my RPI5, however
the config of that is a private repo and makes it messy + the overhead of setting up again is not pragmatic
- Managed Postgres, builds, and deploys with no manifests/Ingress/controller boilerplate — a
  small project doesn't justify a k8s control plane.
- Faster local→prod loop: the same Dockerfile that builds locally ships to Railway; no separate
  image-build/push/manifest pipeline to maintain.
- One platform, one deploy surface, one auth model. Keeps the codebase small enough to defend
  in the follow-up interview.
- Railway is not required for local evaluation — Docker Compose / `cargo run` / `pnpm dev` remain
  the local story, and the repo's README documents that split.

### Why drop Cloudflare Pages for the frontend

Usually for all my projects I deploy on Cloudflare Pages because it is a great tool + its free but
decided to go against it this time for a simpler deployment story. The frontend is a single-page app (SPA) with hashed assets, so the CDN is the only part of Cloudflare Pages we actually needed.

Railway's CDN covers what we'd have used from Cloudflare Pages:

- **Edge caching by content-type**, not file extension — Vite's hashed `dist/assets/*.js|.css`
  outputs match Railway's static-asset content-type list and are cached at edge automatically, even
  with no `Cache-Control` from Caddy.
- **Global POPs** route each visitor to the nearest edge location; cache hits never reach the origin.
- **Edge compression** (Brotli/gzip) is handled at the edge, so Caddy doesn't need `encode gzip`.
- **Purge-on-deploy** invalidates stale content after each successful deploy, so a fresh frontend
  build can't reference outdated hashed assets.
- Keeping the frontend on the same platform as the backend means one IaC file, one deploy surface,
  and one auth/CI path — strictly simpler than splitting frontend to a separate platform.

The frontend is served by **Caddy** (`apps/web/Caddyfile`) running on Railway, with
`try_files {path} /index.html` for SPA routing; Railway terminates TLS at the edge, so the
Caddyfile uses `auto_https off`.

### Why Infrastructure as Code (`.railway/railway.ts`)

`.railway/railway.ts` is the single source of truth for project topology: services, Postgres
database, environment variables, custom domains (defaulting to `shorter.inve.rs` for the frontend
and `s.inve.rs` for the backend, overridable via `SHORTENER_WEB_DOMAIN` and
`SHORTENER_API_DOMAIN`), replicas, and canvas groups. The Railway CLI diffs the file against the
live environment:

- `railway config plan` — preview the diff (additive review, secrets redacted in output)
- `railway config apply` — apply after confirmation; rejects a stale plan if the environment
  changed in between, so concurrent edits can't silently overwrite each other
- `railway config plan --detailed-exit-code` — opt-in exit code 2 on drift, for gating CI

The `.railway/railway.ts` file is the working alternative to `railway.json`/`railway.toml`
(Config as Code). A service cannot be managed by both — IaC owns the whole project here.

### IaC–CDN ownership gap (important caveat)

`.railway/railway.ts` **does not (yet) own CDN/edge settings.** Verified against `railway@3.4.1`
(latest at time of writing): `ServiceConfigInput`, `ServiceNetworking`, and `DeployConfig`
expose **no** edge/CDN fields (no `cdn`, `edgeConfig`, `htmlCaching`, `defaultTtl`,
`purgeOnDeploy`, or `staleWhileRevalidate`). A repo-wide grep of the SDK bundle for those terms
returns zero matches. The `Edge` type that *does* exist is the internal resource-reference graph
edge, unrelated to Railway's edge/CDN product.

Corroborating signal: the `railway cdn` CLI (PR railwayapp/cli#982) implements CDN via dedicated
GraphQL mutations (`enableServiceCdn`, `updateServiceEdgeConfig`, `purgeServiceCache`) — a
separate API surface from the `config plan/apply` graph/patch system. IaC and CDN are two
different API fronts; the DSL has simply not been wired to the edge-config mutations yet.

**Consequence:** CDN config lives outside `.railway/railway.ts` and is applied as a documented
manual CLI step after `railway config apply`:

```bash
railway cdn enable  --service web
railway cdn update  --service web --html-caching force --purge-on-deploy all
```

Chosen settings:

- HTML caching **Force** — Caddy serves `index.html` with no cache header; Force lets the CDN
  cache it via the Default TTL. (Alternative was Auto + emitting `Cache-Control` for HTML from
  Caddy; Force was chosen for a simpler Caddyfile.)
- Purge-on-deploy **all** — every deploy flushes assets too, accepting the extra origin fetches
  in exchange for a simpler mental model. Safer when asset hashes ever change.
- CDN **off** on `shortener-api` — 302 redirects and JSON API responses aren't cacheable without
  explicit `max-age`; leaving it off avoids accidental caching of redirect targets.

**Known risk to verify during #15:** whether `railway config apply` preserves or wipes
unmanaged edge settings when it patches a service. The importer "omits platform defaults," which
*suggests* IaC leaves unset fields alone — but this is unverified. If `config apply` ever recreates
or resets the `web` service, the `railway cdn enable/update` step must be re-run. #15 should verify
this behavior and document the handoff (a small post-apply script if needed).

**DSL stability:** the IaC DSL is explicitly experimental ("expect rough edges while the DSL
settles"). Pin the `railway` SDK version and re-check on each release — `config pull` may start
emitting CDN fields in a future minor, at which point the manual CLI step can move into
`.railway/railway.ts`.

### Generated support files

`railway config init` / `railway config pull` also create `.railway/README.md` and
`.agents/skills/railway-config/SKILL.md`. Whether to commit them or gitignore them as
regenerable scaffold is decided during #15 implementation.

### Consequences

- Backend production target: Railway (Dockerfile + `/health`). Replaces Kubernetes.
- Frontend production target: Railway (Caddy + Railway CDN). Replaces Cloudflare Pages.
- Production Postgres is Railway's managed Postgres (`postgres("postgres")` in `.railway/railway.ts`),
  not the Docker Compose container used locally.
- Backend image selection is supplied to `.railway/railway.ts` with `SHORTENER_API_TAG`, defaulting
  to the current pinned fallback. CI sets this from the release version it wants deployed.
- Frontend/backend custom domains are supplied to `.railway/railway.ts` with
  `SHORTENER_WEB_DOMAIN` and `SHORTENER_API_DOMAIN`, defaulting to the production domains.
- CI uses `railway config plan --detailed-exit-code` to detect drift; deploys use
  `railway config apply` (interactive, or `--yes --confirm-destructive` in CI).
- Kubernetes manifests and Cloudflare Pages config are intentionally **not** part of the repo.
