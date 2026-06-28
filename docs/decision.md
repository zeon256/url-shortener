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
