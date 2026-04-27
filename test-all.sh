#! /bin/bash

cargo run --bin yesser-todo-server &
sleep 1
cargo test --all-features
killall yesser-todo-server
