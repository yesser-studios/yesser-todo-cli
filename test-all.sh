#!/usr/bin/env bash
set -euo pipefail

cargo run --bin yesser-todo-server &
server_pid=$!
trap 'kill "$server_pid" 2>/dev/null || true' EXIT

# Wait for readiness instead of fixed sleep
for _ in {1..30}; do
  if curl -fsS "http://127.0.0.1:6982/tasks" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

cargo test --verbose --all-features
