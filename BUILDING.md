# Building from source

Toolchain Prerequisites:

- Rust Compiler
- pnpm and Node.js

> [!NOTE]
> At the current moment if you are compiling for x86_64 Linux, [`.cargo`](.cargo/config.toml) is configured to link with [Mold](https://github.com/rui314/mold). You can either install it by following the instructions on their repo or remove the configuration from [`.cargo`](.cargo/config.toml) to use the default linker. Another point to note is that it compiles for `skylake` microarchitecture. This is reasonable baseline unless you are using a CPU from < 2015. If you are using a CPU from < 2015, you can remove the configuration from [`.cargo`](.cargo/config.toml) to use the default microarchitecture.

## Building the backend

Run the following command from root of the repo

```bash
cargo build --release
```

## Cross Compiling the Rust Backend

If you are going to run the binary on a machine that is not the same as the one you are building on, you will need to cross compile the binary. You can do this by running the following command from root of the repo

```bash
cargo build --release --target <target-triple>
```

At the moment, only x86_64/ARM64 Linux and ARM64 Apple Sillicon are tested. You can find the target triples for these platforms below:

- x86_64 Linux: `x86_64-unknown-linux-gnu`
- ARM64 Linux: `aarch64-unknown-linux-gnu`
- ARM64 Apple Silicon: `aarch64-apple-darwin`

While windows is not tested, there is a very high chance that it will work. The target triple for windows is `x86_64-pc-windows-gnu`

## Building the frontend

Run the following from the repo root:

```bash
pnpm install
pnpm --filter web run build
```

## Running both backend and frontend

For local development, use Docker Compose to run PostgreSQL, then run the backend and frontend on your host machine.

From the repo root, start PostgreSQL:

```bash
docker compose up -d
```

Then run these commands from 2 different terminals.

```bash
cargo run -- --disallowed-hosts localhost:4002 --port 4002 --postgres-host localhost --postgres-port 5432 --postgres-user urlshort --postgres-password urlshort --postgres-db urlshort
```

```bash
pnpm --filter web run dev
```

## Troubleshooting

### Frontend does not start

Make sure you have installed the frontend dependencies:

```bash
pnpm install
```

Also check that you are using the correct Node.js version. The required version is listed in the `engines` field of `apps/web/package.json`.

### Backend cannot connect to PostgreSQL

The Rust backend checks PostgreSQL readiness on boot. If it cannot connect, it will exit.

Check that PostgreSQL is running and that your connection details match the values used by the backend:

- host
- port
- database name
- username
- password

You can inspect the backend logs for PostgreSQL connection errors.

### Running PostgreSQL with Docker Compose

This repo includes a Docker Compose file for local PostgreSQL. By default, only PostgreSQL starts because the app services are behind the `app` profile (see [Running end-to-end](README.md#running-end-to-end-docker-compose)).

From the repo root, run:

```bash
docker compose up -d
```

This starts a PostgreSQL instance on port `5432` using the credentials shown in the setup command above. To run the full stack instead, use `docker compose --profile app up --build`.

To customize ports or credentials, copy the Compose env example first:

```bash
cp .env.example .env
```
