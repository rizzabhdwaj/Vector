#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 || ! $1 =~ ^[0-9]{1,5}$ ]]; then
  echo "Usage: $0 <worker-grpc-port> (example: $0 5003)" >&2
  exit 1
fi

port=$((10#$1))
if ((port < 1 || port > 65535)); then
  echo "Port must be between 1 and 65535." >&2
  exit 1
fi

command -v ss >/dev/null || { echo "ss is required (iproute2)." >&2; exit 1; }
mapfile -t pids < <(ss -H -ltnp "sport = :$port" |
  grep -oE 'pid=[0-9]+' | cut -d= -f2 | sort -u)

if [[ ${#pids[@]} -ne 1 ]]; then
  echo "Expected one visible process listening on gRPC port $port; found ${#pids[@]}." >&2
  exit 1
fi

pid=${pids[0]}
exe=$(readlink "/proc/$pid/exe") || exit 1
if [[ ${exe##*/} != worker && ${exe##*/} != "worker (deleted)" ]]; then
  echo "Port $port belongs to $exe (PID $pid), not the worker binary." >&2
  exit 1
fi

kill -TERM "$pid"
echo "Sent SIGTERM to worker on gRPC port $port (PID $pid)."
