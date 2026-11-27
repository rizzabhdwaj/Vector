# Vector chaos workload with k6

Start the coordinator and install [Grafana k6](https://grafana.com/docs/k6/latest/set-up/install-k6/).
From the repository root, start the workers with their chaos routes enabled:

```sh
WORKER_ENABLE_CHAOS=1 bash worker/run.sh
```

In another terminal, run:

```sh
k6 run scripts/chaos.js
```

The default run attempts 40 task submissions across 3 concurrent k6 virtual
users. Workers get a random mix of `echo`, short `sleep`, and sleep-then-print
tasks. About 15% deliberately exit nonzero or exceed their execution time limit.
Some submissions are back-to-back; others pause for up to 1.5 seconds per user.
During submissions, k6 also pauses the heartbeats of **one or two randomly
selected workers** for 25 seconds. The workers keep running and resume their
heartbeats automatically, even if k6 has already finished.
Vector runs executables directly, so the sleep-then-print case explicitly uses
`sh -c` with a fixed script and positional arguments. `sleep && printf` waits
before printing; `sleep | echo` would not.

```sh
# Small task-only run: no intentional task failures or heartbeat pauses
TASKS=8 VUS=1 FAILURE_PERCENT=0 HEARTBEAT_PAUSES=0 k6 run scripts/chaos.js

# Burst: 60 submissions, up to 6 concurrent clients, no deliberate pauses
TASKS=60 VUS=6 MAX_PAUSE_MS=0 k6 run scripts/chaos.js
```

| Variable | Default | Meaning |
| --- | --- | --- |
| `VECTOR_URL` | `http://127.0.0.1:3030` | Coordinator base URL |
| `TASKS` | `40` | Total submissions, not per virtual user |
| `VUS` | `3` (or fewer if `TASKS` is smaller) | Concurrent submitters, at most 50 |
| `MAX_PAUSE_MS` | `1500` | Maximum random pause; `0` disables pauses |
| `FAILURE_PERCENT` | `15` | Per-task probability of an intentional worker failure (`0`–`100`) |
| `HEARTBEAT_PAUSES` | Randomly `1` or `2` | Set `0` to disable, or choose `1` or `2`. Two require at least two tasks and two worker URLs. |
| `HEARTBEAT_PAUSE_SECONDS` | `25` | Duration per worker (`1`–`300` seconds); use longer than 15 seconds to exercise the coordinator's dead-worker detection. |
| `WORKER_URLS` | `http://127.0.0.1:9101` through `:9105` | Comma-separated worker metrics base URLs; required with a non-local coordinator when pauses are enabled. |

Use a dedicated dev/test instance: accepted jobs execute on workers and write
records to Vector's database. k6 checks that every submission returned HTTP
`201`, every requested pause returned HTTP `204`, and the run finished all
iterations. A `201` means **queued**, not
**executed**. The workers can still be processing tasks after k6 exits.
This script does not stop worker processes or kill services.

With [Prometheus running](../monitoring/README.md), watch queue depth and outcomes:

```promql
coordinator_tasks_queued
coordinator_workers_dead
sum by (status) (increase(worker_tasks_total[5m]))
```

Run these as separate queries. Worker failures are expected when
`FAILURE_PERCENT` is nonzero, even if every HTTP submission passes.

## Pause a worker's heartbeat

For local testing, restart workers with the opt-in chaos route enabled:

```sh
WORKER_ENABLE_CHAOS=1 bash worker/run.sh
```

Pause worker `5001` via its HTTP metrics port, `9101`:

```sh
curl -fsS -X POST 'http://localhost:9101/chaos/heartbeat/pause?seconds=25'
```

HTTP `204` (no body) means the pause was scheduled. The duration must be an
integer from 1 to 300. A new request can extend an active pause but cannot
shorten it. The worker resumes heartbeats automatically; an in-flight heartbeat
may finish and the worker continues serving metrics and tasks. This route has no
authentication, so only enable it on a trusted test listener.

Graph `time() - worker_last_heartbeat_timestamp_seconds{worker_id="5001"}` to see
the heartbeat age rise and fall. `up{worker_id="5001"}` should stay `1` while the
coordinator can mark the worker dead after 15 seconds without a heartbeat.
