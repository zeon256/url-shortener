<h1 align="center">url-shortener</h1>

<p align="center">
  <img src="./images/logo.webp" alt="logo" width="300">
</p>

<p align="center">
  <a href="https://github.com/zeon256/url-shortener/actions/workflows/qc.yml"><img src="https://github.com/zeon256/url-shortener/actions/workflows/qc.yml/badge.svg" alt="QC"></a>
  <a href="https://github.com/zeon256/url-shortener/actions/workflows/website.yml"><img src="https://github.com/zeon256/url-shortener/actions/workflows/website.yml/badge.svg" alt="Website"></a>
  <a href="https://github.com/zeon256/url-shortener/releases"><img src="https://img.shields.io/github/v/release/zeon256/url-shortener" alt="Release"></a>
  <a href="https://shorter.inve.rs"><img src="https://img.shields.io/badge/demo-shorter.inve.rs-0ea5e9" alt="Live demo"></a>
</p>

## Table of Contents

- [Features](#features)
- [Scope](#scope)
- [Development](#development)
- [Running end-to-end (Docker Compose)](#running-end-to-end-docker-compose)
- [Deployment](DEPLOY.md)
- [Building from source](BUILDING.md)
- [Decisions](docs/decision.md)
- [Minutiae](docs/minutiae.md)
- [Security](SECURITY.md)

---

## Features

- Shorten URLs; `GET /:code` redirects
- Same URL always gets the same code
- Web UI: copy, QR codes, recent links (responsive, a11y basics)
- Rust API, SolidJS frontend, PostgreSQL

## Scope

| Requirement | How |
| --- | --- |
| Shorten + redirect | `POST /api/v1/shorten`, `GET /:code` |
| Web UI | [shorter.inve.rs](https://shorter.inve.rs) |
| Persistence | PostgreSQL + migrations |
| Local dev | Docker Compose ([compose.yml](compose.yml)) + multistage Dockerfiles |
| Deployed | Railway via IaC ([`.railway/railway.ts`](.railway/railway.ts), [DEPLOY.md](DEPLOY.md)) |
| Development | Work in [pull requests](https://github.com/zeon256/url-shortener/pulls); [QC](.github/workflows/qc.yml) and [Website](.github/workflows/website.yml) run on PR |
| Tests | Unit + integration tests ([qc.yml](.github/workflows/qc.yml)) |

Also: release automation, URL dedup/normalization. Rationale in [docs/decision.md](docs/decision.md); smaller calls in [docs/minutiae.md](docs/minutiae.md); security notes in [SECURITY.md](SECURITY.md).

## Development

Feature work lands via pull requests ([history](https://github.com/zeon256/url-shortener/pulls?q=is%3Apr+is%3Aclosed)). PRs run path-filtered CI; they do not deploy. Merge to `main` triggers [release-plz](.github/workflows/release-plz.yml) (version bump, GHCR images, Trivy scan, Railway apply). See [DEPLOY.md](DEPLOY.md) for the prod flow.

## Running end-to-end (Docker Compose)

> [!TIP]
> If you want to run the app end-to-end without installing Rust, Node.js, or pnpm use this!


### Run the full stack

From the repo root:

```bash
docker compose --profile app up --build
```

The first run builds both images (multistage Dockerfiles) and waits for PostgreSQL to become healthy before starting the API.

When the logs settle, open:

- **Frontend** at [http://localhost:8080](http://localhost:8080): shorten a URL in the UI
- **Backend** at [http://localhost:8000/healthz](http://localhost:8000/healthz): should return HTTP 200

The frontend is built with `VITE_BACKEND_URL=http://localhost:8000` (see `.env.example`), so the browser talks to the API on your host loopback.

```bash
docker compose --profile app up --build -d
```

Logs: `docker compose --profile app logs -f`
Stop: `docker compose --profile app down`

### Compose profiles

[`compose.yml`](compose.yml) splits services into two groups:

| Service | Profile | Port (default) | Purpose |
| --- | --- | --- | --- |
| `postgres` | *(none, always starts)* | `5432` | Database |
| `api` | `app` | `8000` | Rust backend |
| `web` | `app` | `8080` | SolidJS frontend (Caddy) |

- **`docker compose up`**: starts **PostgreSQL only**. Use this when running the backend and frontend on your host during development.
- **`docker compose --profile app up`**: starts **PostgreSQL + API + web**, i.e. the full stack end-to-end.

Optional env overrides live in [`.env.example`](.env.example); copy to `.env` if you need different ports or credentials:

```bash
cp .env.example .env
```

The API keeps a bounded, per-process, best-effort redirect cache controlled by `REDIRECT_CACHE_CAPACITY` (default `1000`). PostgreSQL remains the source of truth; the cache only avoids repeated database reads for recently resolved short codes.

### Common pitfalls and debugging (Docker)

- On some OS you can spawn containers with same ports and have no errors. Check that you do not have any existing Docker containers running on the same ports. You can check this with `docker ps` and stop any existing containers with `docker stop <container_id>`. Likewise this applies to locally installed tool that might interfere with the ports.

- If you are running on Windows and have WSL2 installed, make sure that you have the latest version of Docker Desktop installed. You can check this by opening Docker Desktop and checking for updates. If you are running on Linux, make sure that you have the latest version of Docker installed. You can check this by running `docker --version` and comparing it to the latest version on the Docker website.

## Building from source

See [`BUILDING.md`](BUILDING.md) for toolchain prerequisites, building the backend/frontend, cross-compilation, and troubleshooting.

## Deployment

Production runs on [Railway](https://railway.com). The deployment runbook, release CI flow, Railway token notes, and custom-domain configuration live in [`DEPLOY.md`](DEPLOY.md).

Rationale and deployment tradeoffs live in [`docs/decision.md`](docs/decision.md) (D3). Smaller decisions are in [`docs/minutiae.md`](docs/minutiae.md). Security notes (pinned Actions, image scan, API controls) are in [`SECURITY.md`](SECURITY.md).
