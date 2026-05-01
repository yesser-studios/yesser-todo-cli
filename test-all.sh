#!/usr/bin/env bash
set -euo pipefail

cargo build --bin yesser-todo-server

cargo run --bin yesser-todo-server &
server_pid=$!
trap 'kill "$server_pid" 2>/dev/null || true' EXIT

ready=false
# Wait for readiness instead of fixed sleep
for _ in {1..30}; do
  if curl -fsS "http://127.0.0.1:6982/tasks" >/dev/null 2>&1; then
    ready=true
    break
  fi
  sleep 1
done

if [ "$ready" != "true" ]; then
  echo "Server did not become ready within 30 seconds" >&2
  exit 1
fi


cargo test --verbose --all-features
