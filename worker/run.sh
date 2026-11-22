#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")"

build_dir=$(mktemp -d)
pids=()
cleanup() {
  if ((${#pids[@]})); then
    kill "${pids[@]}" 2>/dev/null || true
    wait "${pids[@]}" 2>/dev/null || true
  fi
  rm -rf "$build_dir"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

go build -o "$build_dir/worker" .

for port in {5001..5005}; do
  WORKER_ID="$port" WORKER_GRPC="localhost:$port" "$build_dir/worker" &
  pids+=("$!")
done

wait -n
