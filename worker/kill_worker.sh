#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <worker_id>"
  exit 1
fi

WORKER_ID=$1
# Workers were started with WORKER_ID=5001..5005 and listen on localhost:$WORKER_ID
PORT=$WORKER_ID

# Find the PID listening on that TCP port
PID=$(fuser -t tcp/${PORT}/ 2>/dev/null | head -1 | awk '{print $1}' | grep -E '^[0-9]+$' || true)

if [[ -z "$PID" ]]; then
  echo "No process found listening on port $PORT (worker_id=$WORKER_ID)"
  exit 1
fi

# Optional: verify the process is a worker (check cmdline)
if ! cat /proc/$PID/cmdline 2>/dev/null | tr '\0' ' ' | grep -q worker; then
  echo "PID $PID is not a worker process"
  exit 1
fi

echo "Killing worker $WORKER_ID (PID $PID)"
kill -9 "$PID" 2>/dev/null || echo "Failed to kill PID $PID"
echo "Worker $WORKER_ID killed. The reaper will mark it dead after 15s."