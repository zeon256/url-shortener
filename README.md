# url-shortener

## Running end to end with Docker 

> [!TIP]
> Not to be confused with development. 

If you just want to run the system end to end without installing any of the mentioned toolchains, use the `app` Compose profile from the repo root:

```bash
docker compose --profile app up --build
```

This will build both frontend and backend using multistage builds as well as spawn the database. You can then access the frontend at `http://localhost:8080` and the backend at `http://localhost:8000`.

To run the full stack in the background, add `-d`:

```bash
docker compose --profile app up --build -d
```

### Common Pitfalls and debugging for Docker

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

This repo includes a Docker Compose file for local PostgreSQL. By default, only PostgreSQL starts because the app services are behind the `app` profile.

From the repo root, run:

```bash
docker compose up -d
```

This starts a PostgreSQL instance on port `5432` using the credentials shown in the setup command above. To customize the ports or credentials, copy the Compose env example first:

```bash
cp .env.example .env
```
