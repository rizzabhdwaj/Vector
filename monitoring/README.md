# Prometheus for Vector

This integration is available on `main` and `prometheus_metric`. Restart the
coordinator and workers after rebuilding: existing processes keep running their
old binaries.

Prometheus periodically GETs each process's metrics endpoint ("scraping") and
stores the samples in its own time-series database, separate from Vector's
Postgres database. No task payloads or outputs are exported.

## Start everything

From the repository root:

```sh
git switch main
# Terminal 1: uses the existing coordinator/.env and Postgres setup
cd coordinator
cargo run
```

In another terminal, from the repository root:

```sh
bash worker/run.sh
```

This builds and starts five workers. Each has a separate HTTP metrics port:

| Process | Task/API port | Metrics URL |
| --- | --- | --- |
| Coordinator | HTTP 3030, gRPC 50052 | http://localhost:3030/metrics |
| Worker 5001 | gRPC 5001 | http://localhost:9101/metrics |
| Worker 5002 | gRPC 5002 | http://localhost:9102/metrics |
| Worker 5003 | gRPC 5003 | http://localhost:9103/metrics |
| Worker 5004 | gRPC 5004 | http://localhost:9104/metrics |
| Worker 5005 | gRPC 5005 | http://localhost:9105/metrics |

`/metric` is an equivalent alias on every process. For a manually started worker
set `WORKER_ID`, `WORKER_GRPC`, and `WORKER_METRICS`; the metrics server defaults
to `:9100` if unset, so five manual workers need distinct ports.

Start Prometheus with Docker Compose (Linux host networking):

```sh
docker compose -f monitoring/compose.yml up -d
```

Alternatively, with the Prometheus binary installed, run from the repository root:

```sh
prometheus --config.file=monitoring/prometheus.yml
```

Use only one of these options at a time. Open http://localhost:9090/targets:
there should be **seven UP targets** (Prometheus itself, one coordinator, and
five workers). The jobs are `prometheus`, `vector-coordinator`, and
`vector-workers`; scrape and evaluation intervals are 15 seconds. The Docker
setup keeps samples in a named volume for 15 days.

## What to query

Open http://localhost:9090 and enter these PromQL expressions:

```promql
# Is each endpoint reachable? 1 = yes, 0 = scrape failed.
up

# Queue length and worker availability
coordinator_tasks_queued
coordinator_workers_free
coordinator_workers_busy
coordinator_workers_dead

# Successful tasks per second, across all five workers
sum(rate(worker_tasks_total{status="success"}[5m]))

# Failures over the last five minutes, grouped by worker metrics target
sum by (instance) (increase(worker_tasks_total{status="failure"}[5m]))

# 95th percentile execution time, per worker metrics target, in seconds
histogram_quantile(0.95, sum by (le, instance) (rate(worker_task_duration_seconds_bucket[5m])))

# Seconds since each worker last sent a successful heartbeat
time() - worker_last_heartbeat_timestamp_seconds
```

Submit tasks in the dashboard and watch the counters increase. Rates need
multiple scrapes before they show useful values. Counters reset when a process
restarts; `rate` and `increase` account for resets.

The coordinator also exports `coordinator_tasks_created_total`,
`coordinator_tasks_assigned_total`, `coordinator_tasks_completed_total`,
`coordinator_tasks_failed_total`, `coordinator_heartbeats_received_total`, and
`coordinator_workers_reaped_total`. Completed counts successful executions;
failed includes execution and dispatch errors.

Worker counters include `worker_heartbeats_total{status="success"|"failure"}`.
Go runtime/process metrics are exported by the Go client library too.
Use Prometheus's `up` to detect unreachable processes, not `worker_up`: a dead
process cannot export a zero. Use the `instance` label to distinguish worker
targets; it comes from each target's address in the scrape config.

## Troubleshooting and stopping

```sh
curl http://localhost:3030/metrics
curl http://localhost:9101/metrics
docker compose -f monitoring/compose.yml logs prometheus
docker compose -f monitoring/compose.yml down
```

If a target is DOWN, check its error on `/targets`, then check that its process
is running and its port matches `prometheus.yml`. Host networking lets the
Linux container reach these localhost services; a default bridge network does
not. Stop old worker/coordinator instances before starting new ones on the same
ports. Stopping Compose preserves the metrics volume.
