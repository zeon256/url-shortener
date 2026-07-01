# Decisions

Small decisions live in [minutiae.md](minutiae.md). Security notes live in [SECURITY.md](../SECURITY.md). This file is for larger tradeoffs.

## D1: Frontend/API contract for shortening

**Status:** Accepted; implemented in [#8](https://github.com/zeon256/url-shortener/issues/8) and wired in [#20](https://github.com/zeon256/url-shortener/issues/20).
**Date:** 2026-06-28
**Context:** The frontend ([#12](https://github.com/zeon256/url-shortener/issues/12)) was built in parallel with the backend (Track A: [#4](https://github.com/zeon256/url-shortener/issues/4) → … → [#8](https://github.com/zeon256/url-shortener/issues/8)). Both tracks targeted one agreed contract before [#20](https://github.com/zeon256/url-shortener/issues/20) connected the UI to the real API. Field names mirror the `links` table from [#5](https://github.com/zeon256/url-shortener/issues/5).

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
- The short link is served by the **API host** (`GET /:code` → 301 Moved Permanently), not the web
  app's origin. The client targets `${VITE_BACKEND_URL}/api/v1/shorten`, where
  `VITE_BACKEND_URL` defaults to `http://localhost:4002` for local dev (see [#20](https://github.com/zeon256/url-shortener/issues/20)).
- Re-shortening the same normalized URL returns `200 OK` with the existing row (see D2).

### `links` table (from [#5](https://github.com/zeon256/url-shortener/issues/5))

| column         | type          | notes                          |
| -------------- | ------------- | ------------------------------ |
| `id`           | `UUID` PK     | `DEFAULT gen_random_uuid()`    |
| `short_code`   | `TEXT`        | `NOT NULL UNIQUE`              |
| `original_url` | `TEXT`        | `NOT NULL`                     |
| `created_at`   | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()`       |
| `updated_at`   | `TIMESTAMPTZ` | `NOT NULL DEFAULT now()`       |

Validation errors use `{ "error": { "message": "…" } }` with `400 Bad Request` (unsupported scheme, self-shorten), `404 Not Found` (unknown code), or `422 Unprocessable Entity` (malformed URL). Covered in `apps/shortener_api/tests/api.rs`.

## D2: Deduplicate links by `original_url` via a unique index on `md5(original_url)`

**Status:** Accepted; implemented in migration `0002_dedup_original_url.sql`.
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

A btree index entry cannot exceed **~2700 bytes** (1/3 of the default 8 KB page,
specifically 2704 bytes for btree v4). A plain `UNIQUE` index on `original_url TEXT` indexes
the full URL string, so **any URL longer than that fails to INSERT** with
`index row size … exceeds btree version 4 maximum 2704`. URLs legitimately get long (tracking
/ campaign query strings, signed-URL tokens, etc.), and before this change there was no index
on `original_url`, so long URLs inserted fine. A plain unique constraint would *regress* that.

Indexing a **hash** of the URL instead keeps the index key a small fixed size regardless of
URL length, so dedup works for arbitrarily long URLs. The raw `original_url` is still stored
in full in its (un-indexed) `TEXT` column; only the *index key* is hashed.

### Why `md5`, not `sha256`

First attempt used `sha256(convert_to(original_url, 'UTF8'))`. PostgreSQL rejected it at
migration time: **functions in an index expression must be `IMMUTABLE`**, and `convert_to`
is only `STABLE` (its result depends on the database encoding). `sha256` itself is fine, but
it only accepts `bytea`, so it needs the `convert_to` text→bytea step, which is the
non-immutable part.

`md5(text)` is `IMMUTABLE` **and** takes `text` directly, so it sidesteps `convert_to`
entirely and the functional index is valid:

```sql
CREATE UNIQUE INDEX links_original_url_md5_idx ON links (md5(original_url));
```

md5's cryptographic weakness (forgeable collisions) is acceptable for a *dedup key*: the only
way to exploit it is to craft two different URLs with the same md5, and the sole effect is
that the attacker's own second URL dedups to their own first one, with no impact on other users.
If stronger collision resistance is ever wanted without `convert_to`, the alternatives are an
`IMMUTABLE` SQL wrapper around `sha256(convert_to(…))` or pgcrypto's `digest(text, 'sha256')`.

### Consequences

- A short-code collision (a *different* URL drawing an already-used code) is a violation of
  the separate `short_code` unique constraint (**not** the `md5(original_url)` arbiter), so
  it still surfaces as an error and is handled by the existing code-regeneration retry loop.
- One URL maps to exactly one short code for its lifetime (no per-request vanity codes).
- The shorten response returns the **normalized** `original_url`, not the raw request string.
  Because normalization (and dedup) can change what the caller sent (`https://EXAMPLE.com/a/#x`
  → `https://example.com/a`). Echoing it back is a *confirmation of the canonical URL we stored
  and will redirect to*, not a redundant repeat of the input. The value is highest exactly when
  it differs from what was submitted.

## D3: Railway deployment (Infrastructure as Code)

**Status:** Accepted; tracked in [#15](https://github.com/zeon256/url-shortener/issues/15).
**Date:** 2026-06-30
**Context:** [#1](https://github.com/zeon256/url-shortener/issues/1) originally specified Kubernetes for the backend and Cloudflare Pages for the
frontend. We're consolidating both workloads onto a single Railway project, defined via
Railway's Infrastructure as Code (`.railway/railway.ts`), with optional CDN on the frontend service. This replaces the Kubernetes + Cloudflare Pages plan ([#48](https://github.com/zeon256/url-shortener/issues/48) is closed in
favor of [#15](https://github.com/zeon256/url-shortener/issues/15)).

### Why Railway instead of Kubernetes

- Initially I wanted to deploy to Kubernetes because I already have an existing cluster running on my RPI5, however
the config of that is a private repo and makes it messy + the overhead of setting up again is not pragmatic
- Managed PostgreSQL, builds, and deploys with no manifests/Ingress/controller boilerplate; a
  small project doesn't justify a k8s control plane.
- Faster local→prod loop: the same Dockerfile that builds locally ships to Railway; no separate
  image-build/push/manifest pipeline to maintain.
- One platform, one deploy surface, one auth model. Keeps the codebase small enough to defend
  in the follow-up interview.
- Railway is not required for local evaluation. Docker Compose / `cargo run` / `pnpm dev` remain
  the local story, and the repo's README documents that split.

### Why drop Cloudflare Pages for the frontend

Usually I deploy frontends on Cloudflare Pages, but this time I wanted one platform for API and web. The app is a Vite SPA with hashed static assets; Railway's CDN on the `web` service is enough for that. Caddy serves the built files (`apps/web/Caddyfile`, `try_files {path} /index.html`); Railway terminates TLS, so the Caddyfile uses `auto_https off`.

### CDN and IaC

`.railway/railway.ts` defines services, env, and domains. It does **not** include Railway CDN settings today, so CDN for `web` (if enabled) is configured in the Railway dashboard. Keep CDN **off** on `shortener-api` so redirects and JSON are not cached at the edge.

### Why Infrastructure as Code (`.railway/railway.ts`)

`.railway/railway.ts` is the single source of truth for project topology: services, PostgreSQL
database, environment variables, custom domains (defaulting to `shorter.inve.rs` for the frontend
and `s.inve.rs` for the backend, overridable via `SHORTENER_WEB_DOMAIN` and
`SHORTENER_API_DOMAIN`), replicas, and canvas groups. The Railway CLI diffs the file against the
live environment:

- `railway config plan`: preview the diff (additive review, secrets redacted in output)
- `railway config apply`: apply after confirmation; rejects a stale plan if the environment
  changed in between, so concurrent edits can't silently overwrite each other
- `railway config plan --detailed-exit-code`: opt-in exit code 2 on drift, for gating CI

The `.railway/railway.ts` file is the working alternative to `railway.json`/`railway.toml`
(Config as Code). A service cannot be managed by both; IaC owns the whole project here.

### Generated support files

`railway config init` / `railway config pull` also create `.railway/README.md` and
`.agents/skills/railway-config/SKILL.md`. Whether to commit them or gitignore them as
regenerable scaffold is decided during [#15](https://github.com/zeon256/url-shortener/issues/15) implementation.

### Consequences

- Backend production target: Railway (Dockerfile + `/healthz`). Replaces Kubernetes.
- Frontend production target: Railway (Caddy + static assets). Replaces Cloudflare Pages.
- Production PostgreSQL is Railway's managed PostgreSQL (`postgres("Postgres")` in `.railway/railway.ts`),
  not the Docker Compose container used locally.
- Backend image selection is supplied to `.railway/railway.ts` with `SHORTENER_API_TAG`, defaulting
  to the current pinned fallback. CI sets this from the release version it wants deployed.
- Frontend/backend custom domains are supplied to `.railway/railway.ts` with
  `SHORTENER_WEB_DOMAIN` and `SHORTENER_API_DOMAIN`, defaulting to the production domains.
- CI uses `railway config plan` to preview deploy changes; release deploys use
  `railway config apply --yes` without `--confirm-destructive`, so destructive plans fail instead
  of deleting resources automatically.
- Kubernetes manifests and Cloudflare Pages config are intentionally **not** part of the repo.
