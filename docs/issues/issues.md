# Issues - (found after prototype)


## Solved 

## Unsolved

[IMPORTANT]
- A failed dispatch leaks the worker. The task is already popped from the queue, and on an RPC error the worker stays ``available = false`` forever. The task is lost and the slot is gone.
    - Add last_seen to workers, and have a reaper mark workers dead after about 3 missed heartbeats.
    - Add a proper task state machine (Queued → Running → Succeeded/Failed) with server-generated UUIDs.
    - Requeue tasks from dead workers, add timeouts on both sides, and set a max-retries limit.
    - Write a chaos script that kills random workers while tasks run and checks that every task still completes.

[Yeah aight (Also Important)]
- Heartbeats don’t do anything. ``update_worker`` returns early for known workers, so there’s no ``last_seen`` and no dead-worker detection. A killed worker stays in the registry and keeps receiving tasks.
- Worker address is localhost:{worker_id}. The ID doubles as the port, which blocks running workers on other machines. Workers should register with an address.
- No timeouts anywhere. exec.Command has no context and the gRPC call has no deadline, so one hung task reserves a worker forever.
- Task state is thin. The Status enum is never used, Result.success is dropped, and stderr and exit codes are discarded. The client also has to supply status in the POST body, and task IDs are client-chosen u32s that can collide. outputs grows without bound.
- The worker will crash for anyone who clones the repo. Heartbeat() calls godotenv.Load() every 5 seconds and does log.Fatal if .env is missing, but .env is gitignored. It also reopens a gRPC connection on every heartbeat.
- Security. Workers run arbitrary commands, with no auth and Cors::permissive(). You know this, so put a loud warning in the README.
- The README overclaims. It says “fault tolerance, horizontal scaling,” which aren’t implemented yet. Reviewers will check, so describe what exists today and put the rest in a roadmap. /metrics also returns a JSON string rather than real metrics.
- There are no tests and no CI.
- Everything is in memory. Postgres appears in the architecture doc and sqlx, argon2 and jsonwebtoken are in Cargo.toml, but none of it is used. A coordinator restart loses all tasks.


