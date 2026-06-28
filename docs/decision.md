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
    "short_url":   "https://<api-host>/aB3xY",   // convenience full URL, displayed as-is
    "short_code":  "aB3xY",
    "original_url": "https://example.gov.sg/x"
  }

4xx (validation / bad request):
  { "error": { "message": "user-facing message" } }
```

- `short_url` is the full link clients display; `short_code` is the raw code.
- The short link is served by the **API host** (`GET /:code` → 302), not the web
  app's origin. The mock currently uses `window.location.origin`; #12b replaces
  this with the API base (`VITE_API_BASE_URL`).

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
