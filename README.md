# url-shortener

## Running end-to-end (Docker Compose)

> [!TIP]
> If you want to run the app end-to-end without installing Rust, Node.js, or pnpm.
> This is **not** the local development workflow — see [Building from source](#building-from-source) below.

### Compose profiles

[`compose.yml`](compose.yml) splits services into two groups:

| Service | Profile | Port (default) | Purpose |
| --- | --- | --- | --- |
| `postgres` | *(none — always starts)* | `5432` | Database |
| `api` | `app` | `8000` | Rust backend |
| `web` | `app` | `8080` | SolidJS frontend (Caddy) |

- **`docker compose up`** — starts **Postgres only**. Use this when running the backend and frontend on your host during development.
- **`docker compose --profile app up`** — starts **Postgres + API + web**, i.e. the full stack end-to-end.

Optional env overrides live in [`.env.example`](.env.example); copy to `.env` if you need different ports or credentials:

```bash
cp .env.example .env
```

### Run the full stack

From the repo root:

```bash
docker compose --profile app up --build
```

The first run builds both images (multistage Dockerfiles) and waits for Postgres to become healthy before starting the API.

When the logs settle, open:

- **Frontend:** [http://localhost:8080](http://localhost:8080) — shorten a URL in the UI
- **Backend:** [http://localhost:8000/healthz](http://localhost:8000/healthz) — should return HTTP 200

The frontend is built with `VITE_BACKEND_URL=http://localhost:8000` (see `.env.example`), so the browser talks to the API on your host loopback.

Run detached (background):

```bash
docker compose --profile app up --build -d
```

Follow logs: `docker compose --profile app logs -f`

Stop and remove containers (keeps the Postgres volume):

```bash
docker compose --profile app down
```

### Common pitfalls and debugging (Docker)

- On some OS you can spawn containers with same ports and have no errors. Check that you do not have any existing Docker containers running on the same ports. You can check this with `docker ps` and stop any existing containers with `docker stop <container_id>`. Likewise this applies to locally installed tool that might interfere with the ports.

- If you are running on Windows and have WSL2 installed, make sure that you have the latest version of Docker Desktop installed. You can check this by opening Docker Desktop and checking for updates. If you are running on Linux, make sure that you have the latest version of Docker installed. You can check this by running `docker --version` and comparing it to the latest version on the Docker website.

## Building from source 

Toolchain Prerequisites:

- Rust Compiler
- pnpm and Node.js

> [!NOTE]
> At the current moment if you are compiling for x86_64 Linux, [`.cargo`](.cargo/config.toml) is configured to link with [Mold](https://github.com/rui314/mold). You can either install it by following the instructions on their repo or remove the configuration from [`.cargo`](.cargo/config.toml) to use the default linker. Another point to note is that it compiles for `skylake` microarchitecture. This is reasonable baseline unless you are using a CPU from < 2015. If you are using a CPU from < 2015, you can remove the configuration from [`.cargo`](.cargo/config.toml) to use the default microarchitecture.

### Building the backend 

Run the following command from root of the repo

```bash
cargo build --release
```

### Cross Compiling the Rust Backend 

If you are going to run the binary on a machine that is not the same as the one you are building on, you will need to cross compile the binary. You can do this by running the following command from root of the repo

```bash
cargo build --release --target <target-triple>
```

At the moment, only x86_64/ARM64 Linux and ARM64 Apple Sillicon are tested. You can find the target triples for these platforms below:

- x86_64 Linux: `x86_64-unknown-linux-gnu`
- ARM64 Linux: `aarch64-unknown-linux-gnu`
- ARM64 Apple Silicon: `aarch64-apple-darwin`

While windows is not tested, there is a very high chance that it will work. The target triple for windows is `x86_64-pc-windows-gnu`

### Building the frontend

Run the following from `apps/web` directory

```bash
pnpm install
pnpm run build
```

### Running both backend and frontend

For local development, use Docker Compose to run PostgreSQL, then run the backend and frontend on your host machine.

From the repo root, start PostgreSQL:

```bash
docker compose up -d
```

Then run these commands from 2 different terminals.

```bash
cargo run -- --host localhost:4002 --port 4002 --postgres-host localhost --postgres-port 5432 --postgres-user urlshort --postgres-password urlshort --postgres-db urlshort
```

```bash
cd apps/web
pnpm run dev
```

### Troubleshooting

#### Frontend does not start

Make sure you have installed the frontend dependencies:

```bash
cd apps/web
pnpm install
```

Also check that you are using the correct Node.js version. The required version is listed in the `engines` field of `apps/web/package.json`.

#### Backend cannot connect to PostgreSQL

The Rust backend checks PostgreSQL readiness on boot. If it cannot connect, it will exit.

Check that PostgreSQL is running and that your connection details match the values used by the backend:

- host
- port
- database name
- username
- password

You can inspect the backend logs for PostgreSQL connection errors.

#### Running PostgreSQL with Docker Compose

This repo includes a Docker Compose file for local PostgreSQL. By default, only PostgreSQL starts because the app services are behind the `app` profile (see [Running end-to-end](#running-end-to-end-docker-compose)).

From the repo root, run:

```bash
docker compose up -d
```

This starts a PostgreSQL instance on port `5432` using the credentials shown in the setup command above. To run the full stack instead, use `docker compose --profile app up --build`.

To customize ports or credentials, copy the Compose env example first:

```bash
cp .env.example .env
```

## Deployment

Production runs on **[Railway](https://railway.com)**, both the backend and frontend services live
in a single Railway project. Railway is *not* required for local evaluation; everything above
(Docker Compose / `cargo run` / `pnpm dev`) is for local development. Rationale and the
IaC-vs-CDN ownership discussion live in [`docs/decision.md`](docs/decision.md) (D3).

### Project topology (Infrastructure as Code)

Project topology is defined declaratively in [`.railway/railway.ts`](.railway/railway.ts) using
Railway's IaC DSL:

- `postgres("postgres")` — managed Postgres
- `shortener-api` — the Rust backend, deployed from the pinned GHCR image tag, exposed at
  `https://s.inve.rs`, wired to Postgres, and configured to allow the frontend origin
- `web` — the SolidJS frontend, built from `apps/web` (`pnpm install --frozen-lockfile && pnpm build`)
  and served by **Caddy** from `dist/` via [`apps/web/Caddyfile`](apps/web/Caddyfile), exposed at
  `https://shorter.inve.rs`

Production URLs are configured with custom Railway domains: `shorter.inve.rs` for the frontend and
`s.inve.rs` for the backend. The backend's `HOST` value is the bare host (`s.inve.rs`), matching
the API's self-reference guard parser. CI can override these defaults with `SHORTENER_WEB_DOMAIN`
and `SHORTENER_API_DOMAIN` when applying the Railway config.

### Apply changes

From the repo root, after installing the Railway CLI:

```bash
railway login
railway link                      # select project + environment once

railway config plan               # preview the diff vs the live environment
railway config apply              # apply after confirmation
# or, in CI:
railway config apply --yes --confirm-destructive
```

For pre-merge testing from a Railway deployment branch, point the GitHub source at that branch
explicitly:

```bash
RAILWAY_GITHUB_BRANCH=chore/deploy-railway railway config plan
RAILWAY_GITHUB_BRANCH=chore/deploy-railway railway config apply
```

Leave `RAILWAY_GITHUB_BRANCH` unset for normal production deploys; Railway then builds from the
repository default branch.

The backend image tag defaults to `0.1.1`. To deploy a specific published GHCR image tag, pass
`SHORTENER_API_TAG`:

```bash
SHORTENER_API_TAG=0.1.1 railway config plan
SHORTENER_API_TAG=0.1.1 railway config apply
```

CI should set `SHORTENER_API_TAG` from the release version it wants Railway to deploy.
The `release-plz.yml` workflow does this automatically after the release image is pushed to GHCR
and the Trivy scan passes, then runs `railway config apply --yes` against the `production`
environment.

The custom domains default to `shorter.inve.rs` for the frontend and `s.inve.rs` for the backend.
To deploy the same topology with different domains, pass `SHORTENER_WEB_DOMAIN` and
`SHORTENER_API_DOMAIN`:

```bash
SHORTENER_WEB_DOMAIN=preview.example.com \
SHORTENER_API_DOMAIN=api-preview.example.com \
railway config plan
```

The API domain must be a bare host because it is also used as the backend `HOST` value.

`railway config plan --detailed-exit-code` exits `2` on drift, suitable for gating CI on
unapplied changes to `.railway/railway.ts`.

### CDN (frontend only)

The Railway CDN sits in front of the `web` service. CDN settings are **not** part of
`.railway/railway.ts` (see D3's "IaC–CDN ownership gap" caveat), so they're applied via CLI once
(`/15`):

```bash
railway cdn enable  --service web
railway cdn update  --service web --html-caching force --purge-on-deploy all
```

- **Force** HTML caching — Caddy serves `index.html` without cache headers; the CDN caches it via
  its Default TTL.
- **Purge-on-deploy = all** — every deploy flushes cached assets too, so a fresh build can't
  reference outdated hashed assets.
- CDN is **off** on `shortener-api` — 302 redirects and JSON API responses aren't cacheable
  without explicit `max-age`.

Re-run the CLI step above only if a future `railway config apply` recreates the `web` service.

### Local vs production

| | Local | Production |
| --- | --- | --- |
| Postgres | Docker Compose container | Railway-managed Postgres |
| Backend | `cargo run` (or `docker compose --profile app`) | Railway `shortener-api` service |
| Frontend | `pnpm dev` (or `docker compose --profile app`) | Railway `web` service (Caddy + CDN) |
| TLS | none | Railway edge (Caddyfile uses `auto_https off`) |
