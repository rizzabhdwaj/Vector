#!/usr/bin/env bash
set -euo pipefail

# Usage: ./worker/kill_worker.sh <worker_id>
# Kills the Go worker process that has gRPC endpoint localhost:<worker_id>.

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <worker_id>"
  exit 1
fi

WORKER_ID=$1
# Expected gRPC address for this worker
GRPC_ADDR="localhost:${WORKER_ID}"

# Search all PIDs for one whose environment contains the matching gRPC address.
# We look through /proc/[0-9]*/environ for a match.
FOUND_PID=""

for pid_dir in /proc/[0-9]*/; do
  pid=$(basename "$pid_dir")
  # Quick check: ensure it's a running process
  if [[ ! -r "$pid_dir/cmdline" ]]; then
    continue
  fi
  # Read the environment; note that /proc/PID/environ may have NUL separated entries.
  env=$(tr '\0' '\n' < "$pid_dir/environ" 2>/dev/null | grep -i "WORKER_GRPC=" || true)
  if [[ -n "$env" ]]; then
    # Extract the value after =
    grpc_val=$(echo "$env" | cut -d= -f2-)
    if [[ "$grpc_val" == "$GRPC_ADDR" ]]; then
      FOUND_PID=$pid
      break
    fi
  fi
done

if [[ -z "$FOUND_PID" ]]; then
  echo "No worker process found with gRPC address $GRPC_ADDR"
  exit 1
fi

echo "Found worker process PID=$FOUND_PID with gRPC address $GRPC_ADDR"
kill -9 "$FOUND_PID" 2>/dev/null && echo "Worker $WORKER_ID (PID $FOUND_PID) killed." || echo "Failed to kill PID $FOUND_PID"
echo "The coordinator reaper will mark the worker as dead after 15s (timeout)."