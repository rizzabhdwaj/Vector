# Vector chaos workload with k6

Start the coordinator and workers, install [Grafana k6](https://grafana.com/docs/k6/latest/set-up/install-k6/),
then run from the repository root:

```sh
k6 run scripts/chaos.js
```

The default run attempts 40 task submissions across 3 concurrent k6 virtual
users. Workers get a random mix of `echo`, short `sleep`, and sleep-then-print
tasks. About 15% deliberately exit nonzero or exceed their execution time limit.
Some submissions are back-to-back; others pause for up to 1.5 seconds per user.
Vector runs executables directly, so the sleep-then-print case explicitly uses
`sh -c` with a fixed script and positional arguments. `sleep && printf` waits
before printing; `sleep | echo` would not.

```sh
# Small success-only run
TASKS=8 VUS=1 FAILURE_PERCENT=0 k6 run scripts/chaos.js

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

Use a dedicated dev/test instance: accepted jobs execute on workers and write
records to Vector's database. k6 checks that every submission returned HTTP
`201` and the run finished all iterations. A `201` means **queued**, not
**executed**. The workers can still be processing tasks after k6 exits.
This script does not stop workers or kill services.

With [Prometheus running](../monitoring/README.md), watch queue depth and outcomes:

```promql
coordinator_tasks_queued
sum by (status) (increase(worker_tasks_total[5m]))
```

Run these as separate queries. Worker failures are expected when
`FAILURE_PERCENT` is nonzero, even if every HTTP submission passes.
