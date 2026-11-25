# Vector

Vector is a distributed task queue and execution engine: a Rust coordinator
accepts commands, schedules them across Go workers, stores task state in
PostgreSQL, and pushes live updates to a React dashboard.

The system is intended for **trusted local networks**. Submitted commands run
directly on workers, and the coordinator/dashboard have no authentication.

## Components

| Directory | Role | Stack |
|---|---|---|
| `coordinator/` | HTTP API, gRPC heartbeats and dispatch, scheduler, worker reaper, WebSocket broadcast, PostgreSQL persistence | Rust (Actix Web, Tonic, SQLx, Tokio) |
| `worker/` | Receives tasks over gRPC, executes them, sends heartbeats, exposes metrics | Go |
| `dashboard/` | Task submission and live worker/queue/output views | React, TypeScript, Vite |
| `monitoring/` | Prometheus scrape configuration | Docker Compose / Prometheus |
| `docs/architecture/` | Architecture notes and diagram | Markdown, Excalidraw |

## Architecture

```text
Browser ──HTTP / WebSocket──▶ Dashboard (Vite dev proxy)
                                  │
                    HTTP 3030 ────┼──── WebSocket 3031
                                  ▼
                          Rust coordinator
                       ┌──────────┼───────────┐
                  gRPC │          │           │ SQL
                 50052 │          │           ▼
                       ▼          │      PostgreSQL
                    Go workers    │
                 gRPC :5001…5005  │
                       ▲──────────┘
                  gRPC task dispatch to the worker port
```

1. The dashboard posts a task to `POST /tasks`; the coordinator persists it in
   `tasks_pending` and adds it to an in-memory FIFO queue.
2. The scheduler polls for workers marked free and calls their `FetchTask` gRPC
   method (worker port equals its worker ID).
3. The worker executes the command without a shell and returns its output.
4. The coordinator stores the output, frees the worker, and broadcasts new
   worker/queue snapshots to every WebSocket client.
5. Workers send heartbeats to coordinator gRPC port `50052`. A reaper marks
   workers dead when heartbeats stop.

See [docs/architecture/overview.md](docs/architecture/overview.md) for more.

## Ports

| Port | Protocol | Purpose |
|---|---|---|
| `3030` | HTTP | Coordinator REST API and Prometheus metrics |
| `3031` | WebSocket | Live dashboard updates (bound to coordinator loopback) |
| `50052` | gRPC | Worker → coordinator heartbeats |
| `5001`–`5005` | gRPC | One task-execution server per worker |
| `9101`–`9105` | HTTP | Worker Prometheus metrics |
| `5173` | HTTP | Vite dashboard in development |

The Vite dev server proxies `/vector-api` to `127.0.0.1:3030` (removing the
prefix) and `/vector-ws` to `127.0.0.1:3031`. This avoids browser hostname and
mixed-protocol problems; run Vite and the coordinator on the same host.

## Prerequisites

- Rust toolchain (the project uses edition 2024)
- Go
- Node.js and npm
- PostgreSQL
- protoc (needed by the coordinator build to compile the gRPC protobufs)
- Docker Compose or Prometheus, only if you want metrics

## Setup

### 1. PostgreSQL

Create a `vector` database accessible with the connection string in
`coordinator/.env` (default: `postgres://localhost:5432/vector`). The tables are
defined in `coordinator/migrations/`; apply them if your database is empty:

```sh
for file in coordinator/migrations/*.sql; do psql "$DATABASE_URL" -f "$file"; done
```

### 2. Coordinator

```sh
cd coordinator
cargo run
```

Useful checks:

```sh
cargo test
cargo build --release
```

`coordinator/.env` supplies `DATABASE_URL`. The coordinator starts the REST API,
gRPC heartbeat server, scheduler, reaper, persister, and WebSocket server in
one process. Stop existing instances before starting another one — ports 3030,
3031, and 50052 must be free.

### 3. Workers

From the repository root, start five workers (IDs 5001–5005):

```sh
bash worker/run.sh
```

Ctrl+C stops them all. Each worker reads `COORDINATOR_GRPC` from `worker/.env`
(`localhost:50052` by default) and listens for tasks on the port matching its
`WORKER_ID`. To stop a single worker by its gRPC port:

```sh
bash worker/kill_worker.sh 5003
```

For a manually started worker, set `WORKER_ID`, `WORKER_GRPC` (listen address),
`WORKER_METRICS` (metrics listen address), and `COORDINATOR_GRPC` (coordinator
address).

### 4. Dashboard

```sh
cd dashboard
npm install
npm run dev
```

Open the URL printed by Vite (normally http://localhost:5173). Workers appear
within about five seconds of their first heartbeat. See
[dashboard/DASHBOARD.md](dashboard/DASHBOARD.md) for deployment notes,
`npm run build`, and `npm run lint`.

## HTTP API

| Method | Path | Description |
|---|---|---|
| `POST` | `/tasks` | Persist and enqueue a task; body is `{ "id", "command", "args", "status" }` |
| `GET` | `/outputs` | List task outputs currently held in memory |
| `GET` | `/metric`, `/metrics` | Prometheus exposition format |

There is no endpoint for submitting a task via gRPC: workers implement the
`FetchTask` RPC that the coordinator calls.

For a production deployment, put the dashboard and coordinator's WebSocket
listener behind the same reverse proxy (or set `VITE_API_BASE_URL` and
`VITE_WS_BASE_URL` before building the dashboard). Use `wss://` when the page is
served over HTTPS.

## Monitoring

Start the coordinator and workers first, then Prometheus:

```sh
docker compose -f monitoring/compose.yml up -d
```

Open http://localhost:9090/targets; expect one coordinator target and five
worker targets to be UP. The full metric list, example PromQL queries, and
troubleshooting steps are in [monitoring/README.md](monitoring/README.md).

## Security notes

There is no authentication or TLS. Commands are executed directly (not through
a shell), but a submitted command can still do anything the worker process can.
Keep the coordinator, workers, and dashboard on a trusted network.
